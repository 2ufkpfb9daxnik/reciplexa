//! Paper preview with drag → CST sync, plus a live `.rpx` source pane.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use eframe::egui;
use eframe::egui::text::{CCursor, CCursorRange};
use reciplexa_lower::{
    collect_drag_targets_page, collect_layers_page, lower_source, nudge_drag_target,
    reorder_layer_page, DragTarget, LayerInfo,
};
use reciplexa_macro::expand_source;
use reciplexa_pdf::write_document_with_base;
use reciplexa_types::typecheck_source;
use reciplexa_view::{flatten_page, hit_test_shapes, PaperLayout, WorldShape};

fn pipeline_doc(src: &str) -> Result<reciplexa_scene::Document, String> {
    let expanded = expand_source(src).map_err(|e| format!("macro: {}", e.message))?;
    typecheck_source(&expanded)
        .map_err(|e| format!("type: {} @{}..{}", e.message, e.start, e.end))?;
    lower_source(&expanded).map_err(|e| e.message)
}

fn install_cjk_fonts(ctx: &egui::Context) {
    let windir = PathBuf::from(env::var_os("WINDIR").unwrap_or_else(|| r"C:\Windows".into()));
    let fonts_dir = windir.join("Fonts");
    let candidates = [
        fonts_dir.join("NotoSansJP-VF.ttf"),
        fonts_dir.join("NotoSansJP-VariableFont_wght.ttf"),
        fonts_dir.join("NotoSans-Regular.ttf"),
    ];
    let Some(path) = candidates.into_iter().find(|p| p.is_file()) else {
        eprintln!("warn: no CJK-capable TTF found for GUI preview");
        return;
    };
    let Ok(bytes) = fs::read(&path) else {
        eprintln!("warn: could not read font {}", path.display());
        return;
    };
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "reciplexa_cjk".into(),
        Arc::new(egui::FontData::from_owned(bytes)),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "reciplexa_cjk".into());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("reciplexa_cjk".into());
    ctx.set_fonts(fonts);
    eprintln!("gui font: {}", path.display());
}

