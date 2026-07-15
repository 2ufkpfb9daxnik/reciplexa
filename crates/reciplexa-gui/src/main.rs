//! Paper preview with drag → CST sync, plus a live `.rpx` source pane.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use eframe::egui;
use eframe::egui::text::{CCursor, CCursorRange};
use reciplexa::pipeline::{document_for_export, document_from_source};
use reciplexa_effect::{seed_from_env, EffectError, EffectHandler, LcgRng, Value};
use reciplexa_lower::{
    collect_layer_props, collect_layers_page, collect_size_targets_page, delete_layer_page,
    duplicate_layer_page, insert_layer_page, layer_rotation_deg, nudge_layer_page,
    reorder_layer_page, scale_size_target, set_box_xywh, set_layer_prop, set_layer_rotation_deg,
    set_layers_fill_rgb, set_layers_opacity, set_layers_stroke_rgb, set_layers_stroke_width,
    set_text_box, LayerInfo, PropEditContext, PropGroup, PropValue, SizeTarget,
};
use reciplexa_macro::expand_source;
use reciplexa_pdf::write_document_with_base;
use reciplexa_view::{
    flatten_page, hit_test_shapes, shapes_intersecting_aabb, text_corners_mm, PaperLayout,
    WorldShape,
};

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
        eprintln!("[perform log] {message}");
        Ok(Value::Unit)
    }

    fn on_random(&mut self) -> Result<Value, EffectError> {
        Ok(Value::Number(self.rng.next_unit()))
    }

    fn on_write_path(&mut self, path: &str) -> Result<Value, EffectError> {
        eprintln!("[perform write-path] {path}");
        Ok(Value::Unit)
    }
}

fn install_cjk_fonts(ctx: &egui::Context) {
    let windir = PathBuf::from(env::var_os("WINDIR").unwrap_or_else(|| r"C:\Windows".into()));
    let fonts_dir = windir.join("Fonts");
    // Prefer static TTF/TTC. Variable fonts often break egui glyph metrics, which
    // makes IME preedit look like a solid black block over the whole TextEdit.
    let candidates = [
        fonts_dir.join("YuGothR.ttc"),
        fonts_dir.join("BIZ-UDGothicR.ttc"),
        fonts_dir.join("meiryo.ttc"),
        fonts_dir.join("msgothic.ttc"),
        fonts_dir.join("NotoSans-Regular.ttf"),
        // VF last — usable for PDF exports elsewhere, risky as the primary GUI face.
        fonts_dir.join("NotoSansJP-VF.ttf"),
        fonts_dir.join("NotoSansJP-VariableFont_wght.ttf"),
    ];
    let Some(path) = candidates.into_iter().find(|p| p.is_file()) else {
        eprintln!("warn: no CJK-capable TTF/TTC found for GUI preview");
        return;
    };
    let Ok(bytes) = fs::read(&path) else {
        eprintln!("warn: could not read font {}", path.display());
        return;
    };
    let mut fonts = egui::FontDefinitions::default();
    // Keep egui's bundled Latin faces first; CJK is a fallback for both families
    // so monospace / code editors keep stable metrics for ASCII.
    fonts.font_data.insert(
        "reciplexa_cjk".into(),
        Arc::new(egui::FontData::from_owned(bytes)),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("reciplexa_cjk".into());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("reciplexa_cjk".into());
    ctx.set_fonts(fonts);
    eprintln!("gui font: {}", path.display());
}

/// Light / dark chrome. Paper itself stays light (print preview).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiTheme {
    Light,
    Dark,
}

impl UiTheme {
    fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    fn from_prefs() -> Self {
        let Some(path) = theme_prefs_path() else {
            return Self::Light;
        };
        match fs::read_to_string(path) {
            Ok(s) if s.trim().eq_ignore_ascii_case("dark") => Self::Dark,
            _ => Self::Light,
        }
    }

    fn save_prefs(self) {
        let Some(path) = theme_prefs_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let body = match self {
            Self::Light => "light\n",
            Self::Dark => "dark\n",
        };
        let _ = fs::write(path, body);
    }
}

fn theme_prefs_path() -> Option<PathBuf> {
    let base = env::var_os("LOCALAPPDATA")
        .or_else(|| env::var_os("XDG_CONFIG_HOME"))
        .or_else(|| env::var_os("HOME"))?;
    Some(PathBuf::from(base).join("reciplexa").join("theme"))
}

/// Editor visuals tuned so IME preedit is not a near-black slab.
fn apply_ui_theme(ctx: &egui::Context, theme: UiTheme) {
    let mut visuals = match theme {
        UiTheme::Light => egui::Visuals::light(),
        UiTheme::Dark => egui::Visuals::dark(),
    };
    visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(60, 140, 230, 90);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(90, 160, 230));
    visuals.extreme_bg_color = match theme {
        UiTheme::Light => egui::Color32::from_gray(250),
        // Slightly above panel so TextEdit / source pane stay readable under IME.
        UiTheme::Dark => egui::Color32::from_rgb(36, 36, 40),
    };
    ctx.set_visuals(visuals);
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
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(c),
        }
    }
    out
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

