//! M6: view-only paper preview. Editing lands in M7.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use eframe::egui;
use reciplexa_lower::lower_source;
use reciplexa_scene::Document;
use reciplexa_types::typecheck_source;
use reciplexa_view::{flatten_first_page, PaperLayout};

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
    let doc = match lower_source(&src) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {}", e.message);
            return ExitCode::FAILURE;
        }
    };

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
        Box::new(move |_cc| Ok(Box::new(PreviewApp { doc }))),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: gui: {e}");
            ExitCode::FAILURE
        }
    }
}

struct PreviewApp {
    doc: Document,
}

impl eframe::App for PreviewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Paper preview (M6 — view only)");
            ui.label("Transforms and colors come from the .rpx scene. Drag editing is M7.");
            ui.add_space(8.0);

            let Some((page, circles)) = flatten_first_page(&self.doc) else {
                ui.colored_label(egui::Color32::RED, "Document has no pages.");
                return;
            };

            let avail = ui.available_size();
            let (response, painter) =
                ui.allocate_painter(avail, egui::Sense::hover());
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
        });
    }
}
