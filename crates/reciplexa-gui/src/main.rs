//! Paper preview with drag → CST sync, plus a live `.rpx` source pane.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use eframe::egui;
use reciplexa_lower::{collect_drag_targets, lower_source, nudge_drag_target, DragTarget};
use reciplexa_macro::expand_source;
use reciplexa_types::typecheck_source;
use reciplexa_view::{flatten_first_page, hit_test_shapes, PaperLayout, WorldShape};

fn pipeline_doc(src: &str) -> Result<reciplexa_scene::Document, String> {
    let expanded = expand_source(src).map_err(|e| format!("macro: {}", e.message))?;
    typecheck_source(&expanded)
        .map_err(|e| format!("type: {} @{}..{}", e.message, e.start, e.end))?;
    lower_source(&expanded).map_err(|e| e.message)
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
    // Expand macros into the edit buffer so drag byte-offsets stay valid.
    let src = match expand_source(&src) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: macro: {}", e.message);
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = pipeline_doc(&src) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }

    let title = format!("reciplexa — {}", path.display());
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 900.0])
            .with_title(title),
        ..Default::default()
    };

    match eframe::run_native(
        "reciplexa",
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(PreviewApp {
                path,
                source: src,
                error: None,
                drag: None,
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
}

struct DragState {
    last_mm: (f64, f64),
    target: DragTarget,
}

impl PreviewApp {
    fn reload_ok(&self) -> bool {
        pipeline_doc(&self.source).is_ok()
    }
}

impl eframe::App for PreviewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("source_panel")
            .resizable(true)
            .default_width(420.0)
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
                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, err);
                }
                ui.add_space(4.0);
                let editor = egui::TextEdit::multiline(&mut self.source)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(40);
                let response = ui.add_sized(ui.available_size(), editor);
                if response.changed() {
                    self.drag = None;
                    self.error = pipeline_doc(&self.source).err();
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Paper preview — drag shapes");
            ui.label(
                "Drag updates bound translate / circle / rect / ellipse / text / line leaves.",
            );

            let doc = match pipeline_doc(&self.source) {
                Ok(d) => d,
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, e);
                    return;
                }
            };
            let bindings = match collect_drag_targets(&self.source) {
                Ok(b) => b,
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, &e.message);
                    return;
                }
            };
            let Some((page, shapes)) = flatten_first_page(&doc) else {
                ui.colored_label(egui::Color32::RED, "Document has no pages.");
                return;
            };
            if bindings.len() != shapes.len() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!(
                        "binding/shape count mismatch: {} vs {}",
                        bindings.len(),
                        shapes.len()
                    ),
                );
            }

            let avail = ui.available_size();
            let (response, painter) = ui.allocate_painter(avail, egui::Sense::click_and_drag());
            let rect = response.rect;
            let layout = PaperLayout::fit(
                rect.width(),
                rect.height(),
                24.0,
                page.paper.width_mm,
                page.paper.height_mm,
            );

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

            for shape in &shapes {
                match shape {
                    WorldShape::Circle(c) => {
                        let (x, y) = layout.mm_to_px(c.x_mm, c.y_mm);
                        let r = layout.radius_mm_to_px(c.radius_mm);
                        let center = rect.min + egui::vec2(x, y);
                        painter.circle_filled(center, r, color32(c.fill));
                    }
                    WorldShape::Polygon(p) => {
                        if p.points_mm.len() < 3 {
                            continue;
                        }
                        let mut points = Vec::with_capacity(p.points_mm.len());
                        for &(x_mm, y_mm) in &p.points_mm {
                            let (x, y) = layout.mm_to_px(x_mm, y_mm);
                            points.push(rect.min + egui::vec2(x, y));
                        }
                        painter.add(egui::Shape::convex_polygon(
                            points,
                            color32(p.fill),
                            egui::Stroke::NONE,
                        ));
                    }
                }
            }

            if let Some(pos) = response.interact_pointer_pos() {
                let local = pos - rect.min;
                let (mx, my) = layout.px_to_mm(local.x, local.y);

                if response.drag_started() {
                    if let Some(i) = hit_test_shapes(&shapes, mx, my) {
                        if let Some(target) = bindings.get(i).copied() {
                            self.drag = Some(DragState {
                                last_mm: (mx, my),
                                target,
                            });
                        }
                    }
                }

                if response.dragged() {
                    if let Some(drag) = &self.drag {
                        let dx = mx - drag.last_mm.0;
                        let dy = my - drag.last_mm.1;
                        if dx.abs() > 1e-9 || dy.abs() > 1e-9 {
                            match nudge_drag_target(&self.source, drag.target, dx, dy) {
                                Ok(new_src) => {
                                    let target = drag.target;
                                    self.source = new_src;
                                    self.error = if self.reload_ok() {
                                        None
                                    } else {
                                        Some("edit produced invalid program".into())
                                    };
                                    self.drag = Some(DragState {
                                        last_mm: (mx, my),
                                        target,
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

fn color32(c: reciplexa_scene::Color) -> egui::Color32 {
    egui::Color32::from_rgb(
        (c.r * 255.0).round().clamp(0.0, 255.0) as u8,
        (c.g * 255.0).round().clamp(0.0, 255.0) as u8,
        (c.b * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}