fn set_window_title(ctx: &egui::Context, path: &Path) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
        "reciplexa — {}",
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
        None => (PathBuf::from("untitled.rpx"), "(page a4)\n".to_string()),
    };
    // Keep the author's `.rpx` text. Expansion happens inside `pipeline_doc` /
    // export so Scribble macros (`@title`, …) stay editable.
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
            let theme = UiTheme::from_prefs();
            apply_ui_theme(&cc.egui_ctx, theme);
            Ok(Box::new(PreviewApp {
                path,
                source: src,
                error: initial_error,
                drag: None,
                page_index: 0,
                zoom: 1.0,
                pan: egui::Vec2::ZERO,
                selected: Vec::new(),
                pending_source_select: None,
                textures: std::collections::HashMap::new(),
                ime_enter_hold: 0,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
                typing_undo_open: false,
                props_open: false,
                props_undo_open: false,
                batch_fill: [0.2, 0.2, 0.2],
                batch_stroke: [0.1, 0.1, 0.1],
                batch_stroke_width: 1.0,
                batch_opacity: 1.0,
                theme,
                show_grid: false,
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
    error: Option<String>,
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
}

#[derive(Clone, Copy)]
enum ScaleCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy)]
enum ScaleEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy)]
enum ScaleGrab {
    Corner(ScaleCorner),
    Edge(ScaleEdge),
}

#[derive(Clone, Copy)]
struct BoxDrag {
    grab: ScaleGrab,
    /// Page AABB at drag start: (x0, y0, x1, y1) with y-up.
    start_bounds: (f64, f64, f64, f64),
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
        base_src: String,
        center_mm: (f64, f64),
        start_dist: f64,
        text_box: Option<BoxDrag>,
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
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.source.clone());
            self.source = next;
            self.drag = None;
            self.error = pipeline_doc(&self.source).err();
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
        self.props_open = true;
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
        self.props_open = !self.selected.is_empty();
        if let Some(&last) = self.selected.last() {
            if let Some(layer) = layers.get(last) {
                self.pending_source_select = Some((layer.byte_start, layer.byte_end));
            }
        }
    }

    fn clear_selection(&mut self) {
        self.selected.clear();
        self.props_undo_open = false;
    }

    fn primary_selected(&self) -> Option<usize> {
        self.selected.last().copied()
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
            }
            Err(e) => self.error = Some(e.message),
        }
    }

    fn show_properties_window(&mut self, ctx: &egui::Context) {
        if !self.props_open || self.selected.is_empty() {
            return;
        }

        // Multi-select: batch fill / opacity (range ops).
        if self.selected.len() > 1 {
            let indices = self.selected.clone();
            let mut open = self.props_open;
            let mut apply_fill = false;
            let mut apply_stroke = false;
            let mut apply_stroke_width = false;
            let mut apply_opacity = false;
            let mut align: Option<AlignEdge> = None;
            let mut distribute: Option<bool> = None; // Some(true)=H, Some(false)=V
            let mut bring_front = false;
            let mut send_back = false;
            egui::Window::new(format!("Properties ({} selected)", indices.len()))
                .id(egui::Id::new("selection_properties_multi"))
                .open(&mut open)
                .anchor(egui::Align2::RIGHT_BOTTOM, [-12.0, -12.0])
                .default_width(280.0)
                .show(ctx, |ui| {
                    ui.label("Batch ops on the marquee / multi-selection.");
                    ui.weak("Full per-object args appear for a single selection.");
                    ui.separator();
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.label(egui::RichText::new("Fill color (all)").strong());
                        for (i, lab) in ["fill.r", "fill.g", "fill.b"].iter().enumerate() {
                            ui.add(
                                egui::Slider::new(&mut self.batch_fill[i], 0.0..=1.0).text(*lab),
                            );
                        }
                        let [r, g, b] = self.batch_fill;
                        let swatch = egui::Color32::from_rgb(
                            (r * 255.0) as u8,
                            (g * 255.0) as u8,
                            (b * 255.0) as u8,
                        );
                        let (sw_resp, sw_painter) = ui.allocate_painter(
                            egui::vec2(ui.available_width(), 18.0),
                            egui::Sense::hover(),
                        );
                        sw_painter.rect_filled(sw_resp.rect, 2.0, swatch);
                        if ui.button("Apply fill to selection").clicked() {
                            apply_fill = true;
                        }
                    });
                    ui.add_space(6.0);
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.label(egui::RichText::new("Stroke color (all stroked)").strong());
                        for (i, lab) in ["stroke.r", "stroke.g", "stroke.b"].iter().enumerate() {
                            ui.add(
                                egui::Slider::new(&mut self.batch_stroke[i], 0.0..=1.0).text(*lab),
                            );
                        }
                        let [r, g, b] = self.batch_stroke;
                        let swatch = egui::Color32::from_rgb(
                            (r * 255.0) as u8,
                            (g * 255.0) as u8,
                            (b * 255.0) as u8,
                        );
                        let (sw_resp, sw_painter) = ui.allocate_painter(
                            egui::vec2(ui.available_width(), 18.0),
                            egui::Sense::hover(),
                        );
                        sw_painter.rect_filled(sw_resp.rect, 2.0, swatch);
                        if ui.button("Apply stroke to selection").clicked() {
                            apply_stroke = true;
                        }
                    });
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Stroke width (all stroked)").strong());
                    ui.add(
                        egui::Slider::new(&mut self.batch_stroke_width, 0.2..=12.0)
                            .text("stroke.width"),
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
                });
            self.props_open = open;
            if apply_fill {
                let [r, g, b] = self.batch_fill;
                match set_layers_fill_rgb(&self.source, self.page_index, &indices, r, g, b) {
                    Ok(new_src) => {
                        self.set_source_with_undo(new_src);
                        self.error = pipeline_doc(&self.source).err();
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

        let mut open = self.props_open;
        let mut edit: Option<(String, PropValue)> = None;
        let mut any_slider_down = false;
        let mut bring_front = false;
        let mut send_back = false;

        egui::Window::new("Properties")
            .id(egui::Id::new("selection_properties"))
            .open(&mut open)
            .anchor(egui::Align2::RIGHT_BOTTOM, [-12.0, -12.0])
            .default_width(280.0)
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                ui.label("Named args for the selection (rewrites source).");
                ui.weak("Attribute buttons come later via macros.");
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
                                        ui.add(
                                            egui::Slider::new(&mut n, lo..=hi).text(&field.label),
                                        )
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
                                            ui.add(
                                                egui::TextEdit::singleline(&mut s)
                                                    .desired_width(160.0),
                                            )
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
                ui.horizontal(|ui| {
                    if ui.button("Bring to front").clicked() {
                        bring_front = true;
                    }
                    if ui.button("Send to back").clicked() {
                        send_back = true;
                    }
                });
            });

        self.props_open = open;
        if let Some((id, value)) = edit {
            self.apply_prop_edit(sel, &id, value, prop_ctx);
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
        self.props_open = !self.selected.is_empty();
        self.error = pipeline_doc(&self.source).err();
        if let Some(&last) = self.selected.last() {
            if let Ok(layers) = collect_layers_page(&self.source, self.page_index) {
                if let Some(layer) = layers.get(last) {
                    self.pending_source_select = Some((layer.byte_start, layer.byte_end));
                }
            }
        }
    }

    /// Insert a core shape form onto the current page and select it.
    fn insert_shape(&mut self, form: &str) {
        self.push_undo();
        match insert_layer_page(&self.source, self.page_index, form) {
            Ok((new_src, idx)) => {
                self.source = new_src;
                self.drag = None;
                self.selected = vec![idx];
                self.props_open = true;
                self.error = pipeline_doc(&self.source).err();
                if let Ok(layers) = collect_layers_page(&self.source, self.page_index) {
                    if let Some(layer) = layers.get(idx) {
                        self.pending_source_select = Some((layer.byte_start, layer.byte_end));
                    }
                }
            }
            Err(e) => self.error = Some(e.message),
        }
    }

    fn open_rpx_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        match fs::read_to_string(&path) {
            Ok(raw) => {
                self.path = path;
                self.source = raw;
                self.drag = None;
                self.clear_selection();
                self.page_index = 0;
                self.zoom = 1.0;
                self.pan = egui::Vec2::ZERO;
                self.textures.clear();
                self.undo_stack.clear();
                self.redo_stack.clear();
                self.typing_undo_open = false;
                self.props_undo_open = false;
                self.error = pipeline_doc(&self.source).err();
                set_window_title(ctx, &self.path);
            }
            Err(e) => self.error = Some(format!("open: {e}")),
        }
    }

    fn open_rpx_dialog(&mut self, ctx: &egui::Context) {
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
            set_window_title(ctx, &self.path);
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
        let form = format!("(image \"{}\" 40 120 80 60)", escape_lisp_string(&rel));
        self.insert_shape(&form);
    }

    fn new_document(&mut self, ctx: &egui::Context) {
        self.path = PathBuf::from("untitled.rpx");
        self.source = "(page a4)\n".to_string();
        self.drag = None;
        self.clear_selection();
        self.page_index = 0;
        self.zoom = 1.0;
        self.pan = egui::Vec2::ZERO;
        self.textures.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.typing_undo_open = false;
        self.props_undo_open = false;
        self.error = None;
        set_window_title(ctx, &self.path);
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
        self.push_undo();
        let mut src = self.source.clone();
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
            match nudge_layer_page(&src, self.page_index, i, dx, dy) {
                Ok(new_src) => src = new_src,
                Err(e) => {
                    self.error = Some(e.message);
                    return;
                }
            }
        }
        self.source = src;
        self.error = pipeline_doc(&self.source).err();
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
        self.push_undo();
        let mut src = self.source.clone();
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
            match nudge_layer_page(&src, self.page_index, i, dx, dy) {
                Ok(new_src) => src = new_src,
                Err(e) => {
                    self.error = Some(e.message);
                    return;
                }
            }
        }
        self.source = src;
        self.error = pipeline_doc(&self.source).err();
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