fn suppress_ime_confirm_newline(ctx: &egui::Context, hold_frames: &mut u8) {
    // egui 0.31: IME 確定の Enter が同じフレームで改行として漏れることがある。
    // Commit があるフレームとその直後は Enter / "\n" を落とす。
    ctx.input_mut(|input| {
        let has_ime = input
            .events
            .iter()
            .any(|e| matches!(e, egui::Event::Ime(_)));
        let has_commit = input.events.iter().any(|e| {
            matches!(e, egui::Event::Ime(egui::ImeEvent::Commit(_)))
        });
        if has_commit {
            *hold_frames = (*hold_frames).max(2);
        }
        if has_ime || *hold_frames > 0 {
            input.events.retain(|e| {
                let is_enter =
                    matches!(e, egui::Event::Key { key: egui::Key::Enter, .. });
                let is_newline =
                    matches!(e, egui::Event::Text(t) if t == "\n" || t == "\r\n");
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
        state.cursor.set_char_range(Some(CCursorRange::two(
            CCursor::new(c0),
            CCursor::new(c1),
        )));
        state.store(ctx, id);
    }
}

fn main() -> ExitCode {
    let path = match env::args().nth(1) {
        Some(p) => PathBuf::from(p),
        None => {
            eprintln!("usage: reciplexa-gui <input.rpx>");
            return ExitCode::from(2);
        }
    };

    let src = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: read {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let (src, mut initial_error) = match expand_source(&src) {
        Ok(s) => (s, None),
        Err(e) => (src, Some(format!("macro: {}", e.message))),
    };
    if initial_error.is_none() {
        initial_error = pipeline_doc(&src).err();
    }
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
            Ok(Box::new(PreviewApp {
                path,
                source: src,
                error: initial_error,
                drag: None,
                page_index: 0,
                zoom: 1.0,
                pan: egui::Vec2::ZERO,
                selected: None,
                pending_source_select: None,
                textures: std::collections::HashMap::new(),
                ime_enter_hold: 0,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
                typing_undo_open: false,
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
    selected: Option<usize>,
    pending_source_select: Option<(usize, usize)>,
    /// Texture cache keyed by image path string from the `.rpx`.
    textures: std::collections::HashMap<String, egui::TextureHandle>,
    /// Frames to keep suppressing Enter after IME Commit.
    ime_enter_hold: u8,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    /// Coalesce TextEdit keystrokes into one undo step while focused.
    typing_undo_open: bool,
}

struct DragState {
    last_mm: (f64, f64),
    target: DragTarget,
    undo_pushed: bool,
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
        self.selected = Some(index);
        if let Some(layer) = layers.get(index) {
            self.pending_source_select = Some((layer.byte_start, layer.byte_end));
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
                self.selected = Some(to);
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
}

impl eframe::App for PreviewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        suppress_ime_confirm_newline(ctx, &mut self.ime_enter_hold);

        // Ctrl+Z / Ctrl+Y for source undo/redo (IME mistakes, layer moves, etc.).
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift)
        {
            self.undo();
        }
        if ctx.input(|i| {
            (i.modifiers.command && i.key_pressed(egui::Key::Y))
                || (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z))
        }) {
            self.redo();
        }

        if let Some((start, end)) = self.pending_source_select.take() {
            highlight_source_range(ctx, &self.source, start, end);
        }

        egui::SidePanel::left("source_panel")
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading(".rpx source");
                ui.label("Edits reproject live; drag on the paper rewrites numbers here.");
                ui.horizontal(|ui| {
                    if ui.button("Save .rpx").clicked() {
                        if let Err(e) = fs::write(&self.path, &self.source) {
                            self.error = Some(format!("save: {e}"));
                        } else {
                            self.error = None;
                        }
                    }
                    if ui.button("Reload disk").clicked() {
                        match fs::read_to_string(&self.path) {
                            Ok(raw) => match expand_source(&raw) {
                                Ok(s) => {
                                    self.source = s;
                                    self.drag = None;
                                    self.selected = None;
                                    self.error = pipeline_doc(&self.source).err();
                                }
                                Err(e) => self.error = Some(format!("macro: {}", e.message)),
                            },
                            Err(e) => self.error = Some(format!("reload: {e}")),
                        }
                    }
                    if ui.button("Export PDF").clicked() {
                        match pipeline_doc(&self.source) {
                            Ok(doc) => {
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
                            Err(e) => self.error = Some(e),
                        }
                    }
                    if ui.button("Re-expand macros").clicked() {
                        match expand_source(&self.source) {
                            Ok(s) => {
                                self.source = s;
                                self.error = None;
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
                    egui::vec2(ui.available_width(), (ui.available_height() - 48.0).max(120.0)),
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
        let layers_for_panel = collect_layers_page(&self.source, self.page_index).unwrap_or_default();

        egui::SidePanel::right("layers_panel")
            .resizable(true)
            .default_width(240.0)
            .show(ctx, |ui| {
                ui.heading("Layers");
                ui.label("Drag any row to restack. Top = front (later in source).");
                ui.separator();
                let n = layers_for_panel.len();
                let mut reorder: Option<(usize, usize)> = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Top of list = topmost (last drawn = highest flatten index).
                    for flat in (0..n).rev() {
                        let Some(layer) = layers_for_panel.get(flat) else {
                            continue;
                        };
                        let selected = self.selected == Some(flat);
                        let id = egui::Id::new(("layer_dnd", self.page_index, flat));
                        let text = format!("{}. {}", flat + 1, layer.label);
                        let row = ui.dnd_drag_source(id, flat, |ui| {
                            ui.horizontal(|ui| {
                                ui.weak("⠿");
                                let _ = ui.selectable_label(selected, &text);
                            });
                        });
                        let response = row
                            .response
                            .on_hover_cursor(egui::CursorIcon::Grab)
                            .on_hover_text("Drag to change stacking order");
                        if response.clicked() {
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
                                        2.0,
                                        egui::Color32::from_rgb(30, 120, 220),
                                    ),
                                );
                                if let Some(from) = response.dnd_release_payload::<usize>() {
                                    // Visual top = high flat. Upper half → just in front of
                                    // this row; lower half → this row's slot.
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
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Paper preview");
            ui.label("Scroll = zoom · Middle-drag / Alt-drag = pan · Drag shapes to edit.");

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
                    self.selected = None;
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
                    self.selected = None;
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
            });

            let bindings = match collect_drag_targets_page(&self.source, self.page_index) {
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
            if bindings.len() != shapes.len() || layers.len() != shapes.len() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!(
                        "binding/layer/shape mismatch: {} / {} / {}",
                        bindings.len(),
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
            painter.rect_filled(paper, 0.0, egui::Color32::from_gray(245));
            painter.rect_stroke(
                paper,
                0.0,
                egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
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
                if self.selected == Some(i) {
                    if let Some((x0, y0, x1, y1)) = PaperLayout::shape_bounds_mm(shape) {
                        let (ax, ay) = layout.mm_to_px(x0, y1);
                        let (bx, by) = layout.mm_to_px(x1, y0);
                        let r = egui::Rect::from_min_max(
                            rect.min + egui::vec2(ax.min(bx) - 3.0, ay.min(by) - 3.0),
                            rect.min + egui::vec2(ax.max(bx) + 3.0, ay.max(by) + 3.0),
                        );
                        painter.rect_stroke(
                            r,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(30, 120, 220)),
                            egui::StrokeKind::Outside,
                        );
                    }
                }
            }

            let skip_shape_drag = pan_gesture
                || ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle))
                || ui.input(|i| i.modifiers.alt);

            if let Some(pos) = response.interact_pointer_pos() {
                let local = pos - rect.min;
                let (mx, my) = layout.px_to_mm(local.x, local.y);

                if !skip_shape_drag && response.drag_started() {
                    if let Some(i) = hit_test_shapes(&shapes, mx, my) {
                        self.select_layer(i, &layers);
                        if let Some(target) = bindings.get(i).copied() {
                            self.drag = Some(DragState {
                                last_mm: (mx, my),
                                target,
                                undo_pushed: false,
                            });
                        }
                    }
                }

                if !skip_shape_drag && response.dragged() {
                    if let Some(drag) = &self.drag {
                        let dx = mx - drag.last_mm.0;
                        let dy = my - drag.last_mm.1;
                        if dx.abs() > 1e-9 || dy.abs() > 1e-9 {
                            match nudge_drag_target(&self.source, drag.target, dx, dy) {
                                Ok(new_src) => {
                                    let target = drag.target;
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
                                    self.drag = Some(DragState {
                                        last_mm: (mx, my),
                                        target,
                                        undo_pushed,
                                    });
                                }
                                Err(e) => self.error = Some(e.message),
                            }
                        }
                    }
                }

                if response.drag_stopped() {
                    self.drag = None;
                }
            } else if response.drag_stopped() {
                self.drag = None;
            }
        });
    }
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
            let (x, y) = layout.mm_to_px(t.x_mm, t.y_mm + t.size_mm);
            let font_px = layout.radius_mm_to_px(t.size_mm).max(8.0);
            painter.text(
                rect.min + egui::vec2(x, y),
                egui::Align2::LEFT_BOTTOM,
                &t.content,
                egui::FontId::proportional(font_px),
                color32(t.fill, t.alpha),
            );
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
            let (x0, y0) = layout.mm_to_px(img.x_mm, img.y_mm + img.height_mm);
            let (x1, y1) = layout.mm_to_px(img.x_mm + img.width_mm, img.y_mm);
            let r = egui::Rect::from_min_max(
                rect.min + egui::vec2(x0, y0),
                rect.min + egui::vec2(x1, y1),
            );
            if let Some(tex) = ensure_texture(textures, ctx, &img.path, base) {
                let mut mesh = egui::Mesh::with_texture(tex.id());
                let tint = egui::Color32::from_rgba_unmultiplied(
                    255,
                    255,
                    255,
                    (img.alpha * 255.0).round().clamp(0.0, 255.0) as u8,
                );
                mesh.add_rect_with_uv(
                    r,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    tint,
                );
                painter.add(egui::Shape::mesh(mesh));
            } else {
                // Fallback placeholder when the file is missing / unloadable.
                painter.rect_filled(r, 0.0, egui::Color32::from_gray(230));
                painter.rect_stroke(
                    r,
                    0.0,
                    egui::Stroke::new(1.5, egui::Color32::from_gray(60)),
                    egui::StrokeKind::Outside,
                );
                painter.line_segment(
                    [r.left_top(), r.right_bottom()],
                    egui::Stroke::new(1.0, egui::Color32::from_gray(140)),
                );
                painter.line_segment(
                    [r.left_bottom(), r.right_top()],
                    egui::Stroke::new(1.0, egui::Color32::from_gray(140)),
                );
                let label = img
                    .path
                    .rsplit(['/', '\\'])
                    .next()
                    .unwrap_or(img.path.as_str());
                painter.text(
                    r.left_top() + egui::vec2(4.0, 4.0),
                    egui::Align2::LEFT_TOP,
                    label,
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_gray(40),
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
