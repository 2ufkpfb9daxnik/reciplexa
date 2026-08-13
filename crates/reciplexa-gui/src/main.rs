//! Paper preview with drag → CST sync, plus a live `.rpx` source pane.

use reciplexa_gui::canvas_sync::{
    nudge_authoring_layers, resolve_preview_layers, resolve_preview_size_targets, SyncRefuse,
};
use reciplexa_gui::document_state::DocumentPathState;
use reciplexa_gui::fonts::install_cjk_fonts;
use reciplexa_gui::preview_paint::{
    apply_aspect_lock, box_from_grab, hit_line_endpoint, hit_rotate_handle, hit_scale_grab,
    paint_hover_frame, paint_line_endpoints, paint_paper_grid, paint_selection_frame, paint_shape,
    snap_mm, BoxDrag, ScaleGrab,
};

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use eframe::egui;
use eframe::egui::text::{CCursor, CCursorRange};
use reciplexa::document_pipeline::document_snapshot_from_source;
use reciplexa::pipeline::{document_for_export, document_from_source};
use reciplexa_codec::{compact_after_save, write_journal, RecoveryPaths};
use reciplexa_effect::{seed_from_env, EffectError, EffectHandler, LcgRng, Value};
use reciplexa_gui_runtime::GuiRuntimeHost;
use reciplexa_identity::document::{DocumentIdentity, StableNodeId};
use reciplexa_lower::{
    collect_layer_props, collect_layers_page, collect_size_targets_page, delete_layer_page,
    delete_page, duplicate_layer_page, group_layers_page, insert_layer_page, insert_page_after,
    layer_rotation_deg, nudge_layer_page, reorder_layer_page, scale_layer_uniform,
    scale_size_target, set_box_xywh, set_layer_prop, set_layer_rotation_deg, set_layers_fill_rgb,
    set_layers_opacity, set_layers_stroke_rgb, set_layers_stroke_width, set_line_endpoint,
    set_poly_vertex, set_text_box, ungroup_layer_page, LayerInfo, PropEditContext, PropGroup,
    PropValue, SizeTarget,
};
use reciplexa_macro::expand_source;
use reciplexa_pdf::write_document_with_base;
use reciplexa_view::{
    flatten_page, hit_test_shapes, shapes_intersecting_aabb, PaperLayout, WorldShape,
};

use reciplexa_gui::prefs::{apply_ui_theme, GuiPrefs, UiTheme};

/// Live preview: expand + typecheck + lower **without** running `(src)` effects.
fn pipeline_doc(src: &str) -> Result<reciplexa_scene::Document, String> {
    document_from_source(src).map_err(|e| e.display())
}

struct GuiExportHandler {
    rng: LcgRng,
}

impl Default for GuiExportHandler {
    fn default() -> Self {
        // Same env seed as CLI so export is reproducible across hosts.
        Self {
            rng: LcgRng::new(seed_from_env()),
        }
    }
}

impl EffectHandler for GuiExportHandler {
    fn on_log(&mut self, message: &str) -> Result<Value, EffectError> {
        // Residual effect only (not REPL result dumping).
        eprintln!("{message}");
        Ok(Value::Unit)
    }

    fn on_random(&mut self) -> Result<Value, EffectError> {
        Ok(Value::Number(self.rng.next_unit()))
    }

    fn on_write_path(&mut self, path: &str) -> Result<Value, EffectError> {
        eprintln!("{path}");
        Ok(Value::Unit)
    }
}

fn suppress_ime_confirm_newline(ctx: &egui::Context, hold_frames: &mut u8) {
    // egui 0.31: IME 確定の Enter が同じフレームで改行として漏れることがある。
    // Commit があるフレームとその直後は Enter / "\n" を落とす。
    ctx.input_mut(|input| {
        let has_ime = input
            .events
            .iter()
            .any(|e| matches!(e, egui::Event::Ime(_)));
        let has_commit = input
            .events
            .iter()
            .any(|e| matches!(e, egui::Event::Ime(egui::ImeEvent::Commit(_))));
        if has_commit {
            *hold_frames = (*hold_frames).max(2);
        }
        if has_ime || *hold_frames > 0 {
            input.events.retain(|e| {
                let is_enter = matches!(
                    e,
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        ..
                    }
                );
                let is_newline = matches!(e, egui::Event::Text(t) if t == "\n" || t == "\r\n");
                !(is_enter || is_newline)
            });
        }
    });
    if *hold_frames > 0 {
        *hold_frames -= 1;
    }
}

fn byte_to_char_index(s: &str, byte: usize) -> usize {
    let byte = byte.min(s.len());
    s[..byte].chars().count()
}

fn highlight_source_range(ctx: &egui::Context, source: &str, start: usize, end: usize) {
    let id = egui::Id::new("rpx_source_editor");
    let c0 = byte_to_char_index(source, start);
    let c1 = byte_to_char_index(source, end);
    if let Some(mut state) = egui::text_edit::TextEditState::load(ctx, id) {
        state
            .cursor
            .set_char_range(Some(CCursorRange::two(CCursor::new(c0), CCursor::new(c1))));
        state.store(ctx, id);
    }
}

fn escape_lisp_string(s: &str) -> String {
    // SYN-001 §8: complete literal; no backslash escapes.
    reciplexa_macro::escape_lisp_string(s)
}

/// Prefer a path relative to the `.rpx` directory, using `/` separators.
fn path_for_rpx(doc_path: &Path, asset: &Path) -> String {
    if let Some(base) = doc_path.parent() {
        if let Ok(rel) = asset.strip_prefix(base) {
            return rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
        }
    }
    asset
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn set_window_title(ctx: &egui::Context, path: &Path, dirty: bool) {
    let star = if dirty { " •" } else { "" };
    ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
        "reciplexa — {}{star}",
        path.display()
    )));
}

fn main() -> ExitCode {
    let (path, src) = match env::args().nth(1) {
        Some(p) => {
            let path = PathBuf::from(p);
            match fs::read_to_string(&path) {
                Ok(s) => (path, s),
                Err(e) => {
                    eprintln!("error: read {}: {e}", path.display());
                    return ExitCode::FAILURE;
                }
            }
        }
        None => (
            PathBuf::from("untitled.rpx"),
            "(import graphics/page only a4 page)\n(val main (page a4 (list)))\n".to_string(),
        ),
    };
    // Keep the author's `.rpx` text. Expansion happens inside `pipeline_doc` /
    // export so Scribble `(markup …)` macros (`@title`, …) stay editable.
    let initial_error = pipeline_doc(&src).err();
    if let Some(ref e) = initial_error {
        eprintln!("warn: opening with error (edit to fix): {e}");
    }

    let title = format!("reciplexa — {}", path.display());
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title(title),
        ..Default::default()
    };

    match eframe::run_native(
        "reciplexa",
        native_options,
        Box::new(move |cc| {
            install_cjk_fonts(&cc.egui_ctx);
            let prefs = GuiPrefs::load();
            apply_ui_theme(&cc.egui_ctx, prefs.theme);
            Ok(Box::new(PreviewApp {
                path,
                source: src.clone(),
                saved_source: src,
                error: initial_error,
                sync_warning: None,
                drag: None,
                page_index: 0,
                zoom: prefs.zoom,
                pan: egui::vec2(prefs.pan_x, prefs.pan_y),
                selected: Vec::new(),
                pending_source_select: None,
                textures: std::collections::HashMap::new(),
                ime_enter_hold: 0,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
                typing_undo_open: false,
                props_open: prefs.show_props,
                props_undo_open: false,
                batch_fill: [0.2, 0.2, 0.2],
                batch_stroke: [0.1, 0.1, 0.1],
                batch_stroke_width: 1.0,
                batch_opacity: 1.0,
                theme: prefs.theme,
                show_grid: prefs.show_grid,
                title_dirty: false,
                viewport_paint_size: egui::Vec2::new(400.0, 400.0),
                viewport_page_mm: (210.0, 297.0),
                viewport_prefs_dirty: false,
                show_source: prefs.show_source,
                show_layers: prefs.show_layers,
                show_preview: prefs.show_preview,
                float_source: prefs.float_source,
                float_layers: prefs.float_layers,
                float_preview: prefs.float_preview,
                float_props: prefs.float_props,
                collapse_source: false,
                collapse_layers: false,
                collapse_preview: false,
                collapse_props: false,
                document_path: DocumentPathState::from_env(),
                gui_runtime: GuiRuntimeHost::new(),
            }))
        }),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: gui: {e}");
            ExitCode::FAILURE
        }
    }
}

struct PreviewApp {
    path: PathBuf,
    source: String,
    /// Last-saved / last-loaded buffer; dirty when differs from `source`.
    saved_source: String,
    error: Option<String>,
    /// Soft CST sync refusal (e.g. markup-expanded layers); does not block preview.
    sync_warning: Option<String>,
    drag: Option<DragState>,
    page_index: usize,
    zoom: f32,
    pan: egui::Vec2,
    selected: Vec<usize>,
    pending_source_select: Option<(usize, usize)>,
    /// Texture cache keyed by image path string from the `.rpx`.
    textures: std::collections::HashMap<String, egui::TextureHandle>,
    /// Frames to keep suppressing Enter after IME Commit.
    ime_enter_hold: u8,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    /// Coalesce TextEdit keystrokes into one undo step while focused.
    typing_undo_open: bool,
    /// Properties panel (named args for the selection). Not the attribute-button suite.
    props_open: bool,
    /// Coalesce property slider drags into one undo step.
    props_undo_open: bool,
    /// Shared fill when multiple shapes are selected (marquee batch).
    batch_fill: [f64; 3],
    /// Shared stroke color for multi-select (line / frame / polyline).
    batch_stroke: [f64; 3],
    /// Shared stroke width (mm) for multi-select.
    batch_stroke_width: f64,
    /// Shared opacity for multi-select batch apply.
    batch_opacity: f64,
    /// UI chrome theme (paper stays light).
    theme: UiTheme,
    /// Draw a light millimeter grid on the paper.
    show_grid: bool,
    /// Last window-title dirty bit (avoid spamming viewport cmds).
    title_dirty: bool,
    /// Last frame's paper preview size (for keyboard zoom anchor).
    viewport_paint_size: egui::Vec2,
    /// Last frame's page size in mm (for keyboard zoom anchor).
    viewport_page_mm: (f64, f64),
    /// Defer writing zoom/pan to prefs until end of frame.
    viewport_prefs_dirty: bool,
    /// Workspace panes (docked by default; □ floats).
    show_source: bool,
    show_layers: bool,
    show_preview: bool,
    float_source: bool,
    float_layers: bool,
    float_preview: bool,
    float_props: bool,
    collapse_source: bool,
    collapse_layers: bool,
    collapse_preview: bool,
    collapse_props: bool,
    /// Optional document-model path (Phase 4 strangler); CST sync remains default.
    document_path: DocumentPathState,
    /// Phase 8 GUI runtime host (ephemeral widget state — not document truth).
    gui_runtime: GuiRuntimeHost,
}

#[derive(Clone, Copy)]
enum PropPreset {
    FillRgb([f64; 3]),
    StrokeWidth(f64),
    Opacity(f64),
}

#[derive(Clone)]
struct DragState {
    kind: DragKind,
    undo_pushed: bool,
}

#[derive(Clone)]
enum DragKind {
    /// World-axis translation of the layer root (never touches rotate/scale).
    Move {
        last_mm: (f64, f64),
        /// All layers moved together (multi-select).
        flat_indices: Vec<usize>,
    },
    Scale {
        size: SizeTarget,
        /// Flatten / layer index for [`scale_layer_uniform`] fallbacks.
        flat_index: usize,
        base_src: String,
        center_mm: (f64, f64),
        start_dist: f64,
        text_box: Option<BoxDrag>,
    },
    /// Drag one endpoint of a 2-point line.
    LineEndpoint {
        index: usize,
        endpoint: usize,
        base_src: String,
    },
    /// Drag one polyline/polygon vertex.
    PolyVertex {
        head: &'static str,
        index: usize,
        vertex: usize,
        base_src: String,
    },
    Rotate {
        flat_index: usize,
        base_src: String,
        center_mm: (f64, f64),
        start_angle_rad: f64,
        base_deg: f64,
    },
    /// Drag on empty paper to range-select intersecting shapes.
    Marquee {
        start_mm: (f64, f64),
        current_mm: (f64, f64),
    },
}

impl PreviewApp {
    fn reload_ok(&self) -> bool {
        pipeline_doc(&self.source).is_ok()
    }

    /// Expanded buffer for read-only layer/size sync (falls back to authoring source).
    fn expanded_for_sync(&self) -> String {
        reciplexa::pipeline::expand(&self.source).unwrap_or_else(|_| self.source.clone())
    }

    /// Soft sync refusal: warn in yellow, keep authoring source, never sticky hard-error.
    fn soft_sync_refuse(&mut self, refuse: SyncRefuse) {
        self.sync_warning = Some(refuse.message);
        // Do not set `self.error` — preview stays usable.
    }

    fn clear_sync_warning(&mut self) {
        self.sync_warning = None;
    }