impl eframe::App for PreviewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        suppress_ime_confirm_newline(ctx, &mut self.ime_enter_hold);

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
            self.save_rpx();
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
        }
        if !source_focused && ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::A)) {
            if let Ok(layers) = collect_layers_page(&self.source, self.page_index) {
                self.selected = (0..layers.len()).collect();
                self.props_open = !self.selected.is_empty();
            }
        }
        if !source_focused && !self.selected.is_empty() {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.clear_selection();
                self.props_open = false;
            }
            let step = if ctx.input(|i| i.modifiers.shift) {
                5.0
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
                self.push_undo();
                let mut src = self.source.clone();
                for &sel in &indices {
                    match nudge_layer_page(&src, self.page_index, sel, delta.0, delta.1) {
                        Ok(new_src) => src = new_src,
                        Err(e) => {
                            self.error = Some(e.message);
                            break;
                        }
                    }
                }
                self.source = src;
                self.error = pipeline_doc(&self.source).err();
            }
            if ctx
                .input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace))
            {
                self.delete_selection();
            }
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::D)) {
                self.duplicate_selection();
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
                    .button("Reset view")
                    .on_hover_text("Reset zoom and pan")
                    .clicked()
                {
                    self.zoom = 1.0;
                    self.pan = egui::Vec2::ZERO;
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
                }
            });
        });

        egui::SidePanel::left("source_panel")
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading(".rpx source");
                ui.label(
                    "Edits reproject live; drag on the paper rewrites numbers here. Ctrl+S saves · Ctrl+O opens.",
                );
                ui.horizontal(|ui| {
                    if ui.button("Save .rpx").clicked() {
                        self.save_rpx();
                    }
                    if ui.button("Open…").clicked() {
                        self.open_rpx_dialog(ctx);
                    }
                    if ui.button("Reload disk").clicked() {
                        // Load authoring text as-is; do not bake macros into the editor buffer.
                        match fs::read_to_string(&self.path) {
                            Ok(raw) => {
                                self.source = raw;
                                self.drag = None;
                                self.clear_selection();
                                self.error = pipeline_doc(&self.source).err();
                            }
                            Err(e) => self.error = Some(format!("reload: {e}")),
                        }
                    }
                    if ui.button("Export PDF").clicked() {
                        match document_for_export(&mut GuiExportHandler::default(), &self.source) {
                            Ok((doc, _)) => {
                                let pdf_path = self.path.with_extension("pdf");
                                let base = self.path.parent();
                                match fs::File::create(&pdf_path)
                                    .map_err(|e| e.to_string())
                                    .and_then(|f| {
                                        write_document_with_base(&doc, base, f)
                                            .map_err(|e| format!("{e:?}"))
                                    }) {
                                    Ok(()) => self.error = None,
                                    Err(e) => self.error = Some(format!("pdf: {e}")),
                                }
                            }
                            Err(e) => self.error = Some(e.display()),
                        }
                    }
                    if ui.button("Re-expand macros").clicked() {
                        // Explicit bake: replaces the editor buffer with expanded forms.
                        match expand_source(&self.source) {
                            Ok(s) => {
                                self.push_undo();
                                self.source = s;
                                self.error = pipeline_doc(&self.source).err();
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
                }
                if !response.has_focus() {
                    self.typing_undo_open = false;
                }
                if let Some(err) = &self.error {
                    ui.add_space(4.0);
                    ui.colored_label(egui::Color32::RED, err);
                }
            });

        // Snapshot layers for the right panel (may be empty on error).
        let layers_for_panel =
            collect_layers_page(&self.source, self.page_index).unwrap_or_default();

        egui::SidePanel::right("layers_panel")
            .resizable(true)
            .default_width(240.0)
            .show(ctx, |ui| {
                ui.heading("Layers");
                ui.label("Drag rows vertically to restack. Top = front (later in source).");
                ui.separator();
                let n = layers_for_panel.len();
                let mut reorder: Option<(usize, usize)> = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Top of list = topmost (last drawn = highest flatten index).
                    for flat in (0..n).rev() {
                        let Some(layer) = layers_for_panel.get(flat) else {
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
                                let painter = ui.ctx().layer_painter(egui::LayerId::new(
                                    egui::Order::Tooltip,
                                    id,
                                ));
                                painter.rect_filled(
                                    ghost,
                                    2.0,
                                    egui::Color32::from_rgba_unmultiplied(30, 120, 220, 40),
                                );
                                painter.rect_stroke(
                                    ghost,
                                    2.0,
                                    egui::Stroke::new(
                                        1.0_f32,
                                        egui::Color32::from_rgb(30, 120, 220),
                                    ),
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
                            self.select_layer(flat, &layers_for_panel);
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
                                    egui::Stroke::new(
                                        2.0_f32,
                                        egui::Color32::from_rgb(30, 120, 220),
                                    ),
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
            });

        // Selection properties panel (bottom-right). Edits rewrite .rpx.
        self.show_properties_window(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Paper preview");
            ui.label("Scroll = zoom · Middle/Alt-drag = pan · Drag empty = marquee · Shift-click = add/remove · Ctrl+A = select all · Esc = clear · Body-drag = move · Ctrl-drag = 5mm snap · Corner = scale · Top knob = rotate · Arrows = nudge · Delete = remove · Ctrl+Shift+L = theme.");

            let doc = match pipeline_doc(&self.source) {
                Ok(d) => d,
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, e);
                    ui.label("Keep typing in the source pane — the caret stays there.");
                    return;
                }
            };
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
                ui.separator();
                if ui.button("−").clicked() {
                    self.zoom = (self.zoom / 1.15).max(0.2);
                }
                ui.label(format!("{:.0}%", self.zoom * 100.0));
                if ui.button("+").clicked() {
                    self.zoom = (self.zoom * 1.15).min(8.0);
                }
                if ui.button("Reset view").clicked() {
                    self.zoom = 1.0;
                    self.pan = egui::Vec2::ZERO;
                }
                ui.separator();
                if ui
                    .selectable_label(self.show_grid, "Grid")
                    .on_hover_text("10mm grid (G)")
                    .clicked()
                {
                    self.show_grid = !self.show_grid;
                }
            });

            let size_bindings = match collect_size_targets_page(&self.source, self.page_index) {
                Ok(b) => b,
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, &e.message);
                    return;
                }
            };
            let layers = match collect_layers_page(&self.source, self.page_index) {
                Ok(l) => l,
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, &e.message);
                    return;
                }
            };
            let Some((page, shapes)) = flatten_page(&doc, self.page_index) else {
                ui.colored_label(egui::Color32::RED, "Page not found.");
                return;
            };
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
            let (response, painter) = ui.allocate_painter(avail, egui::Sense::click_and_drag());
            let rect = response.rect;

            if response.hovered() {
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll.abs() > 0.0 {
                    let factor = (1.0 + scroll * 0.002).clamp(0.85, 1.15);
                    self.zoom = (self.zoom * factor).clamp(0.2, 8.0);
                }
            }
            // Pan with middle mouse or Alt + drag.
            let pan_gesture = response.dragged_by(egui::PointerButton::Middle)
                || (response.dragged() && ui.input(|i| i.modifiers.alt));
            if pan_gesture {
                self.pan += response.drag_delta();
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
                paint_paper_grid(&painter, rect, &layout, page.paper.width_mm, page.paper.height_mm);
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

            for (i, shape) in shapes.iter().enumerate() {
                paint_shape(
                    &painter,
                    rect,
                    &layout,
                    shape,
                    &mut self.textures,
                    ui.ctx(),
                    self.path.parent(),
                );
                if self.selected.contains(&i) {
                    if let Some(bounds) = PaperLayout::shape_bounds_mm(shape) {
                        paint_selection_frame(&painter, rect, &layout, bounds);
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
                || ui.input(|i| i.modifiers.alt);

            if let Some(pos) = response.interact_pointer_pos() {
                let local = pos - rect.min;
                let local_pos = egui::pos2(local.x, local.y);
                let (mx, my) = layout.px_to_mm(local.x, local.y);

                if !skip_shape_drag && response.drag_started() {
                    let mut started = false;
                    let hit_body = hit_test_shapes(&shapes, mx, my);
                    let shift = ui.input(|i| i.modifiers.shift);
                    // Handles only for primary selection when not grabbing the fill.
                    if let Some(sel) = self.primary_selected() {
                        if hit_body != Some(sel) {
                            if let Some(shape) = shapes.get(sel) {
                                if let Some(bounds) = PaperLayout::shape_bounds_mm(shape) {
                                    let (x0, y0, x1, y1) = bounds;
                                    let cx = (x0 + x1) * 0.5;
                                    let cy = (y0 + y1) * 0.5;
                                    if hit_rotate_handle(&layout, bounds, local_pos) {
                                        let start_angle_rad = (my - cy).atan2(mx - cx);
                                        let base_deg = layer_rotation_deg(
                                            &self.source,
                                            self.page_index,
                                            sel,
                                        )
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
                                            ScaleGrab::Edge(_) => size_bindings.get(sel).is_some_and(
                                                |t| {
                                                    matches!(
                                                        t,
                                                        SizeTarget::TextSize(_)
                                                            | SizeTarget::RectWh(_)
                                                            | SizeTarget::FrameWh(_)
                                                            | SizeTarget::ImageWh(_)
                                                    )
                                                },
                                            ),
                                        };
                                        if allow
                                            && size_bindings
                                                .get(sel)
                                                .is_some_and(|t| *t != SizeTarget::Unsupported)
                                        {
                                            let start_dist =
                                                ((mx - cx).hypot(my - cy)).max(1e-6);
                                            if let Some(size) = size_bindings.get(sel).copied() {
                                                let box_drag = match size {
                                                    SizeTarget::TextSize(_)
                                                    | SizeTarget::RectWh(_)
                                                    | SizeTarget::FrameWh(_)
                                                    | SizeTarget::ImageWh(_) => Some(BoxDrag {
                                                        grab,
                                                        start_bounds: bounds,
                                                    }),
                                                    _ => None,
                                                };
                                                self.drag = Some(DragState {
                                                    kind: DragKind::Scale {
                                                        size,
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
                                // Ctrl/Cmd: snap motion to a 5mm grid (paper coords).
                                if ui.input(|i| i.modifiers.command) {
                                    const GRID: f64 = 5.0;
                                    let snap = |v: f64| (v / GRID).round() * GRID;
                                    dx = snap(mx) - snap(last_mm.0);
                                    dy = snap(my) - snap(last_mm.1);
                                }
                                let mut indices = flat_indices;
                                if dx.abs() > 1e-9 || dy.abs() > 1e-9 {
                                    let mut undo_pushed = drag.undo_pushed;
                                    if !undo_pushed {
                                        self.push_undo();
                                        undo_pushed = true;
                                    }
                                    let mut src = self.source.clone();
                                    indices.sort_unstable();
                                    let mut ok = true;
                                    for &idx in &indices {
                                        match nudge_layer_page(
                                            &src,
                                            self.page_index,
                                            idx,
                                            dx,
                                            dy,
                                        ) {
                                            Ok(new_src) => src = new_src,
                                            Err(e) => {
                                                self.error = Some(e.message);
                                                ok = false;
                                                break;
                                            }
                                        }
                                    }
                                    if ok {
                                        self.source = src;
                                        self.error = if self.reload_ok() {
                                            None
                                        } else {
                                            Some("edit produced invalid program".into())
                                        };
                                        self.drag = Some(DragState {
                                            kind: DragKind::Move {
                                                last_mm: (mx, my),
                                                flat_indices: indices,
                                            },
                                            undo_pushed,
                                        });
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
                                base_src,
                                center_mm,
                                start_dist,
                                text_box,
                            } => {
                                let scale_result = if let Some(tb) = text_box {
                                    let (nx, ny, nw, nh) = box_from_grab(tb, mx, my);
                                    match size {
                                        SizeTarget::TextSize(idx) => {
                                            set_text_box(&base_src, idx, nx, ny, nw, nh)
                                        }
                                        SizeTarget::RectWh(idx) => set_box_xywh(
                                            &base_src, "rect", idx, [1, 2, 3, 4], nx, ny, nw, nh,
                                        ),
                                        SizeTarget::FrameWh(idx) => set_box_xywh(
                                            &base_src, "frame", idx, [1, 2, 3, 4], nx, ny, nw, nh,
                                        ),
                                        SizeTarget::ImageWh(idx) => set_box_xywh(
                                            &base_src, "image", idx, [2, 3, 4, 5], nx, ny, nw, nh,
                                        ),
                                        other => {
                                            let dist = (mx - center_mm.0)
                                                .hypot(my - center_mm.1)
                                                .max(1e-6);
                                            let factor = (dist / start_dist).clamp(0.05, 20.0);
                                            scale_size_target(&base_src, other, factor)
                                        }
                                    }
                                } else {
                                    let dist = (mx - center_mm.0)
                                        .hypot(my - center_mm.1)
                                        .max(1e-6);
                                    let factor = (dist / start_dist).clamp(0.05, 20.0);
                                    scale_size_target(&base_src, size, factor)
                                };
                                match scale_result {
                                    Ok(new_src) => {
                                        let kind = DragKind::Scale {
                                            size,
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
                                        self.error = if self.reload_ok() {
                                            None
                                        } else {
                                            Some("edit produced invalid program".into())
                                        };
                                        self.drag = Some(DragState { kind, undo_pushed });
                                    }
                                    Err(e) => self.error = Some(e.message),
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
                                let delta_deg =
                                    (angle - start_angle_rad).to_degrees();
                                let deg = base_deg + delta_deg;
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
                                        self.error = if self.reload_ok() {
                                            None
                                        } else {
                                            Some("edit produced invalid program".into())
                                        };
                                        self.drag = Some(DragState { kind, undo_pushed });
                                    }
                                    Err(e) => self.error = Some(e.message),
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
                            self.selected = hits;
                            self.props_open = !self.selected.is_empty();
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
                kind: DragKind::Marquee {
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
        });
    }
}

fn paint_selection_frame(
    painter: &egui::Painter,
    rect: egui::Rect,
    layout: &PaperLayout,
    bounds: (f64, f64, f64, f64),
) {
    let (x0, y0, x1, y1) = bounds;
    let (ax, ay) = layout.mm_to_px(x0, y1);
    let (bx, by) = layout.mm_to_px(x1, y0);
    let frame = egui::Rect::from_min_max(
        rect.min + egui::vec2(ax.min(bx) - 3.0, ay.min(by) - 3.0),
        rect.min + egui::vec2(ax.max(bx) + 3.0, ay.max(by) + 3.0),
    );
    painter.rect_stroke(
        frame,
        0.0,
        egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(30, 120, 220)),
        egui::StrokeKind::Outside,
    );
    let handle = 5.0;
    let corners = [
        frame.left_top(),
        frame.right_top(),
        frame.left_bottom(),
        frame.right_bottom(),
    ];
    for c in corners {
        let hr = egui::Rect::from_center_size(c, egui::vec2(handle * 2.0, handle * 2.0));
        painter.rect_filled(hr, 0.0, egui::Color32::WHITE);
        painter.rect_stroke(
            hr,
            0.0,
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(30, 120, 220)),
            egui::StrokeKind::Outside,
        );
    }
    let edges = [
        frame.center_top(),
        frame.center_bottom(),
        frame.left_center(),
        frame.right_center(),
    ];
    for c in edges {
        let hr = egui::Rect::from_center_size(c, egui::vec2(handle * 1.6, handle * 1.6));
        painter.rect_filled(hr, 0.0, egui::Color32::WHITE);
        painter.rect_stroke(
            hr,
            0.0,
            egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(30, 120, 220)),
            egui::StrokeKind::Outside,
        );
    }
    // Rotate knob: above the top-center of the selection frame.
    let knob = rotate_handle_pos(frame);
    painter.line_segment(
        [egui::pos2(frame.center().x, frame.top()), knob],
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(30, 120, 220)),
    );
    painter.circle_filled(knob, 5.0, egui::Color32::WHITE);
    painter.circle_stroke(
        knob,
        5.0,
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(30, 120, 220)),
    );
}

fn selection_frame_local(layout: &PaperLayout, bounds: (f64, f64, f64, f64)) -> egui::Rect {
    let (x0, y0, x1, y1) = bounds;
    let (ax, ay) = layout.mm_to_px(x0, y1);
    let (bx, by) = layout.mm_to_px(x1, y0);
    egui::Rect::from_min_max(
        egui::pos2(ax.min(bx) - 3.0, ay.min(by) - 3.0),
        egui::pos2(ax.max(bx) + 3.0, ay.max(by) + 3.0),
    )
}

fn rotate_handle_pos(frame: egui::Rect) -> egui::Pos2 {
    egui::pos2(frame.center().x, frame.top() - 22.0)
}

fn hit_scale_grab(
    layout: &PaperLayout,
    bounds: (f64, f64, f64, f64),
    local_px: egui::Pos2,
) -> Option<ScaleGrab> {
    let frame = selection_frame_local(layout, bounds);
    let hit_r2 = 10.0_f32 * 10.0;
    let corners = [
        (frame.left_top(), ScaleGrab::Corner(ScaleCorner::TopLeft)),
        (frame.right_top(), ScaleGrab::Corner(ScaleCorner::TopRight)),
        (
            frame.left_bottom(),
            ScaleGrab::Corner(ScaleCorner::BottomLeft),
        ),
        (
            frame.right_bottom(),
            ScaleGrab::Corner(ScaleCorner::BottomRight),
        ),
    ];
    if let Some((_, grab)) = corners
        .into_iter()
        .find(|(c, _)| local_px.distance_sq(*c) <= hit_r2)
    {
        return Some(grab);
    }
    let edges = [
        (frame.center_top(), ScaleGrab::Edge(ScaleEdge::Top)),
        (frame.center_bottom(), ScaleGrab::Edge(ScaleEdge::Bottom)),
        (frame.left_center(), ScaleGrab::Edge(ScaleEdge::Left)),
        (frame.right_center(), ScaleGrab::Edge(ScaleEdge::Right)),
    ];
    edges
        .into_iter()
        .find(|(c, _)| local_px.distance_sq(*c) <= hit_r2)
        .map(|(_, grab)| grab)
}

fn box_from_grab(tb: BoxDrag, mx: f64, my: f64) -> (f64, f64, f64, f64) {
    let (x0, y0, x1, y1) = tb.start_bounds;
    const MIN: f64 = 0.5;
    match tb.grab {
        ScaleGrab::Corner(ScaleCorner::TopLeft) => {
            let w = (x1 - mx).max(MIN);
            let h = (my - y0).max(MIN);
            (x1 - w, y0, w, h)
        }
        ScaleGrab::Corner(ScaleCorner::TopRight) => {
            let w = (mx - x0).max(MIN);
            let h = (my - y0).max(MIN);
            (x0, y0, w, h)
        }
        ScaleGrab::Corner(ScaleCorner::BottomLeft) => {
            let w = (x1 - mx).max(MIN);
            let h = (y1 - my).max(MIN);
            (x1 - w, y1 - h, w, h)
        }
        ScaleGrab::Corner(ScaleCorner::BottomRight) => {
            let w = (mx - x0).max(MIN);
            let h = (y1 - my).max(MIN);
            (x0, y1 - h, w, h)
        }
        ScaleGrab::Edge(ScaleEdge::Left) => {
            let w = (x1 - mx).max(MIN);
            (x1 - w, y0, w, y1 - y0)
        }
        ScaleGrab::Edge(ScaleEdge::Right) => {
            let w = (mx - x0).max(MIN);
            (x0, y0, w, y1 - y0)
        }
        ScaleGrab::Edge(ScaleEdge::Top) => {
            let h = (my - y0).max(MIN);
            (x0, y0, x1 - x0, h)
        }
        ScaleGrab::Edge(ScaleEdge::Bottom) => {
            let h = (y1 - my).max(MIN);
            (x0, y1 - h, x1 - x0, h)
        }
    }
}

fn paint_paper_grid(
    painter: &egui::Painter,
    rect: egui::Rect,
    layout: &PaperLayout,
    paper_w_mm: f64,
    paper_h_mm: f64,
) {
    const STEP: f64 = 10.0;
    let stroke = egui::Stroke::new(
        1.0_f32,
        egui::Color32::from_rgba_unmultiplied(40, 80, 140, 35),
    );
    let mut x = 0.0;
    while x <= paper_w_mm + 1e-6 {
        let (px0, py0) = layout.mm_to_px(x, 0.0);
        let (px1, py1) = layout.mm_to_px(x, paper_h_mm);
        painter.line_segment(
            [
                rect.min + egui::vec2(px0, py0),
                rect.min + egui::vec2(px1, py1),
            ],
            stroke,
        );
        x += STEP;
    }
    let mut y = 0.0;
    while y <= paper_h_mm + 1e-6 {
        let (px0, py0) = layout.mm_to_px(0.0, y);
        let (px1, py1) = layout.mm_to_px(paper_w_mm, y);
        painter.line_segment(
            [
                rect.min + egui::vec2(px0, py0),
                rect.min + egui::vec2(px1, py1),
            ],
            stroke,
        );
        y += STEP;
    }
}

fn hit_rotate_handle(
    layout: &PaperLayout,
    bounds: (f64, f64, f64, f64),
    local_px: egui::Pos2,
) -> bool {
    let frame = selection_frame_local(layout, bounds);
    let knob = rotate_handle_pos(frame);
    local_px.distance_sq(knob) <= 12.0_f32 * 12.0
}

fn paint_shape(
    painter: &egui::Painter,
    rect: egui::Rect,
    layout: &PaperLayout,
    shape: &WorldShape,
    textures: &mut std::collections::HashMap<String, egui::TextureHandle>,
    ctx: &egui::Context,
    base: Option<&std::path::Path>,
) {
    match shape {
        WorldShape::Circle(c) => {
            let (x, y) = layout.mm_to_px(c.x_mm, c.y_mm);
            let r = layout.radius_mm_to_px(c.radius_mm);
            let center = rect.min + egui::vec2(x, y);
            match c.stroke_width_mm {
                None => {
                    painter.circle_filled(center, r, color32(c.color, c.alpha));
                }
                Some(w) => {
                    painter.circle_stroke(
                        center,
                        r,
                        egui::Stroke::new(
                            layout.radius_mm_to_px(w).max(1.0),
                            color32(c.color, c.alpha),
                        ),
                    );
                }
            }
        }
        WorldShape::Polygon(p) => {
            if p.points_mm.len() < 3 {
                return;
            }
            let mut points = Vec::with_capacity(p.points_mm.len());
            for &(x_mm, y_mm) in &p.points_mm {
                let (x, y) = layout.mm_to_px(x_mm, y_mm);
                points.push(rect.min + egui::vec2(x, y));
            }
            match p.stroke_width_mm {
                None => {
                    painter.add(egui::Shape::convex_polygon(
                        points,
                        color32(p.color, p.alpha),
                        egui::Stroke::NONE,
                    ));
                }
                Some(w) => {
                    let stroke = egui::Stroke::new(
                        layout.radius_mm_to_px(w).max(1.0),
                        color32(p.color, p.alpha),
                    );
                    for i in 0..points.len() {
                        let a = points[i];
                        let b = points[(i + 1) % points.len()];
                        painter.line_segment([a, b], stroke);
                    }
                }
            }
        }
        WorldShape::Text(t) => {
            let font_px = layout.radius_mm_to_px(t.size_mm).max(0.5);
            let color = color32(t.fill, t.alpha);
            let wrap_px = layout.radius_mm_to_px(t.width_mm).max(1.0);
            // Soft-wrap to the layout box width; hard newlines still break.
            let galley = painter.layout(
                t.content.clone(),
                egui::FontId::proportional(font_px),
                color,
                wrap_px,
            );
            // Baseline in screen pixels (page Y-up → screen Y-down).
            let (bx, by) = layout.mm_to_px(t.x_mm, t.y_mm);
            let baseline = rect.min + egui::vec2(bx, by);
            // Page CCW angle appears as clockwise in Y-down screen space.
            let angle = t.rotation_deg.to_radians() as f32;
            let h = galley.size().y;
            // Unrotated top-left is above the baseline; rotate that offset around baseline.
            let tl_rel = egui::vec2(0.0, -h);
            let (s, c) = (angle.sin(), angle.cos());
            let top_left =
                baseline + egui::vec2(tl_rel.x * c + tl_rel.y * s, -tl_rel.x * s + tl_rel.y * c);
            // Clip to the layout box AABB so height shrinks hide overflow.
            let box_corners = text_corners_mm(t);
            let mut clip = egui::Rect::NOTHING;
            for &(xmm, ymm) in &box_corners {
                let (px, py) = layout.mm_to_px(xmm, ymm);
                clip = clip.union(egui::Rect::from_center_size(
                    rect.min + egui::vec2(px, py),
                    egui::vec2(1.0, 1.0),
                ));
            }
            clip = clip.expand(2.0).intersect(painter.clip_rect());
            let clipped = painter.with_clip_rect(clip);
            clipped.add(egui::epaint::TextShape::new(top_left, galley, color).with_angle(angle));
        }
        WorldShape::Path(p) => {
            if p.points_mm.len() < 2 {
                return;
            }
            let stroke = egui::Stroke::new(
                layout.radius_mm_to_px(p.width_mm).max(1.0),
                color32(p.stroke, p.alpha),
            );
            let mut pts = Vec::with_capacity(p.points_mm.len());
            for &(x_mm, y_mm) in &p.points_mm {
                let (x, y) = layout.mm_to_px(x_mm, y_mm);
                pts.push(rect.min + egui::vec2(x, y));
            }
            let n = pts.len();
            let segs = if p.closed { n } else { n - 1 };
            for i in 0..segs {
                painter.line_segment([pts[i], pts[(i + 1) % n]], stroke);
            }
        }
        WorldShape::Image(img) => {
            let mut screen = [egui::pos2(0.0, 0.0); 4];
            for (i, &(x_mm, y_mm)) in img.corners_mm.iter().enumerate() {
                let (x, y) = layout.mm_to_px(x_mm, y_mm);
                screen[i] = rect.min + egui::vec2(x, y);
            }
            // corners: BL, BR, TR, TL — UVs match image space (V grows down).
            let uvs = [
                egui::pos2(0.0, 1.0),
                egui::pos2(1.0, 1.0),
                egui::pos2(1.0, 0.0),
                egui::pos2(0.0, 0.0),
            ];
            if let Some(tex) = ensure_texture(textures, ctx, &img.path, base) {
                let tint = egui::Color32::from_rgba_unmultiplied(
                    255,
                    255,
                    255,
                    (img.alpha * 255.0).round().clamp(0.0, 255.0) as u8,
                );
                let mut mesh = egui::Mesh::with_texture(tex.id());
                let i0 = mesh.vertices.len() as u32;
                for i in 0..4 {
                    mesh.vertices.push(egui::epaint::Vertex {
                        pos: screen[i],
                        uv: uvs[i],
                        color: tint,
                    });
                }
                mesh.indices
                    .extend_from_slice(&[i0, i0 + 1, i0 + 2, i0, i0 + 2, i0 + 3]);
                painter.add(egui::Shape::mesh(mesh));
            } else {
                let stroke = egui::Stroke::new(1.5_f32, egui::Color32::from_gray(60));
                for i in 0..4 {
                    painter.line_segment([screen[i], screen[(i + 1) % 4]], stroke);
                }
                painter.line_segment(
                    [screen[0], screen[2]],
                    egui::Stroke::new(1.0_f32, egui::Color32::from_gray(140)),
                );
            }
        }
    }
}

fn ensure_texture(
    textures: &mut std::collections::HashMap<String, egui::TextureHandle>,
    ctx: &egui::Context,
    path: &str,
    base: Option<&std::path::Path>,
) -> Option<egui::TextureHandle> {
    if let Some(tex) = textures.get(path) {
        return Some(tex.clone());
    }
    let resolved = match base {
        Some(b) => b.join(path),
        None => PathBuf::from(path),
    };
    let raster = reciplexa_pdf::load_raster_file(&resolved).ok()?;
    let mut rgba = Vec::with_capacity(raster.rgb.len() / 3 * 4);
    for px in raster.rgb.chunks_exact(3) {
        rgba.push(px[0]);
        rgba.push(px[1]);
        rgba.push(px[2]);
        rgba.push(255);
    }
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [raster.width as usize, raster.height as usize],
        &rgba,
    );
    let tex = ctx.load_texture(path.to_string(), color_image, egui::TextureOptions::LINEAR);
    textures.insert(path.to_string(), tex.clone());
    Some(tex)
}

fn color32(c: reciplexa_scene::Color, alpha: f64) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (c.r * 255.0).round().clamp(0.0, 255.0) as u8,
        (c.g * 255.0).round().clamp(0.0, 255.0) as u8,
        (c.b * 255.0).round().clamp(0.0, 255.0) as u8,
        (alpha * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}
