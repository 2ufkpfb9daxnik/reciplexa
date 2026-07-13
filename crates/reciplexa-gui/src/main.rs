//! M7: paper preview with drag → CST translate sync.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use eframe::egui;
use reciplexa_lower::{lower_source, nudge_first_translate};
use reciplexa_types::typecheck_source;
use reciplexa_view::{flatten_first_page, hit_test_circles, PaperLayout};

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
    if let Err(e) = typecheck_source(&src) {
        eprintln!("error: type: {} @{}..{}", e.message, e.start, e.end);
        return ExitCode::FAILURE;
    }
    if let Err(e) = lower_source(&src) {
        eprintln!("error: {}", e.message);
        return ExitCode::FAILURE;
    }

    let title = format!("reciplexa — {}", path.display());
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 1100.0])
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
}

impl PreviewApp {
    fn reload_ok(&self) -> bool {
        typecheck_source(&self.source).is_ok() && lower_source(&self.source).is_ok()
    }
}

impl eframe::App for PreviewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Paper preview (M7 — drag translate)");
            ui.label(
                "Drag the circle to nudge the first (translate …) in the .rpx (CST-preserving).",
            );
            if ui.button("Save .rpx").clicked() {
                if let Err(e) = fs::write(&self.path, &self.source) {
                    self.error = Some(format!("save: {e}"));
                } else {
                    self.error = None;
                }
            }
            if let Some(err) = &self.error {
                ui.colored_label(egui::Color32::RED, err);
            }
            ui.add_space(8.0);

            let doc = match lower_source(&self.source) {
                Ok(d) => d,
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, &e.message);
                    return;
                }
            };
            let Some((page, circles)) = flatten_first_page(&doc) else {
                ui.colored_label(egui::Color32::RED, "Document has no pages.");
                return;
            };

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

            for c in &circles {
                let (x, y) = layout.mm_to_px(c.x_mm, c.y_mm);
                let r = layout.radius_mm_to_px(c.radius_mm);
                let center = rect.min + egui::vec2(x, y);
                let color = egui::Color32::from_rgb(
                    (c.fill.r * 255.0).round().clamp(0.0, 255.0) as u8,
                    (c.fill.g * 255.0).round().clamp(0.0, 255.0) as u8,
                    (c.fill.b * 255.0).round().clamp(0.0, 255.0) as u8,
                );
                painter.circle_filled(center, r, color);
            }

            if let Some(pos) = response.interact_pointer_pos() {
                let local = pos - rect.min;
                let (mx, my) = layout.px_to_mm(local.x, local.y);

                if response.drag_started() && hit_test_circles(&circles, mx, my).is_some() {
                    self.drag = Some(DragState { last_mm: (mx, my) });
                }

                if response.dragged() {
                    if let Some(drag) = &self.drag {
                        let dx = mx - drag.last_mm.0;
                        let dy = my - drag.last_mm.1;
                        if dx.abs() > 1e-9 || dy.abs() > 1e-9 {
                            match nudge_first_translate(&self.source, dx, dy) {
                                Ok(new_src) => {
                                    self.source = new_src;
                                    self.error = if self.reload_ok() {
                                        None
                                    } else {
                                        Some("edit produced invalid program".into())
                                    };
                                    self.drag = Some(DragState { last_mm: (mx, my) });
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