    /// Nudge selection in authoring source; soft-refuse when layers are synthetic.
    fn try_nudge_selection(&mut self, indices: &[usize], dx: f64, dy: f64) -> bool {
        if dx == 0.0 && dy == 0.0 {
            return false;
        }
        let expanded = self.expanded_for_sync();
        match nudge_authoring_layers(&self.source, &expanded, self.page_index, indices, dx, dy) {
            Ok(new_src) => {
                if new_src != self.source {
                    self.push_undo();
                    self.source = new_src;
                    self.clear_sync_warning();
                    self.error = if self.reload_ok() {
                        None
                    } else {
                        Some("edit produced invalid program".into())
                    };
                    self.rebuild_document_path();
                    true
                } else {
                    false
                }
            }
            Err(refuse) => {
                self.soft_sync_refuse(refuse);
                false
            }
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(self.source.clone());
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.source.clone());
            self.source = prev;
            self.drag = None;
            self.error = pipeline_doc(&self.source).err();
            self.rebuild_document_path();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.source.clone());
            self.source = next;
            self.drag = None;
            self.error = pipeline_doc(&self.source).err();
            self.rebuild_document_path();
        }
    }

    fn set_source_with_undo(&mut self, new_src: String) {
        if new_src != self.source {
            self.push_undo();
            self.source = new_src;
        }
    }

    fn select_layer(&mut self, index: usize, layers: &[LayerInfo]) {
        self.selected = vec![index];
        if self.document_path.enabled {
            self.document_path.select_layer(index);
        }
        if let Some(layer) = layers.get(index) {
            self.pending_source_select = Some((layer.byte_start, layer.byte_end));
        }
    }

    fn toggle_layer_in_selection(&mut self, index: usize, layers: &[LayerInfo]) {
        if let Some(pos) = self.selected.iter().position(|&i| i == index) {
            self.selected.remove(pos);
        } else {
            self.selected.push(index);
        }
        if self.document_path.enabled {
            self.document_path.toggle_layer(index);
        }
        if let Some(&last) = self.selected.last() {
            if let Some(layer) = layers.get(last) {
                self.pending_source_select = Some((layer.byte_start, layer.byte_end));
            }
        }
    }

    fn clear_selection(&mut self) {
        self.selected.clear();
        self.document_path.clear_selection();
        self.props_undo_open = false;
    }

    fn rebuild_document_path(&mut self) {
        if self.document_path.enabled {
            self.document_path.rebuild(&self.source);
            self.document_path.sync_selection_indices(&self.selected);
        }
        self.sync_gui_runtime();
    }

    /// Push layer identities into the Phase 8 runtime and reconcile.
    fn sync_gui_runtime(&mut self) {
        let layers: Vec<(StableNodeId, String)> = if !self.document_path.layer_to_node.is_empty() {
            self.document_path
                .layer_to_node
                .iter()
                .enumerate()
                .map(|(i, id)| (*id, format!("layer-{i}")))
                .collect()
        } else if let Ok(doc) = pipeline_doc(&self.source) {
            flatten_page(&doc, self.page_index)
                .map(|(_, shapes)| {
                    (0..shapes.len())
                        .map(|i| {
                            (
                                StableNodeId::new((i as u64).saturating_add(1)),
                                format!("layer-{i}"),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        self.gui_runtime.sync_layers(&layers);
        if let Some(&idx) = self.selected.last() {
            if let Some((id, _)) = layers.get(idx) {
                self.gui_runtime.select_node(*id);
            }
        } else {
            self.gui_runtime.selection.clear();
        }
        let known: Vec<StableNodeId> = layers.iter().map(|(id, _)| *id).collect();
        self.gui_runtime
            .prune_selection(&mut |id| known.contains(&id));
    }

    fn primary_selected(&self) -> Option<usize> {
        self.selected.last().copied()
    }

    fn selection_bounds_mm(&self) -> Option<(f64, f64, f64, f64)> {
        if self.selected.is_empty() {
            return None;
        }
        let doc = pipeline_doc(&self.source).ok()?;
        let (_, shapes) = flatten_page(&doc, self.page_index)?;
        let mut acc: Option<(f64, f64, f64, f64)> = None;
        for &i in &self.selected {
            let b = shapes.get(i).and_then(PaperLayout::shape_bounds_mm)?;
            acc = Some(match acc {
                None => b,
                Some((x0, y0, x1, y1)) => (x0.min(b.0), y0.min(b.1), x1.max(b.2), y1.max(b.3)),
            });
        }
        acc
    }

    /// Change zoom while keeping the paper point at `anchor_local` fixed on screen.
    fn set_viewport_zoom(&mut self, new_zoom: f32, anchor_local: egui::Vec2) {
        let new_zoom = new_zoom.clamp(0.2, 8.0);
        if (new_zoom - self.zoom).abs() < 1e-6 {
            return;
        }
        let (page_w, page_h) = self.viewport_page_mm;
        let base = PaperLayout::fit(
            self.viewport_paint_size.x,
            self.viewport_paint_size.y,
            24.0,
            page_w,
            page_h,
        );
        let (px, py) = PaperLayout::pan_for_zoom_change(
            &base,
            self.zoom,
            (self.pan.x, self.pan.y),
            new_zoom,
            (anchor_local.x, anchor_local.y),
        );
        self.zoom = new_zoom;
        self.pan = egui::vec2(px, py);
    }

    fn zoom_viewport_by(&mut self, factor: f32, anchor_local: Option<egui::Vec2>) {
        let anchor = anchor_local.unwrap_or_else(|| self.viewport_paint_size * 0.5);
        self.set_viewport_zoom(self.zoom * factor, anchor);
        self.viewport_prefs_dirty = true;
    }

    fn persist_gui_prefs(&self) {
        let mut prefs = GuiPrefs::load();
        prefs.theme = self.theme;
        prefs.show_grid = self.show_grid;
        prefs.zoom = self.zoom;
        prefs.pan_x = self.pan.x;
        prefs.pan_y = self.pan.y;
        prefs.show_source = self.show_source;
        prefs.show_layers = self.show_layers;
        prefs.show_preview = self.show_preview;
        prefs.show_props = self.props_open;
        prefs.float_source = self.float_source;
        prefs.float_layers = self.float_layers;
        prefs.float_preview = self.float_preview;
        prefs.float_props = self.float_props;
        prefs.save();
    }

    fn reset_viewport(&mut self) {
        self.zoom = 1.0;
        self.pan = egui::Vec2::ZERO;
        self.viewport_prefs_dirty = true;
    }

    fn frame_in_viewport(&mut self, bounds_mm: (f64, f64, f64, f64)) {
        let (page_w, page_h) = self.viewport_page_mm;
        let base = PaperLayout::fit(
            self.viewport_paint_size.x,
            self.viewport_paint_size.y,
            24.0,
            page_w,
            page_h,
        );
        let (zoom, px, py) = PaperLayout::viewport_fit_bounds(
            &base,
            (self.viewport_paint_size.x, self.viewport_paint_size.y),
            bounds_mm,
            32.0,
        );
        self.zoom = zoom;
        self.pan = egui::vec2(px, py);
        self.viewport_prefs_dirty = true;
    }

    fn apply_prop_edit(
        &mut self,
        flat: usize,
        id: &str,
        value: PropValue,
        prop_ctx: PropEditContext,
    ) {
        match set_layer_prop(&self.source, self.page_index, flat, id, &value, &prop_ctx) {
            Ok(new_src) => {
                if !self.props_undo_open {
                    self.push_undo();
                    self.props_undo_open = true;
                }
                self.source = new_src;
                self.error = pipeline_doc(&self.source).err();
                self.rebuild_document_path();
            }
            Err(e) => self.error = Some(e.message),
        }
    }

    fn paint_source_pane(&mut self, ui: &mut egui::Ui) {
        ui.label(
            "Edits reproject live; drag on the paper rewrites numbers here. Ctrl+S saves · Ctrl+O opens.",
        );
        ui.horizontal(|ui| {
            if ui.button("Save .rpx").clicked() {
                self.save_rpx();
            }
            if ui.button("Open…").clicked() {
                self.open_rpx_dialog(ui.ctx());
            }
            if ui.button("Reload disk").clicked() {
                // Load authoring text as-is; do not bake macros into the editor buffer.
                match fs::read_to_string(&self.path) {
                    Ok(raw) => {
                        self.source = raw;
                        self.drag = None;
                        self.clear_selection();
                        self.error = pipeline_doc(&self.source).err();
                        self.rebuild_document_path();
                    }
                    Err(e) => self.error = Some(format!("reload: {e}")),
                }
            }
            if ui.button("Export PDF").on_hover_text("Ctrl+E").clicked() {
                self.export_pdf();
            }
            if ui.button("Export SVG").clicked() {
                self.export_document(self.path.with_extension("svg"));
            }
            if ui.button("Export PPTX").clicked() {
                self.export_document(self.path.with_extension("pptx"));
            }
            if ui.button("Re-expand macros").clicked() {
                // Explicit bake: replaces the editor buffer with expanded forms.
                match expand_source(&self.source) {
                    Ok(s) => {
                        self.push_undo();
                        self.source = s;
                        self.error = pipeline_doc(&self.source).err();
                        self.rebuild_document_path();
                    }
                    Err(e) => self.error = Some(format!("macro: {}", e.message)),
                }
            }
        });
        ui.add_space(4.0);
        let pre_edit = self.source.clone();
        let editor = egui::TextEdit::multiline(&mut self.source)
            .id(egui::Id::new("rpx_source_editor"))
            .code_editor()
            .desired_width(f32::INFINITY)
            .desired_rows(36);
        let response = ui.add_sized(
            egui::vec2(
                ui.available_width(),
                (ui.available_height() - 48.0).max(120.0),
            ),
            editor,
        );
        if response.changed() {
            if !self.typing_undo_open {
                self.undo_stack.push(pre_edit);
                if self.undo_stack.len() > 100 {
                    self.undo_stack.remove(0);
                }
                self.redo_stack.clear();
                self.typing_undo_open = true;
            }
            self.drag = None;
            self.error = pipeline_doc(&self.source).err();
            self.sync_warning = None;
            self.rebuild_document_path();
        }
        if !response.has_focus() {
            self.typing_undo_open = false;
        }
        if let Some(err) = &self.error {
            ui.add_space(4.0);
            ui.colored_label(egui::Color32::RED, err);
        }
        if let Some(warn) = &self.sync_warning {
            ui.add_space(4.0);
            ui.colored_label(egui::Color32::from_rgb(180, 120, 0), warn);
        }
    }

    fn paint_layers_pane(&mut self, ui: &mut egui::Ui, layers: &[LayerInfo]) {
        ui.label("Drag rows vertically to restack. Top = front (later in source).");
        ui.separator();
        let n = layers.len();
        let mut reorder: Option<(usize, usize)> = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Top of list = topmost (last drawn = highest flatten index).
            for flat in (0..n).rev() {
                let Some(layer) = layers.get(flat) else {
                    continue;
                };
                let selected = self.selected.contains(&flat);
                let id = egui::Id::new(("layer_dnd", self.page_index, flat));
                let text = format!("{}. {}", flat + 1, layer.label);
                let being_dragged = ui.ctx().is_being_dragged(id);

                let row_resp = ui
                    .horizontal(|ui| {
                        ui.weak("⠿");
                        let label = if being_dragged {
                            ui.weak(&text)
                        } else {
                            ui.selectable_label(selected, &text)
                        };
                        let _ = label;
                    })
                    .response;
                let response = ui
                    .interact(row_resp.rect, id, egui::Sense::click_and_drag())
                    .on_hover_cursor(egui::CursorIcon::Grab)
                    .on_hover_text("Drag vertically to change stacking order");

                if response.dragged() {
                    response.dnd_set_drag_payload(flat);
                    // Ghost follows pointer Y only (stacking is 1D).
                    if let Some(pointer) = ui.ctx().pointer_interact_pos() {
                        let ghost = egui::Rect::from_min_size(
                            egui::pos2(
                                row_resp.rect.left(),
                                pointer.y - row_resp.rect.height() * 0.5,
                            ),
                            row_resp.rect.size(),
                        );
                        let painter = ui
                            .ctx()
                            .layer_painter(egui::LayerId::new(egui::Order::Tooltip, id));
                        painter.rect_filled(
                            ghost,
                            2.0,
                            egui::Color32::from_rgba_unmultiplied(30, 120, 220, 40),
                        );
                        painter.rect_stroke(
                            ghost,
                            2.0,
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(30, 120, 220)),
                            egui::StrokeKind::Outside,
                        );
                        painter.text(
                            ghost.left_center() + egui::vec2(8.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            &text,
                            egui::FontId::proportional(13.0),
                            egui::Color32::from_rgb(20, 60, 120),
                        );
                    }
                }

                if response.clicked() && !response.dragged() {
                    self.select_layer(flat, layers);
                }
                // Drop onto a row → take that stacking slot (rewrites .rpx).
                if let Some(pointer) = ui.ctx().pointer_interact_pos() {
                    if response.rect.contains(pointer)
                        && response.dnd_hover_payload::<usize>().is_some()
                    {
                        let insert_above = pointer.y < response.rect.center().y;
                        let y = if insert_above {
                            response.rect.top()
                        } else {
                            response.rect.bottom()
                        };
                        ui.painter().hline(
                            response.rect.x_range(),
                            y,
                            egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(30, 120, 220)),
                        );
                        if let Some(from) = response.dnd_release_payload::<usize>() {
                            let to = if insert_above {
                                (flat + 1).min(n.saturating_sub(1))
                            } else {
                                flat
                            };
                            if *from != to {
                                reorder = Some((*from, to));
                            }
                        }
                    }
                }
            }
            if n == 0 {
                ui.weak("(no shapes on this page)");
            }
        });
        if let Some((from, to)) = reorder {
            self.apply_layer_reorder(from, to);
        }
        if !self.selected.is_empty() {
            ui.separator();
            if self.selected.len() == 1 {
                ui.label(format!("Selected layer {}", self.selected[0] + 1));
            } else {
                ui.label(format!("{} layers selected", self.selected.len()));
            }
            if ui
                .button(if self.props_open {
                    "Hide properties"
                } else {
                    "Properties…"
                })
                .on_hover_text(
                    "Named arguments for the selection (size, position, color, text…). Not the attribute-button suite.",
                )
                .clicked()
                {
                    self.props_open = !self.props_open;
                    self.persist_gui_prefs();
                }
            if ui
                .button("Duplicate")
                .on_hover_text("Copy selection in .rpx (Ctrl+D)")
                .clicked()
            {
                self.duplicate_selection();
            }
            if ui
                .button("Delete")
                .on_hover_text("Remove from .rpx (Delete key)")
                .clicked()
            {
                self.delete_selection();
            }
        } else {
            self.props_undo_open = false;
        }
    }

    fn paint_preview_pane(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Scroll = zoom at cursor · Space/Middle/Alt-drag = pan · F = frame · Shift-drag = axis/aspect/15° · Ctrl-drag = 5mm snap.");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .checkbox(&mut self.document_path.enabled, "Document path")
                    .on_hover_text("Phase 4: keep StableNodeId selection alongside CST sync")
                    .changed()
                {
                    self.rebuild_document_path();
                }
                if self.document_path.enabled {
                    if let Some(id) = self.document_path.primary_node() {
                        ui.weak(format!("node {}", id.get()));
                    }
                }
            });
        });

        let doc = match pipeline_doc(&self.source) {
            Ok(d) => d,
            Err(e) => {
                ui.colored_label(egui::Color32::RED, e);
                ui.label("Keep typing in the source pane — the caret stays there.");
                return;
            }
        };
        if let Some(warn) = &self.sync_warning {
            ui.colored_label(egui::Color32::from_rgb(180, 120, 0), warn);
        }
        let page_count = doc.pages.len();
        if page_count == 0 {
            ui.colored_label(egui::Color32::RED, "Document has no pages.");
            return;
        }
        if self.page_index >= page_count {
            self.page_index = page_count - 1;
        }

        ui.horizontal(|ui| {
            if ui
                .add_enabled(self.page_index > 0, egui::Button::new("◀ Prev"))
                .clicked()
            {
                self.page_index -= 1;
                self.drag = None;
                self.clear_selection();
            }
            ui.label(format!("Page {} / {}", self.page_index + 1, page_count));
            if ui
                .add_enabled(
                    self.page_index + 1 < page_count,
                    egui::Button::new("Next ▶"),
                )
                .clicked()
            {
                self.page_index += 1;
                self.drag = None;
                self.clear_selection();
            }
            if ui
                .button("+ Page")
                .on_hover_text("Insert empty A4 page after current")
                .clicked()
            {
                self.add_page_after_current();
            }
            if ui
                .add_enabled(page_count > 1, egui::Button::new("− Page"))
                .on_hover_text("Delete current page")
                .clicked()
            {
                self.delete_current_page();
            }
            ui.separator();
            if ui.button("−").clicked() {
                self.zoom_viewport_by(1.0 / 1.15, None);
            }
            ui.label(format!("{:.0}%", self.zoom * 100.0));
            if ui.button("+").clicked() {
                self.zoom_viewport_by(1.15, None);
            }
            if ui.button("Reset view").clicked() {
                self.reset_viewport();
            }
            ui.separator();
            if ui
                .selectable_label(self.show_grid, "Grid")
                .on_hover_text("10mm grid (G)")
                .clicked()
            {
                self.show_grid = !self.show_grid;
                let mut prefs = GuiPrefs::load();
                prefs.show_grid = self.show_grid;
                prefs.save();
            }
        });

        let expanded = self.expanded_for_sync();
        let Some((page, shapes)) = flatten_page(&doc, self.page_index) else {
            ui.colored_label(egui::Color32::RED, "Page not found.");
            return;
        };
        let size_bindings = resolve_preview_size_targets(&expanded, self.page_index, shapes.len());
        let layers = resolve_preview_layers(&expanded, self.page_index, &shapes);
        if layers.len() != shapes.len() || size_bindings.len() != shapes.len() {
            ui.colored_label(
                egui::Color32::YELLOW,
                format!(
                    "size/layer/shape mismatch: {} / {} / {}",
                    size_bindings.len(),
                    layers.len(),
                    shapes.len()
                ),
            );
        }

        let avail = ui.available_size();
        self.viewport_paint_size = avail;
        self.viewport_page_mm = (page.paper.width_mm, page.paper.height_mm);

        let (response, painter) = ui.allocate_painter(avail, egui::Sense::click_and_drag());
        let rect = response.rect;

        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.0 {
                let factor = (1.0 + scroll * 0.002).clamp(0.85, 1.15);
                let anchor = ui
                    .input(|i| i.pointer.hover_pos())
                    .map(|p| p - rect.min)
                    .unwrap_or_else(|| avail * 0.5);
                self.zoom_viewport_by(factor, Some(anchor));
            }
        }
        let space_held = ui.input(|i| i.key_down(egui::Key::Space) && !i.modifiers.any());
        // Pan with Space+drag, middle mouse, or Alt + drag.
        let pan_gesture = response.dragged_by(egui::PointerButton::Middle)
            || (space_held && response.dragged_by(egui::PointerButton::Primary))
            || (response.dragged() && ui.input(|i| i.modifiers.alt));
        if pan_gesture {
            self.pan += response.drag_delta();
            self.viewport_prefs_dirty = true;
        }
        if space_held && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }

        let layout = PaperLayout::fit(
            rect.width(),
            rect.height(),
            24.0,
            page.paper.width_mm,
            page.paper.height_mm,
        )
        .with_view(self.zoom, self.pan.x, self.pan.y);

        let paper = egui::Rect::from_min_size(
            rect.min + egui::vec2(layout.origin_x_px, layout.origin_y_px),
            egui::vec2(layout.width_px, layout.height_px),
        );
        painter.rect_filled(paper, 0.0, egui::Color32::WHITE);
        if self.show_grid {
            paint_paper_grid(
                &painter,
                rect,
                &layout,
                page.paper.width_mm,
                page.paper.height_mm,
            );
        }
        let paper_stroke = match self.theme {
            UiTheme::Light => egui::Color32::from_gray(80),
            UiTheme::Dark => egui::Color32::from_gray(160),
        };
        painter.rect_stroke(
            paper,
            0.0,
            egui::Stroke::new(1.0_f32, paper_stroke),
            egui::StrokeKind::Outside,
        );

        for shape in &shapes {
            paint_shape(
                &painter,
                rect,
                &layout,
                shape,
                &mut self.textures,
                ui.ctx(),
                self.path.parent(),
            );
        }

        if let Some(pos) = response.hover_pos() {
            let local = pos - rect.min;
            let (hx, hy) = layout.px_to_mm(local.x, local.y);
            if let Some(hi) = hit_test_shapes(&shapes, hx, hy) {
                if !self.selected.contains(&hi) {
                    if let Some(shape) = shapes.get(hi) {
                        if let Some(bounds) = PaperLayout::shape_bounds_mm(shape) {
                            paint_hover_frame(&painter, rect, &layout, bounds);
                        }
                    }
                }
            }
        }

        for (i, shape) in shapes.iter().enumerate() {
            if self.selected.contains(&i) {
                if let Some(bounds) = PaperLayout::shape_bounds_mm(shape) {
                    paint_selection_frame(&painter, rect, &layout, bounds);
                }
                if let WorldShape::Path(p) = shape {
                    let is_line = p.points_mm.len() == 2
                        && size_bindings
                            .get(i)
                            .is_some_and(|t| matches!(t, SizeTarget::LineSeg(_)));
                    let is_poly = size_bindings.get(i).is_some_and(|t| {
                        matches!(
                            t,
                            SizeTarget::PolylinePoints(_) | SizeTarget::PolygonPoints(_)
                        )
                    });
                    if is_line || is_poly {
                        paint_line_endpoints(&painter, rect, &layout, &p.points_mm);
                    }
                }
            }
        }

        if let Some(pos) = response.hover_pos() {
            let local = pos - rect.min;
            let (mx, my) = layout.px_to_mm(local.x, local.y);
            if mx >= -1.0
                && my >= -1.0
                && mx <= page.paper.width_mm + 1.0
                && my <= page.paper.height_mm + 1.0
            {
                painter.text(
                    rect.left_bottom() + egui::vec2(8.0, -8.0),
                    egui::Align2::LEFT_BOTTOM,
                    format!("{mx:.1}, {my:.1} mm"),
                    egui::FontId::monospace(12.0),
                    egui::Color32::from_gray(110),
                );
            }
        }

        let skip_shape_drag = pan_gesture
            || ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle))
            || ui.input(|i| i.modifiers.alt)
            || (space_held && ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary)));

        if let Some(pos) = response.interact_pointer_pos() {
            let local = pos - rect.min;
            let local_pos = egui::pos2(local.x, local.y);
            let (mx, my) = layout.px_to_mm(local.x, local.y);

            if !skip_shape_drag && response.drag_started() {
                let mut started = false;
                let hit_body = hit_test_shapes(&shapes, mx, my);
                let shift = ui.input(|i| i.modifiers.shift);
                // Handles for primary selection (endpoints / scale / rotate).
                if let Some(sel) = self.primary_selected() {
                    if let Some(shape) = shapes.get(sel) {
                        // Path vertices work even when the stroke itself is hit.
                        if let WorldShape::Path(p) = shape {
                            if let Some(vertex) =
                                hit_line_endpoint(&layout, &p.points_mm, local_pos)
                            {
                                match size_bindings.get(sel).copied() {
                                    Some(SizeTarget::LineSeg(idx)) if p.points_mm.len() == 2 => {
                                        self.drag = Some(DragState {
                                            kind: DragKind::LineEndpoint {
                                                index: idx,
                                                endpoint: vertex,
                                                base_src: self.source.clone(),
                                            },
                                            undo_pushed: false,
                                        });
                                        started = true;
                                    }
                                    Some(SizeTarget::PolylinePoints(idx)) => {
                                        self.drag = Some(DragState {
                                            kind: DragKind::PolyVertex {
                                                head: "polyline",
                                                index: idx,
                                                vertex,
                                                base_src: self.source.clone(),
                                            },
                                            undo_pushed: false,
                                        });
                                        started = true;
                                    }
                                    Some(SizeTarget::PolygonPoints(idx)) => {
                                        self.drag = Some(DragState {
                                            kind: DragKind::PolyVertex {
                                                head: "polygon",
                                                index: idx,
                                                vertex,
                                                base_src: self.source.clone(),
                                            },
                                            undo_pushed: false,
                                        });
                                        started = true;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if !started && hit_body != Some(sel) {
                            if let Some(bounds) = PaperLayout::shape_bounds_mm(shape) {
                                let (x0, y0, x1, y1) = bounds;
                                let cx = (x0 + x1) * 0.5;
                                let cy = (y0 + y1) * 0.5;
                                if hit_rotate_handle(&layout, bounds, local_pos) {
                                    let start_angle_rad = (my - cy).atan2(mx - cx);
                                    let base_deg =
                                        layer_rotation_deg(&self.source, self.page_index, sel)
                                            .unwrap_or(0.0);
                                    self.drag = Some(DragState {
                                        kind: DragKind::Rotate {
                                            flat_index: sel,
                                            base_src: self.source.clone(),
                                            center_mm: (cx, cy),
                                            start_angle_rad,
                                            base_deg,
                                        },
                                        undo_pushed: false,
                                    });
                                    started = true;
                                } else if let Some(grab) =
                                    hit_scale_grab(&layout, bounds, local_pos)
                                {
                                    let allow = match grab {
                                        ScaleGrab::Corner(_) => true,
                                        ScaleGrab::Edge(_) => {
                                            size_bindings.get(sel).is_some_and(|t| {
                                                matches!(
                                                    t,
                                                    SizeTarget::TextSize(_)
                                                        | SizeTarget::RectWh(_)
                                                        | SizeTarget::FrameWh(_)
                                                        | SizeTarget::ImageWh(_)
                                                        | SizeTarget::EllipseRxRy(_)
                                                )
                                            })
                                        }
                                    };
                                    if allow {
                                        let start_dist = ((mx - cx).hypot(my - cy)).max(1e-6);
                                        if let Some(size) = size_bindings.get(sel).copied() {
                                            let box_drag = match size {
                                                SizeTarget::TextSize(_)
                                                | SizeTarget::RectWh(_)
                                                | SizeTarget::FrameWh(_)
                                                | SizeTarget::ImageWh(_)
                                                | SizeTarget::EllipseRxRy(_) => Some(BoxDrag {
                                                    grab,
                                                    start_bounds: bounds,
                                                }),
                                                _ => None,
                                            };
                                            self.drag = Some(DragState {
                                                kind: DragKind::Scale {
                                                    size,
                                                    flat_index: sel,
                                                    base_src: self.source.clone(),
                                                    center_mm: (cx, cy),
                                                    start_dist,
                                                    text_box: box_drag,
                                                },
                                                undo_pushed: false,
                                            });
                                            started = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !started {
                    if let Some(i) = hit_body {
                        if shift {
                            self.toggle_layer_in_selection(i, &layers);
                        } else if !self.selected.contains(&i) {
                            self.select_layer(i, &layers);
                        }
                        let flat_indices = if self.selected.contains(&i) {
                            self.selected.clone()
                        } else {
                            vec![i]
                        };
                        self.drag = Some(DragState {
                            kind: DragKind::Move {
                                last_mm: (mx, my),
                                flat_indices,
                            },
                            undo_pushed: false,
                        });
                    } else if !shift {
                        // Empty drag → marquee range select.
                        self.clear_selection();
                        self.drag = Some(DragState {
                            kind: DragKind::Marquee {
                                start_mm: (mx, my),
                                current_mm: (mx, my),
                            },
                            undo_pushed: false,
                        });
                    }
                }
            }

            if !skip_shape_drag && response.dragged() {
                if let Some(drag) = self.drag.clone() {
                    match drag.kind {
                        DragKind::Move {
                            last_mm,
                            flat_indices,
                        } => {
                            let mut dx = mx - last_mm.0;
                            let mut dy = my - last_mm.1;
                            // Shift: lock to dominant axis.
                            if ui.input(|i| i.modifiers.shift) {
                                if dx.abs() >= dy.abs() {
                                    dy = 0.0;
                                } else {
                                    dx = 0.0;
                                }
                            }
                            // Ctrl/Cmd: snap motion to a 5mm grid (paper coords).
                            if ui.input(|i| i.modifiers.command) {
                                const GRID: f64 = 5.0;
                                let snap = |v: f64| (v / GRID).round() * GRID;
                                dx = snap(mx) - snap(last_mm.0);
                                dy = snap(my) - snap(last_mm.1);
                                if ui.input(|i| i.modifiers.shift) {
                                    if dx.abs() >= dy.abs() {
                                        dy = 0.0;
                                    } else {
                                        dx = 0.0;
                                    }
                                }
                            }
                            let mut indices = flat_indices;
                            if dx.abs() > 1e-9 || dy.abs() > 1e-9 {
                                indices.sort_unstable();
                                let expanded = self.expanded_for_sync();
                                match nudge_authoring_layers(
                                    &self.source,
                                    &expanded,
                                    self.page_index,
                                    &indices,
                                    dx,
                                    dy,
                                ) {
                                    Ok(new_src) => {
                                        let mut undo_pushed = drag.undo_pushed;
                                        if new_src != self.source {
                                            if !undo_pushed {
                                                self.push_undo();
                                                undo_pushed = true;
                                            }
                                            self.source = new_src;
                                            self.clear_sync_warning();
                                            self.error = if self.reload_ok() {
                                                None
                                            } else {
                                                Some("edit produced invalid program".into())
                                            };
                                            self.rebuild_document_path();
                                        }
                                        self.drag = Some(DragState {
                                            kind: DragKind::Move {
                                                last_mm: (mx, my),
                                                flat_indices: indices,
                                            },
                                            undo_pushed,
                                        });
                                    }
                                    Err(refuse) => {
                                        self.soft_sync_refuse(refuse);
                                        // Keep last_mm so the gesture does not accumulate delta.
                                        self.drag = Some(DragState {
                                            kind: DragKind::Move {
                                                last_mm: (mx, my),
                                                flat_indices: indices,
                                            },
                                            undo_pushed: drag.undo_pushed,
                                        });
                                    }
                                }
                            }
                        }
                        DragKind::Marquee { start_mm, .. } => {
                            self.drag = Some(DragState {
                                kind: DragKind::Marquee {
                                    start_mm,
                                    current_mm: (mx, my),
                                },
                                undo_pushed: false,
                            });
                        }
                        DragKind::Scale {
                            size,
                            flat_index,
                            base_src,
                            center_mm,
                            start_dist,
                            text_box,
                        } => {
                            let scale_result = if let Some(tb) = text_box {
                                let (mut nx, mut ny, mut nw, mut nh) = box_from_grab(tb, mx, my);
                                if ui.input(|i| i.modifiers.shift) {
                                    (nx, ny, nw, nh) = apply_aspect_lock(tb, nx, ny, nw, nh);
                                }
                                if ui.input(|i| i.modifiers.command) {
                                    const GRID: f64 = 5.0;
                                    let x1 = snap_mm(nx + nw, GRID);
                                    let y1 = snap_mm(ny + nh, GRID);
                                    nx = snap_mm(nx, GRID);
                                    ny = snap_mm(ny, GRID);
                                    nw = (x1 - nx).max(0.5);
                                    nh = (y1 - ny).max(0.5);
                                }
                                match size {
                                    SizeTarget::TextSize(idx) => {
                                        set_text_box(&base_src, idx, nx, ny, nw, nh)
                                    }
                                    SizeTarget::RectWh(idx) => set_box_xywh(
                                        &base_src,
                                        "rect",
                                        idx,
                                        [1, 2, 3, 4],
                                        nx,
                                        ny,
                                        nw,
                                        nh,
                                    ),
                                    SizeTarget::FrameWh(idx) => set_box_xywh(
                                        &base_src,
                                        "frame",
                                        idx,
                                        [1, 2, 3, 4],
                                        nx,
                                        ny,
                                        nw,
                                        nh,
                                    ),
                                    SizeTarget::ImageWh(idx) => set_box_xywh(
                                        &base_src,
                                        "image",
                                        idx,
                                        [2, 3, 4, 5],
                                        nx,
                                        ny,
                                        nw,
                                        nh,
                                    ),
                                    SizeTarget::EllipseRxRy(idx) => {
                                        let cx = nx + nw * 0.5;
                                        let cy = ny + nh * 0.5;
                                        let rx = (nw * 0.5).max(0.25);
                                        let ry = (nh * 0.5).max(0.25);
                                        set_box_xywh(
                                            &base_src,
                                            "ellipse",
                                            idx,
                                            [1, 2, 3, 4],
                                            cx,
                                            cy,
                                            rx,
                                            ry,
                                        )
                                    }
                                    other => {
                                        let dist =
                                            (mx - center_mm.0).hypot(my - center_mm.1).max(1e-6);
                                        let factor = (dist / start_dist).clamp(0.05, 20.0);
                                        scale_size_target(&base_src, other, factor)
                                    }
                                }
                            } else {
                                let dist = (mx - center_mm.0).hypot(my - center_mm.1).max(1e-6);
                                let factor = (dist / start_dist).clamp(0.05, 20.0);
                                match size {
                                    SizeTarget::Unsupported => scale_layer_uniform(
                                        &base_src,
                                        self.page_index,
                                        flat_index,
                                        factor,
                                    ),
                                    size => scale_size_target(&base_src, size, factor),
                                }
                            };
                            match scale_result {
                                Ok(new_src) => {
                                    let kind = DragKind::Scale {
                                        size,
                                        flat_index,
                                        base_src,
                                        center_mm,
                                        start_dist,
                                        text_box,
                                    };
                                    let mut undo_pushed = drag.undo_pushed;
                                    if !undo_pushed {
                                        self.push_undo();
                                        undo_pushed = true;
                                    }
                                    self.source = new_src;
                                    self.clear_sync_warning();
                                    self.error = if self.reload_ok() {
                                        None
                                    } else {
                                        Some("edit produced invalid program".into())
                                    };
                                    self.rebuild_document_path();
                                    self.drag = Some(DragState { kind, undo_pushed });
                                }
                                Err(e) => self.soft_sync_refuse(SyncRefuse::new(format!(
                                    "canvas resize skipped: {} (authoring source unchanged)",
                                    e.message
                                ))),
                            }
                        }
                        DragKind::LineEndpoint {
                            index,
                            endpoint,
                            base_src,
                        } => {
                            let (mut sx, mut sy) = (mx, my);
                            if ui.input(|i| i.modifiers.command) {
                                const GRID: f64 = 5.0;
                                sx = snap_mm(sx, GRID);
                                sy = snap_mm(sy, GRID);
                            }
                            match set_line_endpoint(&base_src, index, endpoint, sx, sy) {
                                Ok(new_src) => {
                                    let kind = DragKind::LineEndpoint {
                                        index,
                                        endpoint,
                                        base_src,
                                    };
                                    let mut undo_pushed = drag.undo_pushed;
                                    if !undo_pushed {
                                        self.push_undo();
                                        undo_pushed = true;
                                    }
                                    self.source = new_src;
                                    self.clear_sync_warning();
                                    self.error = if self.reload_ok() {
                                        None
                                    } else {
                                        Some("edit produced invalid program".into())
                                    };
                                    self.rebuild_document_path();
                                    self.drag = Some(DragState { kind, undo_pushed });
                                }
                                Err(e) => self.soft_sync_refuse(SyncRefuse::new(format!(
                                    "canvas edit skipped: {} (authoring source unchanged)",
                                    e.message
                                ))),
                            }
                        }
                        DragKind::PolyVertex {
                            head,
                            index,
                            vertex,
                            base_src,
                        } => {
                            let (mut sx, mut sy) = (mx, my);
                            if ui.input(|i| i.modifiers.command) {
                                const GRID: f64 = 5.0;
                                sx = snap_mm(sx, GRID);
                                sy = snap_mm(sy, GRID);
                            }
                            match set_poly_vertex(&base_src, head, index, vertex, sx, sy) {
                                Ok(new_src) => {
                                    let kind = DragKind::PolyVertex {
                                        head,
                                        index,
                                        vertex,
                                        base_src,
                                    };
                                    let mut undo_pushed = drag.undo_pushed;
                                    if !undo_pushed {
                                        self.push_undo();
                                        undo_pushed = true;
                                    }
                                    self.source = new_src;
                                    self.clear_sync_warning();
                                    self.error = if self.reload_ok() {
                                        None
                                    } else {
                                        Some("edit produced invalid program".into())
                                    };
                                    self.rebuild_document_path();
                                    self.drag = Some(DragState { kind, undo_pushed });
                                }
                                Err(e) => self.soft_sync_refuse(SyncRefuse::new(format!(
                                    "canvas edit skipped: {} (authoring source unchanged)",
                                    e.message
                                ))),
                            }
                        }
                        DragKind::Rotate {
                            flat_index,
                            base_src,
                            center_mm,
                            start_angle_rad,
                            base_deg,
                        } => {
                            let angle = (my - center_mm.1).atan2(mx - center_mm.0);
                            let delta_deg = (angle - start_angle_rad).to_degrees();
                            let mut deg = base_deg + delta_deg;
                            if ui.input(|i| i.modifiers.shift) {
                                deg = (deg / 15.0).round() * 15.0;
                            }
                            match set_layer_rotation_deg(
                                &base_src,
                                self.page_index,
                                flat_index,
                                deg,
                                center_mm,
                            ) {
                                Ok(new_src) => {
                                    let kind = DragKind::Rotate {
                                        flat_index,
                                        base_src,
                                        center_mm,
                                        start_angle_rad,
                                        base_deg,
                                    };
                                    let mut undo_pushed = drag.undo_pushed;
                                    if !undo_pushed {
                                        self.push_undo();
                                        undo_pushed = true;
                                    }
                                    self.source = new_src;
                                    self.clear_sync_warning();
                                    self.error = if self.reload_ok() {
                                        None
                                    } else {
                                        Some("edit produced invalid program".into())
                                    };
                                    self.rebuild_document_path();
                                    self.drag = Some(DragState { kind, undo_pushed });
                                }
                                Err(e) => self.soft_sync_refuse(SyncRefuse::new(format!(
                                    "canvas rotate skipped: {} (authoring source unchanged)",
                                    e.message
                                ))),
                            }
                        }
                    }
                }
            }

            let mut suppress_click = false;
            if response.drag_stopped() {
                if let Some(DragState {
                    kind:
                        DragKind::Marquee {
                            start_mm,
                            current_mm,
                        },
                    ..
                }) = self.drag.take()
                {
                    suppress_click = true;
                    let aabb = (
                        start_mm.0.min(current_mm.0),
                        start_mm.1.min(current_mm.1),
                        start_mm.0.max(current_mm.0),
                        start_mm.1.max(current_mm.1),
                    );
                    let w = aabb.2 - aabb.0;
                    let h = aabb.3 - aabb.1;
                    if w > 0.5 || h > 0.5 {
                        let hits = shapes_intersecting_aabb(&shapes, aabb);
                        if self.document_path.enabled {
                            self.document_path.select_layers(&hits);
                        }
                        self.selected = hits;
                        if let Some(&last) = self.selected.last() {
                            if let Some(layer) = layers.get(last) {
                                self.pending_source_select =
                                    Some((layer.byte_start, layer.byte_end));
                            }
                        }
                    }
                }
            }

            if !skip_shape_drag && !suppress_click && response.clicked() {
                let shift = ui.input(|i| i.modifiers.shift);
                match hit_test_shapes(&shapes, mx, my) {
                    Some(i) if shift => self.toggle_layer_in_selection(i, &layers),
                    Some(i) => self.select_layer(i, &layers),
                    None if !shift => self.clear_selection(),
                    None => {}
                }
            }
        } else if response.drag_stopped() {
            self.drag = None;
        }

        // Draw active marquee rectangle.
        if let Some(DragState {
            kind:
                DragKind::Marquee {
                    start_mm,
                    current_mm,
                },
            ..
        }) = &self.drag
        {
            let (ax, ay) = layout.mm_to_px(start_mm.0, start_mm.1);
            let (bx, by) = layout.mm_to_px(current_mm.0, current_mm.1);
            let mrect = egui::Rect::from_two_pos(
                rect.min + egui::vec2(ax, ay),
                rect.min + egui::vec2(bx, by),
            );
            painter.rect_filled(
                mrect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(30, 120, 220, 40),
            );
            painter.rect_stroke(
                mrect,
                0.0,
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(30, 120, 220)),
                egui::StrokeKind::Outside,
            );
        }
    }

    fn paint_properties_pane(&mut self, ui: &mut egui::Ui) {
        if self.selected.is_empty() {
            ui.weak("Select a shape to edit properties.");
            return;
        }

        // Multi-select: batch fill / opacity (range ops).
        if self.selected.len() > 1 {
            let indices = self.selected.clone();
            let mut apply_fill = false;
            let mut apply_stroke = false;
            let mut apply_stroke_width = false;
            let mut apply_opacity = false;
            let mut align: Option<AlignEdge> = None;
            let mut distribute: Option<bool> = None; // Some(true)=H, Some(false)=V
            let mut bring_front = false;
            let mut send_back = false;
            let mut bring_forward = false;
            let mut send_backward = false;
            ui.label("Batch ops on the marquee / multi-selection.");
            ui.weak("Full per-object args appear for a single selection.");
            ui.separator();
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new("Fill color (all)").strong());
                for (i, lab) in ["fill.r", "fill.g", "fill.b"].iter().enumerate() {
                    ui.add(egui::Slider::new(&mut self.batch_fill[i], 0.0..=1.0).text(*lab));
                }
                let [r, g, b] = self.batch_fill;
                let swatch = egui::Color32::from_rgb(
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                );
                let (sw_resp, sw_painter) = ui
                    .allocate_painter(egui::vec2(ui.available_width(), 18.0), egui::Sense::hover());
                sw_painter.rect_filled(sw_resp.rect, 2.0, swatch);
                if ui.button("Apply fill to selection").clicked() {
                    apply_fill = true;
                }
            });
            ui.add_space(6.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new("Stroke color (all stroked)").strong());
                for (i, lab) in ["stroke.r", "stroke.g", "stroke.b"].iter().enumerate() {
                    ui.add(egui::Slider::new(&mut self.batch_stroke[i], 0.0..=1.0).text(*lab));
                }
                let [r, g, b] = self.batch_stroke;
                let swatch = egui::Color32::from_rgb(
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                );
                let (sw_resp, sw_painter) = ui
                    .allocate_painter(egui::vec2(ui.available_width(), 18.0), egui::Sense::hover());
                sw_painter.rect_filled(sw_resp.rect, 2.0, swatch);
                if ui.button("Apply stroke to selection").clicked() {
                    apply_stroke = true;
                }
            });
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Stroke width (all stroked)").strong());
            ui.add(
                egui::Slider::new(&mut self.batch_stroke_width, 0.2..=12.0).text("stroke.width"),
            );
            if ui.button("Apply stroke width").clicked() {
                apply_stroke_width = true;
            }
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Opacity (all)").strong());
            ui.add(egui::Slider::new(&mut self.batch_opacity, 0.0..=1.0).text("opacity"));
            if ui.button("Apply opacity to selection").clicked() {
                apply_opacity = true;
            }
            ui.separator();
            ui.label(egui::RichText::new("Align").strong());
            ui.horizontal(|ui| {
                if ui.button("Left").clicked() {
                    align = Some(AlignEdge::Left);
                }
                if ui.button("Right").clicked() {
                    align = Some(AlignEdge::Right);
                }
                if ui.button("Top").clicked() {
                    align = Some(AlignEdge::Top);
                }
                if ui.button("Bottom").clicked() {
                    align = Some(AlignEdge::Bottom);
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Center H").clicked() {
                    align = Some(AlignEdge::CenterH);
                }
                if ui.button("Center V").clicked() {
                    align = Some(AlignEdge::CenterV);
                }
            });
            if indices.len() >= 3 {
                ui.label(egui::RichText::new("Distribute").strong());
                ui.horizontal(|ui| {
                    if ui.button("Horizontal").clicked() {
                        distribute = Some(true);
                    }
                    if ui.button("Vertical").clicked() {
                        distribute = Some(false);
                    }
                });
            }
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Bring to front").clicked() {
                    bring_front = true;
                }
                if ui.button("Send to back").clicked() {
                    send_back = true;
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Forward").on_hover_text("Ctrl+Shift+]").clicked() {
                    bring_forward = true;
                }
                if ui
                    .button("Backward")
                    .on_hover_text("Ctrl+Shift+[")
                    .clicked()
                {
                    send_backward = true;
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Group").on_hover_text("Ctrl+G").clicked() {
                    self.group_selection();
                }
                if ui.button("Ungroup").on_hover_text("Ctrl+Shift+G").clicked() {
                    self.ungroup_selection();
                }
            });
            if apply_fill {
                let [r, g, b] = self.batch_fill;
                match set_layers_fill_rgb(&self.source, self.page_index, &indices, r, g, b) {
                    Ok(new_src) => {
                        self.set_source_with_undo(new_src);
                        self.error = pipeline_doc(&self.source).err();
                        self.rebuild_document_path();
                    }
                    Err(e) => self.error = Some(e.message),
                }
            }
            if apply_stroke {
                let [r, g, b] = self.batch_stroke;
                match set_layers_stroke_rgb(&self.source, self.page_index, &indices, r, g, b) {
                    Ok(new_src) => {
                        self.set_source_with_undo(new_src);
                        self.error = pipeline_doc(&self.source).err();
                        self.rebuild_document_path();
                    }
                    Err(e) => self.error = Some(e.message),
                }
            }
            if apply_stroke_width {
                match set_layers_stroke_width(
                    &self.source,
                    self.page_index,
                    &indices,
                    self.batch_stroke_width,
                ) {
                    Ok(new_src) => {
                        self.set_source_with_undo(new_src);
                        self.error = pipeline_doc(&self.source).err();
                        self.rebuild_document_path();
                    }
                    Err(e) => self.error = Some(e.message),
                }
            }
            if apply_opacity {
                match set_layers_opacity(
                    &self.source,
                    self.page_index,
                    &indices,
                    self.batch_opacity,
                ) {
                    Ok(new_src) => {
                        self.set_source_with_undo(new_src);
                        self.error = pipeline_doc(&self.source).err();
                        self.rebuild_document_path();
                    }
                    Err(e) => self.error = Some(e.message),
                }
            }
            if let Some(edge) = align {
                self.align_selection(edge);
            }
            if let Some(horizontal) = distribute {
                self.distribute_selection(horizontal);
            }
            if bring_front {
                self.bring_selection_to_front();
            }
            if send_back {
                self.send_selection_to_back();
            }
            if bring_forward {
                self.nudge_selection_z(1);
            }
            if send_backward {
                self.nudge_selection_z(-1);
            }
            return;
        }

        let Some(sel) = self.primary_selected() else {
            return;
        };
        let Ok(doc) = pipeline_doc(&self.source) else {
            return;
        };
        let Some(page) = doc.pages.get(self.page_index) else {
            return;
        };
        let Some((_, shapes)) = flatten_page(&doc, self.page_index) else {
            return;
        };
        let Some(aabb) = shapes.get(sel).and_then(PaperLayout::shape_bounds_mm) else {
            return;
        };
        let prop_ctx = PropEditContext {
            aabb_mm: aabb,
            paper_w_mm: page.paper.width_mm,
            paper_h_mm: page.paper.height_mm,
        };
        let props =
            collect_layer_props(&self.source, self.page_index, sel, &prop_ctx).unwrap_or_default();

        let mut edit: Option<(String, PropValue)> = None;
        let mut preset: Option<PropPreset> = None;
        let mut any_slider_down = false;
        let mut bring_front = false;
        let mut send_back = false;
        let mut bring_forward = false;
        let mut send_backward = false;

        ui.label("Named args for the selection (rewrites source).");
        ui.weak("Preset buttons below are built-in; macro-defined suites can replace them later.");
        ui.separator();
        for group in [
            PropGroup::Layout,
            PropGroup::Transform,
            PropGroup::Fill,
            PropGroup::Stroke,
            PropGroup::Content,
            PropGroup::Geometry,
        ] {
            let fields: Vec<_> = props.iter().filter(|p| p.group == group).collect();
            if fields.is_empty() {
                continue;
            }
            let framed = matches!(group, PropGroup::Fill | PropGroup::Stroke);
            let mut paint_group = |ui: &mut egui::Ui| {
                ui.label(egui::RichText::new(group.title()).strong());
                ui.horizontal_wrapped(|ui| match group {
                    PropGroup::Fill => {
                        for (label, rgb) in [
                            ("Black", [0.0, 0.0, 0.0]),
                            ("White", [1.0, 1.0, 1.0]),
                            ("Red", [0.85, 0.15, 0.15]),
                            ("Blue", [0.15, 0.35, 0.85]),
                        ] {
                            if ui.small_button(label).clicked() {
                                preset = Some(PropPreset::FillRgb(rgb));
                            }
                        }
                    }
                    PropGroup::Stroke => {
                        for (label, w) in [("0.5", 0.5), ("1", 1.0), ("2", 2.0)] {
                            if ui.small_button(format!("{label}mm")).clicked() {
                                preset = Some(PropPreset::StrokeWidth(w));
                            }
                        }
                    }
                    PropGroup::Transform => {
                        for (label, a) in [("100%", 1.0), ("50%", 0.5), ("25%", 0.25)] {
                            if ui.small_button(label).clicked() {
                                preset = Some(PropPreset::Opacity(a));
                            }
                        }
                    }
                    _ => {}
                });
                if matches!(group, PropGroup::Fill | PropGroup::Stroke) {
                    let prefix = if group == PropGroup::Fill {
                        "fill."
                    } else {
                        "stroke."
                    };
                    let mut rgb = [0.0_f64; 3];
                    let mut got = 0usize;
                    for (i, ch) in ["r", "g", "b"].iter().enumerate() {
                        let id = format!("{prefix}{ch}");
                        if let Some(PropValue::Number(v)) =
                            fields.iter().find(|f| f.id == id).map(|f| &f.value)
                        {
                            rgb[i] = *v;
                            got += 1;
                        }
                    }
                    if got == 3 {
                        let swatch = egui::Color32::from_rgb(
                            (rgb[0] * 255.0) as u8,
                            (rgb[1] * 255.0) as u8,
                            (rgb[2] * 255.0) as u8,
                        );
                        let (sw_resp, sw_painter) = ui.allocate_painter(
                            egui::vec2(ui.available_width().min(120.0), 16.0),
                            egui::Sense::hover(),
                        );
                        sw_painter.rect_filled(sw_resp.rect, 2.0, swatch);
                    }
                }
                for field in &fields {
                    match &field.value {
                        PropValue::Number(v) => {
                            let mut n = *v;
                            let resp = if let Some((lo, hi)) = field.slider {
                                ui.add(egui::Slider::new(&mut n, lo..=hi).text(&field.label))
                            } else {
                                ui.horizontal(|ui| {
                                    ui.label(&field.label);
                                    ui.add(egui::DragValue::new(&mut n).speed(0.1))
                                })
                                .inner
                            };
                            if resp.is_pointer_button_down_on() {
                                any_slider_down = true;
                            }
                            if resp.changed() {
                                edit = Some((field.id.clone(), PropValue::Number(n)));
                            }
                        }
                        PropValue::Text(t) => {
                            let mut s = t.clone();
                            ui.vertical(|ui| {
                                ui.label(&field.label);
                                let resp = if field.id == "content.text" {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut s)
                                            .desired_width(220.0)
                                            .desired_rows(3),
                                    )
                                } else {
                                    ui.add(egui::TextEdit::singleline(&mut s).desired_width(160.0))
                                };
                                if resp.changed() {
                                    edit = Some((field.id.clone(), PropValue::Text(s)));
                                }
                            });
                        }
                    }
                }
            };
            if framed {
                egui::Frame::group(ui.style()).show(ui, &mut paint_group);
            } else {
                paint_group(ui);
            }
            ui.add_space(4.0);
        }
        ui.separator();
        if !props.iter().any(|p| p.id == "geom.w") {
            if let Ok(sizes) = collect_size_targets_page(&self.source, self.page_index) {
                if let Some(SizeTarget::TextSize(idx)) = sizes.get(sel).copied() {
                    if ui
                            .button("Add layout box")
                            .on_hover_text(
                                "Insert w/h from the current selection bounds so edges can resize the box.",
                            )
                            .clicked()
                        {
                            let (x0, y0, x1, y1) = aabb;
                            match set_text_box(
                                &self.source,
                                idx,
                                x0,
                                y0,
                                (x1 - x0).max(0.5),
                                (y1 - y0).max(0.5),
                            ) {
                                Ok(new_src) => {
                                    self.push_undo();
                                    self.source = new_src;
                                    self.clear_sync_warning();
                                    self.error = pipeline_doc(&self.source).err();
                                    self.rebuild_document_path();
                                }
                                Err(e) => self.soft_sync_refuse(SyncRefuse::new(format!(
                                    "layout box skipped: {} (authoring source unchanged)",
                                    e.message
                                ))),
                            }
                        }
                }
            }
        }
        ui.horizontal(|ui| {
            if ui.button("Bring to front").clicked() {
                bring_front = true;
            }
            if ui.button("Send to back").clicked() {
                send_back = true;
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Forward").on_hover_text("Ctrl+Shift+]").clicked() {
                bring_forward = true;
            }
            if ui
                .button("Backward")
                .on_hover_text("Ctrl+Shift+[")
                .clicked()
            {
                send_backward = true;
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Group").on_hover_text("Ctrl+G").clicked() {
                self.group_selection();
            }
            if ui.button("Ungroup").on_hover_text("Ctrl+Shift+G").clicked() {
                self.ungroup_selection();
            }
        });
        if let Some((id, value)) = edit {
            self.apply_prop_edit(sel, &id, value, prop_ctx);
        }
        if let Some(p) = preset {
            let indices = [sel];
            let result = match p {
                PropPreset::FillRgb(rgb) => set_layers_fill_rgb(
                    &self.source,
                    self.page_index,
                    &indices,
                    rgb[0],
                    rgb[1],
                    rgb[2],
                ),
                PropPreset::StrokeWidth(w) => {
                    set_layers_stroke_width(&self.source, self.page_index, &indices, w)
                }
                PropPreset::Opacity(a) => {
                    set_layers_opacity(&self.source, self.page_index, &indices, a)
                }
            };
            match result {
                Ok(new_src) => {
                    self.push_undo();
                    self.source = new_src;
                    self.error = pipeline_doc(&self.source).err();
                    self.rebuild_document_path();
                }
                Err(e) => self.error = Some(e.message),
            }
        }
        if !any_slider_down {
            self.props_undo_open = false;
        }
        if bring_front {
            self.bring_selection_to_front();
        }
        if send_back {
            self.send_selection_to_back();
        }
        if bring_forward {
            self.nudge_selection_z(1);
        }
        if send_backward {
            self.nudge_selection_z(-1);
        }
    }

    fn apply_layer_reorder(&mut self, from: usize, to: usize) {
        if from == to {
            return;
        }
        match reorder_layer_page(&self.source, self.page_index, from, to) {
            Ok(new_src) => {
                self.set_source_with_undo(new_src);
                self.drag = None;
                self.selected = vec![to];
                self.error = pipeline_doc(&self.source).err();
                self.rebuild_document_path();
                if let Ok(layers) = collect_layers_page(&self.source, self.page_index) {
                    if let Some(layer) = layers.get(to) {
                        self.pending_source_select = Some((layer.byte_start, layer.byte_end));
                    }
                }
            }
            Err(e) => self.error = Some(e.message),
        }
    }

    /// Duplicate every selected layer (low→high), offsetting each copy, then select the copies.
    fn duplicate_selection(&mut self) {
        let mut indices = self.selected.clone();
        if indices.is_empty() {
            return;
        }
        indices.sort_unstable();
        indices.dedup();
        self.push_undo();
        let mut src = self.source.clone();
        let mut new_sel = Vec::with_capacity(indices.len());
        let mut offset = 0usize;
        for &i in &indices {
            let from = i + offset;
            match duplicate_layer_page(&src, self.page_index, from) {
                Ok(dup) => {
                    let copy_i = from + 1;
                    src = nudge_layer_page(&dup, self.page_index, copy_i, 5.0, -5.0).unwrap_or(dup);
                    new_sel.push(copy_i);
                    offset += 1;
                }
                Err(e) => {
                    self.error = Some(e.message);
                    return;
                }
            }
        }
        self.source = src;
        self.drag = None;
        self.selected = new_sel;
        self.error = pipeline_doc(&self.source).err();
        self.rebuild_document_path();
        if let Some(&last) = self.selected.last() {
            if let Ok(layers) = collect_layers_page(&self.source, self.page_index) {
                if let Some(layer) = layers.get(last) {
                    self.pending_source_select = Some((layer.byte_start, layer.byte_end));
                }
            }
        }
    }

    fn copy_selection_forms(&self) -> Option<String> {
        let layers = collect_layers_page(&self.source, self.page_index).ok()?;
        if self.selected.is_empty() {
            return None;
        }
        let mut indices = self.selected.clone();
        indices.sort_unstable();
        let mut forms = Vec::new();
        for i in indices {
            let layer = layers.get(i)?;
            if layer.byte_end <= self.source.len() && layer.byte_start <= layer.byte_end {
                forms.push(
                    self.source[layer.byte_start..layer.byte_end]
                        .trim()
                        .to_string(),
                );
            }
        }
        if forms.is_empty() {
            None
        } else {
            Some(forms.join("\n"))
        }
    }

    fn paste_forms(&mut self, text: &str) {
        let forms: Vec<&str> = text
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with('(') && l.ends_with(')'))
            .collect();
        if forms.is_empty() {
            return;
        }
        self.push_undo();
        let mut src = self.source.clone();
        let mut new_sel = Vec::new();
        for form in forms {
            match insert_layer_page(&src, self.page_index, form) {
                Ok((next, idx)) => {
                    // Offset pasted copies slightly so they are visible.
                    src = nudge_layer_page(&next, self.page_index, idx, 8.0, -8.0).unwrap_or(next);
                    new_sel.push(idx);
                }
                Err(e) => {
                    self.error = Some(e.message);
                    return;
                }
            }
        }
        self.source = src;
        self.selected = new_sel;
        self.error = pipeline_doc(&self.source).err();
        self.rebuild_document_path();
    }

    /// Insert a core shape form onto the current page and select it.
    fn insert_shape(&mut self, form: &str) {
        self.push_undo();
        match insert_layer_page(&self.source, self.page_index, form) {
            Ok((new_src, idx)) => {
                self.source = new_src;
                self.drag = None;
                // Place the new shape at the current viewport center.
                if let Some((vx, vy)) = self.viewport_center_mm() {
                    if let Ok(doc) = pipeline_doc(&self.source) {
                        if let Some((_, shapes)) = flatten_page(&doc, self.page_index) {
                            if let Some(b) = shapes.get(idx).and_then(PaperLayout::shape_bounds_mm)
                            {
                                let cx = (b.0 + b.2) * 0.5;
                                let cy = (b.1 + b.3) * 0.5;
                                let dx = vx - cx;
                                let dy = vy - cy;
                                if dx.abs() > 1e-6 || dy.abs() > 1e-6 {
                                    if let Ok(nudged) =
                                        nudge_layer_page(&self.source, self.page_index, idx, dx, dy)
                                    {
                                        self.source = nudged;
                                    }
                                }
                            }
                        }
                    }
                }
                self.selected = vec![idx];
                self.error = pipeline_doc(&self.source).err();
                self.rebuild_document_path();
                if let Ok(layers) = collect_layers_page(&self.source, self.page_index) {
                    if let Some(layer) = layers.get(idx) {
                        self.pending_source_select = Some((layer.byte_start, layer.byte_end));
                    }
                }
            }
            Err(e) => self.error = Some(e.message),
        }
    }

    fn viewport_center_mm(&self) -> Option<(f64, f64)> {
        let (page_w, page_h) = self.viewport_page_mm;
        if page_w <= 0.0 || page_h <= 0.0 {
            return None;
        }
        let avail = self.viewport_paint_size;
        if avail.x <= 1.0 || avail.y <= 1.0 {
            return None;
        }
        let base = PaperLayout::fit(avail.x, avail.y, 24.0, page_w, page_h);
        let layout = base.with_view(self.zoom, self.pan.x, self.pan.y);
        Some(layout.px_to_mm(avail.x * 0.5, avail.y * 0.5))
    }

    fn open_rpx_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        match fs::read_to_string(&path) {
            Ok(raw) => {
                self.path = path;
                self.source = raw;
                self.drag = None;
                self.clear_selection();
                self.page_index = 0;
                self.reset_viewport();
                self.textures.clear();
                self.undo_stack.clear();
                self.redo_stack.clear();
                self.typing_undo_open = false;
                self.props_undo_open = false;
                self.error = pipeline_doc(&self.source).err();
                self.rebuild_document_path();
                self.mark_saved();
                self.title_dirty = true; // force title refresh
                set_window_title(ctx, &self.path, false);
                self.title_dirty = false;
            }
            Err(e) => self.error = Some(format!("open: {e}")),
        }
    }

    fn confirm_discard_if_dirty(&self) -> bool {
        if !self.is_dirty() {
            return true;
        }
        let result = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Warning)
            .set_title("Unsaved changes")
            .set_description("Discard unsaved changes?")
            .set_buttons(rfd::MessageButtons::OkCancel)
            .show();
        matches!(result, rfd::MessageDialogResult::Ok)
    }

    fn open_rpx_dialog(&mut self, ctx: &egui::Context) {
        if !self.confirm_discard_if_dirty() {
            return;
        }
        let pick = rfd::FileDialog::new()
            .add_filter("reciplexa", &["rpx"])
            .add_filter("All", &["*"])
            .set_title("Open .rpx")
            .pick_file();
        if let Some(path) = pick {
            self.open_rpx_path(ctx, path);
        }
    }

    fn save_rpx(&mut self) {
        if let Err(e) = fs::write(&self.path, &self.source) {
            self.error = Some(format!("save: {e}"));
        } else {
            self.error = None;
            self.persist_codec_snapshot();
            self.mark_saved();
        }
    }

    /// Phase 9: write portable snapshot sidecar (never replaces recovery into primary silently).
    fn persist_codec_snapshot(&mut self) {
        match document_snapshot_from_source(&self.source, DocumentIdentity::new(1)) {
            Ok(snap) => {
                let paths = RecoveryPaths::for_primary(self.path.with_extension("rpxsnap"));
                if let Err(e) = write_journal(&paths, &snap) {
                    self.error = Some(format!("journal: {e:?}"));
                    return;
                }
                if let Err(e) = compact_after_save(&paths, &snap) {
                    self.error = Some(format!("snapshot: {e:?}"));
                }
            }
            Err(e) => {
                // Source may be temporarily invalid while typing; keep .rpx save.
                let _ = e;
            }
        }
    }

    fn save_rpx_as_dialog(&mut self, ctx: &egui::Context) {
        let pick = rfd::FileDialog::new()
            .add_filter("reciplexa", &["rpx"])
            .set_file_name(
                self.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("untitled.rpx"),
            )
            .set_title("Save .rpx As")
            .save_file();
        if let Some(mut path) = pick {
            if path.extension().is_none() {
                path.set_extension("rpx");
            }
            self.path = path;
            self.save_rpx();
            self.title_dirty = true;
            set_window_title(ctx, &self.path, false);
            self.title_dirty = false;
        }
    }

    fn insert_image_dialog(&mut self) {
        let pick = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg"])
            .set_title("Insert Image")
            .pick_file();
        let Some(asset) = pick else {
            return;
        };
        let rel = path_for_rpx(&self.path, &asset);
        let form = format!("(image {} 40 120 80 60)", escape_lisp_string(&rel));
        self.insert_shape(&form);
    }

    fn new_document(&mut self, ctx: &egui::Context) {
        if !self.confirm_discard_if_dirty() {
            return;
        }
        self.path = PathBuf::from("untitled.rpx");
        self.source =
            "(import graphics/page only a4 page)\n(val main (page a4 (list)))\n".to_string();
        self.drag = None;
        self.clear_selection();
        self.page_index = 0;
        self.reset_viewport();
        self.textures.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.typing_undo_open = false;
        self.props_undo_open = false;
        self.error = None;
        self.mark_saved();
        self.title_dirty = true;
        set_window_title(ctx, &self.path, false);
        self.title_dirty = false;
    }

    fn delete_selection(&mut self) {
        let mut indices = self.selected.clone();
        if indices.is_empty() {
            return;
        }
        indices.sort_unstable();
        indices.dedup();
        self.push_undo();
        // Delete high indices first so lower indices stay valid.
        for &i in indices.iter().rev() {
            match delete_layer_page(&self.source, self.page_index, i) {
                Ok(new_src) => self.source = new_src,
                Err(e) => {
                    self.error = Some(e.message);
                    return;
                }
            }
        }
        self.clear_selection();
        self.drag = None;
        self.error = pipeline_doc(&self.source).err();
        self.rebuild_document_path();
    }
    fn align_selection(&mut self, edge: AlignEdge) {
        let Ok(doc) = pipeline_doc(&self.source) else {
            return;
        };
        let Some((_, shapes)) = flatten_page(&doc, self.page_index) else {
            return;
        };
        let mut items: Vec<(usize, (f64, f64, f64, f64))> = self
            .selected
            .iter()
            .filter_map(|&i| {
                shapes
                    .get(i)
                    .and_then(PaperLayout::shape_bounds_mm)
                    .map(|b| (i, b))
            })
            .collect();
        if items.len() < 2 {
            return;
        }
        let target = match edge {
            AlignEdge::Left => items.iter().map(|(_, b)| b.0).fold(f64::INFINITY, f64::min),
            AlignEdge::Right => items
                .iter()
                .map(|(_, b)| b.2)
                .fold(f64::NEG_INFINITY, f64::max),
            AlignEdge::Bottom => items.iter().map(|(_, b)| b.1).fold(f64::INFINITY, f64::min),
            AlignEdge::Top => items
                .iter()
                .map(|(_, b)| b.3)
                .fold(f64::NEG_INFINITY, f64::max),
            AlignEdge::CenterH => {
                let min_x = items.iter().map(|(_, b)| b.0).fold(f64::INFINITY, f64::min);
                let max_x = items
                    .iter()
                    .map(|(_, b)| b.2)
                    .fold(f64::NEG_INFINITY, f64::max);
                (min_x + max_x) * 0.5
            }
            AlignEdge::CenterV => {
                let min_y = items.iter().map(|(_, b)| b.1).fold(f64::INFINITY, f64::min);
                let max_y = items
                    .iter()
                    .map(|(_, b)| b.3)
                    .fold(f64::NEG_INFINITY, f64::max);
                (min_y + max_y) * 0.5
            }
        };
        items.sort_by_key(|(i, _)| *i);
        let expanded = self.expanded_for_sync();
        let mut src = self.source.clone();
        let mut any = false;
        for &(i, (x0, y0, x1, y1)) in &items {
            let (dx, dy) = match edge {
                AlignEdge::Left => (target - x0, 0.0),
                AlignEdge::Right => (target - x1, 0.0),
                AlignEdge::Bottom => (0.0, target - y0),
                AlignEdge::Top => (0.0, target - y1),
                AlignEdge::CenterH => (target - (x0 + x1) * 0.5, 0.0),
                AlignEdge::CenterV => (0.0, target - (y0 + y1) * 0.5),
            };
            if dx.abs() < 1e-12 && dy.abs() < 1e-12 {
                continue;
            }
            match nudge_authoring_layers(&src, &expanded, self.page_index, &[i], dx, dy) {
                Ok(new_src) => {
                    src = new_src;
                    any = true;
                }
                Err(refuse) => {
                    self.soft_sync_refuse(refuse);
                    return;
                }
            }
        }
        if any {
            self.push_undo();
            self.source = src;
            self.clear_sync_warning();
            self.error = pipeline_doc(&self.source).err();
            self.rebuild_document_path();
        }
    }

    fn distribute_selection(&mut self, horizontal: bool) {
        let Ok(doc) = pipeline_doc(&self.source) else {
            return;
        };
        let Some((_, shapes)) = flatten_page(&doc, self.page_index) else {
            return;
        };
        let mut items: Vec<(usize, f64)> = self
            .selected
            .iter()
            .filter_map(|&i| {
                let b = shapes.get(i).and_then(PaperLayout::shape_bounds_mm)?;
                let center = if horizontal {
                    (b.0 + b.2) * 0.5
                } else {
                    (b.1 + b.3) * 0.5
                };
                Some((i, center))
            })
            .collect();
        if items.len() < 3 {
            return;
        }
        items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let first = items[0].1;
        let last = items[items.len() - 1].1;
        let n = items.len();
        let step = (last - first) / (n - 1) as f64;
        let expanded = self.expanded_for_sync();
        let mut src = self.source.clone();
        let mut any = false;
        for (k, &(i, center)) in items.iter().enumerate() {
            let want = first + step * k as f64;
            let delta = want - center;
            if delta.abs() < 1e-12 {
                continue;
            }
            let (dx, dy) = if horizontal {
                (delta, 0.0)
            } else {
                (0.0, delta)
            };
            match nudge_authoring_layers(&src, &expanded, self.page_index, &[i], dx, dy) {
                Ok(new_src) => {
                    src = new_src;
                    any = true;
                }
                Err(refuse) => {
                    self.soft_sync_refuse(refuse);
                    return;
                }
            }
        }
        if any {
            self.push_undo();
            self.source = src;
            self.clear_sync_warning();
            self.error = pipeline_doc(&self.source).err();
            self.rebuild_document_path();
        }
    }

    fn bring_selection_to_front(&mut self) {
        let mut remaining = self.selected.clone();
        remaining.sort_unstable();
        remaining.dedup();
        if remaining.is_empty() {
            return;
        }
        let count = remaining.len();
        self.push_undo();
        let mut src = self.source.clone();
        while let Some(&from) = remaining.first() {
            let n = match collect_layers_page(&src, self.page_index) {
                Ok(l) => l.len(),
                Err(e) => {
                    self.error = Some(e.message);
                    return;
                }
            };
            if from >= n {
                break;
            }
            match reorder_layer_page(&src, self.page_index, from, n - 1) {
                Ok(new_src) => src = new_src,
                Err(e) => {
                    self.error = Some(e.message);
                    return;
                }
            }
            remaining.remove(0);
            for r in &mut remaining {
                if *r > from {
                    *r -= 1;
                }
            }
        }
        let n = collect_layers_page(&src, self.page_index)
            .map(|l| l.len())
            .unwrap_or(0);
        self.source = src;
        if n >= count {
            self.selected = ((n - count)..n).collect();
        }
        self.error = pipeline_doc(&self.source).err();
        self.rebuild_document_path();
    }

    fn send_selection_to_back(&mut self) {
        let mut remaining = self.selected.clone();
        remaining.sort_unstable();
        remaining.dedup();
        if remaining.is_empty() {
            return;
        }
        let count = remaining.len();
        self.push_undo();
        let mut src = self.source.clone();
        // Move highest first down to the growing back stack.
        while let Some(&from) = remaining.last() {
            let slot = remaining.len() - 1; // destination among back slots
            if from != slot {
                match reorder_layer_page(&src, self.page_index, from, slot) {
                    Ok(new_src) => src = new_src,
                    Err(e) => {
                        self.error = Some(e.message);
                        return;
                    }
                }
                // Update remaining after moving from → slot (from > slot).
                for r in &mut remaining {
                    if *r == from {
                        *r = slot;
                    } else if *r >= slot && *r < from {
                        *r += 1;
                    }
                }
            }
            remaining.pop();
        }
        self.source = src;
        self.selected = (0..count).collect();
        self.error = pipeline_doc(&self.source).err();
        self.rebuild_document_path();
    }

    /// Move selection one step toward front (`delta > 0`) or back (`delta < 0`).
    fn nudge_selection_z(&mut self, delta: isize) {
        if delta == 0 {
            return;
        }
        let mut sel = self.selected.clone();
        sel.sort_unstable();
        sel.dedup();
        if sel.is_empty() {
            return;
        }
        self.push_undo();
        let mut src = self.source.clone();
        let order: Vec<usize> = if delta > 0 {
            (0..sel.len()).rev().collect()
        } else {
            (0..sel.len()).collect()
        };
        for i in order {
            let from = sel[i];
            let n = match collect_layers_page(&src, self.page_index) {
                Ok(l) => l.len(),
                Err(e) => {
                    self.error = Some(e.message);
                    return;
                }
            };
            if n == 0 {
                break;
            }
            let to = (from as isize + delta).clamp(0, (n - 1) as isize) as usize;
            if to == from {
                continue;
            }
            // Avoid hopping over another still-selected member in the same step.
            if sel.contains(&to) {
                continue;
            }
            match reorder_layer_page(&src, self.page_index, from, to) {
                Ok(new_src) => src = new_src,
                Err(e) => {
                    self.error = Some(e.message);
                    return;
                }
            }
            for (j, s) in sel.iter_mut().enumerate() {
                if j == i {
                    *s = to;
                    continue;
                }
                if from < to {
                    if *s > from && *s <= to {
                        *s -= 1;
                    }
                } else if *s >= to && *s < from {
                    *s += 1;
                }
            }
        }
        self.source = src;
        self.selected = sel;
        self.error = pipeline_doc(&self.source).err();
        self.rebuild_document_path();
    }

    fn group_selection(&mut self) {
        if self.selected.len() < 2 {
            self.error = Some("select at least two contiguous layers to group".into());
            return;
        }
        self.push_undo();
        match group_layers_page(&self.source, self.page_index, &self.selected) {
            Ok((new_src, sel)) => {
                self.source = new_src;
                self.selected = sel;
                self.error = pipeline_doc(&self.source).err();
                self.rebuild_document_path();
            }
            Err(e) => {
                let _ = self.undo_stack.pop();
                self.error = Some(e.message);
            }
        }
    }

    fn ungroup_selection(&mut self) {
        let Some(flat) = self.primary_selected() else {
            return;
        };
        self.push_undo();
        match ungroup_layer_page(&self.source, self.page_index, flat) {
            Ok((new_src, sel)) => {
                self.source = new_src;
                self.selected = sel;
                self.error = pipeline_doc(&self.source).err();
                self.rebuild_document_path();
            }
            Err(e) => {
                let _ = self.undo_stack.pop();
                self.error = Some(e.message);
            }
        }
    }

    fn add_page_after_current(&mut self) {
        self.push_undo();
        match insert_page_after(&self.source, Some(self.page_index), "(page a4)") {
            Ok((new_src, idx)) => {
                self.source = new_src;
                self.page_index = idx;
                self.drag = None;
                self.clear_selection();
                self.error = pipeline_doc(&self.source).err();
                self.rebuild_document_path();
            }
            Err(e) => {
                let _ = self.undo_stack.pop();
                self.error = Some(e.message);
            }
        }
    }

    fn delete_current_page(&mut self) {
        self.push_undo();
        match delete_page(&self.source, self.page_index) {
            Ok(new_src) => {
                self.source = new_src;
                if self.page_index > 0 {
                    self.page_index -= 1;
                }
                self.drag = None;
                self.clear_selection();
                self.error = pipeline_doc(&self.source).err();
                self.rebuild_document_path();
            }
            Err(e) => {
                let _ = self.undo_stack.pop();
                self.error = Some(e.message);
            }
        }
    }

    fn is_dirty(&self) -> bool {
        self.source != self.saved_source
    }

    fn refresh_window_title(&mut self, ctx: &egui::Context) {
        let dirty = self.is_dirty();
        if dirty != self.title_dirty {
            self.title_dirty = dirty;
            set_window_title(ctx, &self.path, dirty);
        }
    }

    fn mark_saved(&mut self) {
        self.saved_source = self.source.clone();
    }

    fn export_pdf(&mut self) {
        self.export_document(self.path.with_extension("pdf"));
    }

    fn export_document(&mut self, out_path: PathBuf) {
        match document_for_export(&mut GuiExportHandler::default(), &self.source) {
            Ok((doc, _)) => {
                let base = self.path.parent();
                let result = fs::File::create(&out_path)
                    .map_err(|e| e.to_string())
                    .and_then(|mut f| {
                        match out_path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("pdf")
                            .to_ascii_lowercase()
                            .as_str()
                        {
                            "svg" => reciplexa_svg::write_document(&doc, &mut f)
                                .map_err(|e| e.to_string()),
                            "pptx" => reciplexa_pptx::write_document(&doc, &mut f)
                                .map_err(|e| e.to_string()),
                            _ => write_document_with_base(&doc, base, &mut f)
                                .map_err(|e| format!("{e:?}")),
                        }
                    });
                match result {
                    Ok(()) => self.error = None,
                    Err(e) => {
                        self.error = Some(format!("{}: {e}", out_path.display()));
                    }
                }
            }
            Err(e) => self.error = Some(e.display()),
        }
    }

    fn set_theme(&mut self, ctx: &egui::Context, theme: UiTheme) {
        if self.theme == theme {
            return;
        }
        self.theme = theme;
        theme.save_prefs();
        apply_ui_theme(ctx, theme);
    }

    fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.set_theme(ctx, self.theme.toggle());
        self.persist_gui_prefs();
    }
}

#[derive(Clone, Copy)]
enum AlignEdge {
    Left,
    Right,
    Top,
    Bottom,
    CenterH,
    CenterV,
}

fn pane_chrome(
    ui: &mut egui::Ui,
    title: &str,
    floating: &mut bool,
    visible: &mut bool,
    collapsed: &mut bool,
) {
    ui.horizontal(|ui| {
        if !title.is_empty() {
            ui.label(egui::RichText::new(title).strong());
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("×").on_hover_text("Hide pane").clicked() {
                *visible = false;
            }
            let float_tip = if *floating {
                "Dock into layout"
            } else {
                "Float as window"
            };
            if ui.small_button("□").on_hover_text(float_tip).clicked() {
                *floating = !*floating;
            }
            let collapse_tip = if *collapsed { "Expand" } else { "Collapse" };
            if ui.small_button("–").on_hover_text(collapse_tip).clicked() {
                *collapsed = !*collapsed;
            }
        });
    });
    ui.separator();
}

impl eframe::App for PreviewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        suppress_ime_confirm_newline(ctx, &mut self.ime_enter_hold);
        self.refresh_window_title(ctx);
        self.sync_gui_runtime();

        // Ctrl+Z / Ctrl+Y for source undo/redo (IME mistakes, layer moves, etc.).
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift) {
            self.undo();
        }
        if ctx.input(|i| {
            (i.modifiers.command && i.key_pressed(egui::Key::Y))
                || (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z))
        }) {
            self.redo();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            if ctx.input(|i| i.modifiers.shift) {
                self.save_rpx_as_dialog(ctx);
            } else {
                self.save_rpx();
            }
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::E)) {
            self.export_pdf();
        }

        // Arrow keys nudge the selection when the source editor is not focused.
        let source_focused = ctx.memory(|m| m.has_focus(egui::Id::new("rpx_source_editor")));
        if !source_focused && ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
            self.open_rpx_dialog(ctx);
        }
        if !source_focused && ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::N)) {
            self.new_document(ctx);
        }
        // Ctrl+Shift+L — toggle light / dark chrome (paper stays white).
        if ctx.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::L)) {
            self.toggle_theme(ctx);
        }
        if !source_focused && ctx.input(|i| i.key_pressed(egui::Key::G) && !i.modifiers.any()) {
            self.show_grid = !self.show_grid;
            let mut prefs = GuiPrefs::load();
            prefs.show_grid = self.show_grid;
            prefs.save();
        }
        if !source_focused
            && ctx
                .input(|i| i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::G))
        {
            self.group_selection();
        }
        if !source_focused
            && ctx
                .input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::G))
        {
            self.ungroup_selection();
        }
        if !source_focused
            && ctx.input(|i| {
                i.key_pressed(egui::Key::Equals)
                    || i.key_pressed(egui::Key::Plus)
                    || (i.modifiers.command && i.key_pressed(egui::Key::Equals))
            })
        {
            self.zoom_viewport_by(1.15, None);
        }
        if !source_focused
            && ctx.input(|i| {
                i.key_pressed(egui::Key::Minus)
                    || (i.modifiers.command && i.key_pressed(egui::Key::Minus))
            })
        {
            self.zoom_viewport_by(1.0 / 1.15, None);
        }
        if !source_focused && ctx.input(|i| i.key_pressed(egui::Key::Num0) && i.modifiers.command) {
            self.reset_viewport();
        }
        if !source_focused && ctx.input(|i| i.key_pressed(egui::Key::PageUp)) && self.page_index > 0
        {
            self.page_index -= 1;
            self.drag = None;
            self.clear_selection();
        }
        if !source_focused && ctx.input(|i| i.key_pressed(egui::Key::PageDown)) {
            if let Ok(doc) = pipeline_doc(&self.source) {
                if self.page_index + 1 < doc.pages.len() {
                    self.page_index += 1;
                    self.drag = None;
                    self.clear_selection();
                }
            }
        }
        if !source_focused && ctx.input(|i| i.key_pressed(egui::Key::F) && !i.modifiers.any()) {
            if let Some(bounds) = self.selection_bounds_mm() {
                self.frame_in_viewport(bounds);
            } else {
                let (page_w, page_h) = self.viewport_page_mm;
                self.frame_in_viewport((0.0, 0.0, page_w, page_h));
            }
        }
        if !source_focused && ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::A)) {
            let expanded = self.expanded_for_sync();
            if let Ok(doc) = pipeline_doc(&self.source) {
                if let Some((_, shapes)) = flatten_page(&doc, self.page_index) {
                    let layers = resolve_preview_layers(&expanded, self.page_index, &shapes);
                    self.selected = (0..layers.len()).collect();
                }
            }
        }
        if !source_focused && !self.selected.is_empty() {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.clear_selection();
            }
            let step = if ctx.input(|i| i.modifiers.shift) {
                5.0
            } else if ctx.input(|i| i.modifiers.alt) {
                0.1
            } else {
                1.0
            };
            let mut delta = (0.0_f64, 0.0_f64);
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                delta.0 -= step;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                delta.0 += step;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                delta.1 -= step;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                delta.1 += step;
            }
            if delta != (0.0, 0.0) {
                let mut indices = self.selected.clone();
                indices.sort_unstable();
                self.try_nudge_selection(&indices, delta.0, delta.1);
            }
            if ctx
                .input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace))
            {
                self.delete_selection();
            }
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::D)) {
                self.duplicate_selection();
            }
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::C)) {
                if let Some(forms) = self.copy_selection_forms() {
                    ctx.copy_text(forms);
                }
            }
            if ctx.input(|i| {
                i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::CloseBracket)
            }) {
                self.bring_selection_to_front();
            }
            if ctx.input(|i| {
                i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::OpenBracket)
            }) {
                self.send_selection_to_back();
            }
            if ctx.input(|i| {
                i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::CloseBracket)
            }) {
                self.nudge_selection_z(1);
            }
            if ctx.input(|i| {
                i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::OpenBracket)
            }) {
                self.nudge_selection_z(-1);
            }
        }
        if !source_focused && self.selected.is_empty() {
            let step = if ctx.input(|i| i.modifiers.shift) {
                48.0
            } else {
                16.0
            };
            let mut delta = egui::Vec2::ZERO;
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                delta.x += step;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                delta.x -= step;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                delta.y -= step;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                delta.y += step;
            }
            if delta != egui::Vec2::ZERO {
                self.pan += delta;
                self.viewport_prefs_dirty = true;
            }
        }
        if !source_focused {
            let pasted: Vec<String> = ctx.input(|i| {
                i.events
                    .iter()
                    .filter_map(|e| match e {
                        egui::Event::Paste(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect()
            });
            for s in pasted {
                self.paste_forms(&s);
            }
        }

        if let Some((start, end)) = self.pending_source_select.take() {
            highlight_source_range(ctx, &self.source, start, end);
        }

        egui::TopBottomPanel::top("insert_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("File").strong());
                if ui.button("New").clicked() {
                    self.new_document(ctx);
                }
                if ui.button("Open…").clicked() {
                    self.open_rpx_dialog(ctx);
                }
                if ui.button("Save").clicked() {
                    self.save_rpx();
                }
                if ui.button("Save As…").clicked() {
                    self.save_rpx_as_dialog(ctx);
                }
                if ui.button("Export PDF").on_hover_text("Ctrl+E").clicked() {
                    self.export_pdf();
                }
                if ui.button("Export SVG").clicked() {
                    self.export_document(self.path.with_extension("svg"));
                }
                if ui.button("Export PPTX").clicked() {
                    self.export_document(self.path.with_extension("pptx"));
                }
                ui.separator();
                ui.label(egui::RichText::new("Insert").strong());
                if ui.button("Circle").clicked() {
                    self.insert_shape("(circle 105 148.5 20 (rgb 0.85 0.3 0.35))");
                }
                if ui.button("Rect").clicked() {
                    self.insert_shape("(rect 60 120 90 60 (rgb 0.25 0.55 0.9))");
                }
                if ui.button("Ellipse").clicked() {
                    self.insert_shape("(ellipse 105 148.5 40 25 (rgb 0.3 0.72 0.45))");
                }
                if ui.button("Frame").clicked() {
                    self.insert_shape("(frame 50 100 110 80 1.5 (rgb 0.15 0.35 0.75))");
                }
                if ui.button("Line").clicked() {
                    self.insert_shape("(line 40 200 170 200 (rgb 0.9 0.35 0.2) 1.2)");
                }
                if ui.button("Polyline").clicked() {
                    self.insert_shape(
                        "(polyline 40 180 80 220 120 190 160 210 (rgb 0.2 0.55 0.85) 1.2)",
                    );
                }
                if ui.button("Polygon").clicked() {
                    self.insert_shape("(polygon 60 140 100 180 140 140 (rgb 0.85 0.55 0.2))");
                }
                if ui.button("Text").clicked() {
                    self.insert_shape("(text 40 200 12 60 24 \"Text\" (rgb 0.15 0.15 0.2))");
                }
                if ui.button("Ring").clicked() {
                    self.insert_shape("(ring 105 148.5 30 8 (rgb 0.95 0.55 0.15))");
                }
                if ui.button("Image…").clicked() {
                    self.insert_image_dialog();
                }
                ui.separator();
                ui.label(egui::RichText::new("View").strong());
                let theme_label = format!("Theme: {}", self.theme.label());
                if ui
                    .button(theme_label)
                    .on_hover_text("Toggle light / dark UI (Ctrl+Shift+L). Paper stays white.")
                    .clicked()
                {
                    self.toggle_theme(ctx);
                }
                if ui
                    .selectable_label(self.show_source, "Source")
                    .on_hover_text("Show/hide Source pane (docked by default; □ floats)")
                    .clicked()
                {
                    self.show_source = !self.show_source;
                    self.persist_gui_prefs();
                }
                if ui
                    .selectable_label(self.show_preview, "Preview")
                    .on_hover_text("Show/hide Preview pane (docked by default; □ floats)")
                    .clicked()
                {
                    self.show_preview = !self.show_preview;
                    self.persist_gui_prefs();
                }
                if ui
                    .selectable_label(self.show_layers, "Layers")
                    .on_hover_text("Show/hide Layers pane (docked by default; □ floats)")
                    .clicked()
                {
                    self.show_layers = !self.show_layers;
                    self.persist_gui_prefs();
                }
                if ui
                    .selectable_label(self.props_open, "Properties")
                    .on_hover_text("Show/hide Properties pane (docked by default; □ floats)")
                    .clicked()
                {
                    self.props_open = !self.props_open;
                    self.persist_gui_prefs();
                }
                if ui
                    .button("Reset view")
                    .on_hover_text("Reset zoom and pan")
                    .clicked()
                {
                    self.reset_viewport();
                }
                ui.label(format!("{:.0}%", self.zoom * 100.0));
                let grid_label = if self.show_grid {
                    "Grid: On"
                } else {
                    "Grid: Off"
                };
                if ui
                    .button(grid_label)
                    .on_hover_text("Toggle 10mm paper grid")
                    .clicked()
                {
                    self.show_grid = !self.show_grid;
                    let mut prefs = GuiPrefs::load();
                    prefs.show_grid = self.show_grid;
                    prefs.save();
                }
            });
        });

        let layers_for_panel = {
            let expanded = self.expanded_for_sync();
            match pipeline_doc(&self.source) {
                Ok(doc) => flatten_page(&doc, self.page_index)
                    .map(|(_, shapes)| resolve_preview_layers(&expanded, self.page_index, &shapes))
                    .unwrap_or_default(),
                Err(_) => collect_layers_page(&expanded, self.page_index).unwrap_or_default(),
            }
        };

        // --- Docked bento panes ---
        if self.show_source && !self.float_source {
            egui::SidePanel::left("dock_source")
                .resizable(true)
                .default_width(360.0)
                .width_range(220.0..=720.0)
                .show(ctx, |ui| {
                    let mut floating = self.float_source;
                    let mut visible = self.show_source;
                    let mut collapsed = self.collapse_source;
                    pane_chrome(
                        ui,
                        ".rpx source",
                        &mut floating,
                        &mut visible,
                        &mut collapsed,
                    );
                    let changed = floating != self.float_source
                        || visible != self.show_source
                        || collapsed != self.collapse_source;
                    self.float_source = floating;
                    self.show_source = visible;
                    self.collapse_source = collapsed;
                    if changed {
                        self.persist_gui_prefs();
                    }
                    if self.show_source && !self.float_source && !self.collapse_source {
                        self.paint_source_pane(ui);
                    }
                });
        }

        let dock_layers = self.show_layers && !self.float_layers;
        let dock_props = self.props_open && !self.float_props;
        if dock_layers || dock_props {
            egui::SidePanel::right("dock_right")
                .resizable(true)
                .default_width(280.0)
                .width_range(180.0..=480.0)
                .show(ctx, |ui| {
                    if dock_layers {
                        let height = if dock_props {
                            (ui.available_height() * 0.55).max(120.0)
                        } else {
                            ui.available_height()
                        };
                        egui::ScrollArea::vertical()
                            .id_salt("dock_layers_scroll")
                            .max_height(height)
                            .show(ui, |ui| {
                                let mut floating = self.float_layers;
                                let mut visible = self.show_layers;
                                let mut collapsed = self.collapse_layers;
                                pane_chrome(
                                    ui,
                                    "Layers",
                                    &mut floating,
                                    &mut visible,
                                    &mut collapsed,
                                );
                                let changed = floating != self.float_layers
                                    || visible != self.show_layers
                                    || collapsed != self.collapse_layers;
                                self.float_layers = floating;
                                self.show_layers = visible;
                                self.collapse_layers = collapsed;
                                if changed {
                                    self.persist_gui_prefs();
                                }
                                if self.show_layers && !self.float_layers && !self.collapse_layers {
                                    self.paint_layers_pane(ui, &layers_for_panel);
                                }
                            });
                        if dock_props {
                            ui.separator();
                        }
                    }
                    if dock_props {
                        let mut floating = self.float_props;
                        let mut visible = self.props_open;
                        let mut collapsed = self.collapse_props;
                        let title = if self.selected.len() > 1 {
                            format!("Properties ({} selected)", self.selected.len())
                        } else {
                            "Properties".to_string()
                        };
                        pane_chrome(ui, &title, &mut floating, &mut visible, &mut collapsed);
                        let changed = floating != self.float_props
                            || visible != self.props_open
                            || collapsed != self.collapse_props;
                        self.float_props = floating;
                        self.props_open = visible;
                        self.collapse_props = collapsed;
                        if changed {
                            self.persist_gui_prefs();
                        }
                        if self.props_open && !self.float_props && !self.collapse_props {
                            egui::ScrollArea::vertical()
                                .id_salt("dock_props_scroll")
                                .show(ui, |ui| {
                                    self.paint_properties_pane(ui);
                                });
                        }
                    }
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.show_preview && !self.float_preview {
                let mut floating = self.float_preview;
                let mut visible = self.show_preview;
                let mut collapsed = self.collapse_preview;
                pane_chrome(
                    ui,
                    "Paper preview",
                    &mut floating,
                    &mut visible,
                    &mut collapsed,
                );
                let changed = floating != self.float_preview
                    || visible != self.show_preview
                    || collapsed != self.collapse_preview;
                self.float_preview = floating;
                self.show_preview = visible;
                self.collapse_preview = collapsed;
                if changed {
                    self.persist_gui_prefs();
                }
                if self.show_preview && !self.float_preview && !self.collapse_preview {
                    self.paint_preview_pane(ui);
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.weak(
                        "Preview is hidden or floating — restore it from View, or dock with □.",
                    );
                });
            }
        });

        // --- Floating panes ---
        if self.show_source && self.float_source {
            let mut floating = self.float_source;
            let mut visible = self.show_source;
            let mut collapsed = self.collapse_source;
            egui::Window::new(".rpx source")
                .id(egui::Id::new("floating_source"))
                .default_pos([12.0, 56.0])
                .default_size([420.0, 560.0])
                .min_width(280.0)
                .min_height(200.0)
                .resizable(true)
                .collapsible(false)
                .constrain(true)
                .show(ctx, |ui| {
                    pane_chrome(ui, "", &mut floating, &mut visible, &mut collapsed);
                    if !collapsed {
                        self.paint_source_pane(ui);
                    }
                });
            let changed = floating != self.float_source
                || visible != self.show_source
                || collapsed != self.collapse_source;
            self.float_source = floating;
            self.show_source = visible;
            self.collapse_source = collapsed;
            if changed {
                self.persist_gui_prefs();
            }
        }

        if self.show_layers && self.float_layers {
            let mut floating = self.float_layers;
            let mut visible = self.show_layers;
            let mut collapsed = self.collapse_layers;
            egui::Window::new("Layers")
                .id(egui::Id::new("floating_layers"))
                .default_pos([920.0, 56.0])
                .default_size([260.0, 420.0])
                .min_width(180.0)
                .min_height(160.0)
                .resizable(true)
                .collapsible(false)
                .constrain(true)
                .show(ctx, |ui| {
                    pane_chrome(ui, "", &mut floating, &mut visible, &mut collapsed);
                    if !collapsed {
                        self.paint_layers_pane(ui, &layers_for_panel);
                    }
                });
            let changed = floating != self.float_layers
                || visible != self.show_layers
                || collapsed != self.collapse_layers;
            self.float_layers = floating;
            self.show_layers = visible;
            self.collapse_layers = collapsed;
            if changed {
                self.persist_gui_prefs();
            }
        }

        if self.props_open && self.float_props {
            let mut floating = self.float_props;
            let mut visible = self.props_open;
            let mut collapsed = self.collapse_props;
            let title = if self.selected.len() > 1 {
                format!("Properties ({} selected)", self.selected.len())
            } else {
                "Properties".to_string()
            };
            egui::Window::new(title)
                .id(egui::Id::new("floating_props"))
                .default_pos([920.0, 480.0])
                .default_size([280.0, 360.0])
                .min_width(200.0)
                .min_height(160.0)
                .resizable(true)
                .collapsible(false)
                .constrain(true)
                .show(ctx, |ui| {
                    pane_chrome(ui, "", &mut floating, &mut visible, &mut collapsed);
                    if !collapsed {
                        self.paint_properties_pane(ui);
                    }
                });
            let changed = floating != self.float_props
                || visible != self.props_open
                || collapsed != self.collapse_props;
            self.float_props = floating;
            self.props_open = visible;
            self.collapse_props = collapsed;
            if changed {
                self.persist_gui_prefs();
            }
        }

        if self.show_preview && self.float_preview {
            let mut floating = self.float_preview;
            let mut visible = self.show_preview;
            let mut collapsed = self.collapse_preview;
            egui::Window::new("Paper preview")
                .id(egui::Id::new("floating_preview"))
                .default_pos([440.0, 56.0])
                .default_size([480.0, 640.0])
                .min_width(320.0)
                .min_height(280.0)
                .resizable(true)
                .collapsible(false)
                .constrain(true)
                .show(ctx, |ui| {
                    pane_chrome(ui, "", &mut floating, &mut visible, &mut collapsed);
                    if !collapsed {
                        self.paint_preview_pane(ui);
                    }
                });
            let changed = floating != self.float_preview
                || visible != self.show_preview
                || collapsed != self.collapse_preview;
            self.float_preview = floating;
            self.show_preview = visible;
            self.collapse_preview = collapsed;
            if changed {
                self.persist_gui_prefs();
            }
        }

        if self.viewport_prefs_dirty {
            self.persist_gui_prefs();
            self.viewport_prefs_dirty = false;
        }
    }
}
