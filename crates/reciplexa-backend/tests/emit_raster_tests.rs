//! Raster emit tests.

use reciplexa_backend::emit_raster::emit_raster_page_from_plan;
use reciplexa_backend::plan::{plan_raster, BackendTarget};
use reciplexa_backend::profile::{OutputProfile, ProfileKind};
use reciplexa_backend::{BackendCapability, LossDisposition, OutputLossKind};
use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Shape, Text};
use reciplexa_visual_ir::lower_scene_document;

#[test]
fn emit_raster_preview_reports_text_loss() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![
            Shape::Circle(Circle {
                x_mm: 50.0,
                y_mm: 50.0,
                radius_mm: 10.0,
                fill: Color::BLACK,
            }),
            Shape::Text(Text {
                x_mm: 1.0,
                y_mm: 2.0,
                size_mm: 12.0,
                width_mm: None,
                height_mm: None,
                content: "hi".into(),
                fill: Color::BLACK,
            }),
        ],
    });
    let (render, _) = lower_scene_document(&doc);
    let profile = OutputProfile::preview_raster();
    let plan = plan_raster(&render, &BackendCapability::raster_default(), &profile).unwrap();
    assert_eq!(plan.target, BackendTarget::Raster);
    let page = emit_raster_page_from_plan(&plan, &doc, 0).unwrap();
    assert!(page.png.starts_with(&[0x89, b'P', b'N', b'G']));
    assert_eq!(page.losses.profile, ProfileKind::Preview);
    assert!(page.losses.losses.iter().any(|l| {
        l.kind == OutputLossKind::SemanticText && l.disposition == LossDisposition::Report
    }));
}

#[test]
fn emit_raster_final_stricter_disposition_and_higher_res() {
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 10.0,
            height_mm: 10.0,
        },
        shapes: vec![Shape::Text(Text {
            x_mm: 1.0,
            y_mm: 2.0,
            size_mm: 3.0,
            width_mm: None,
            height_mm: None,
            content: "x".into(),
            fill: Color::BLACK,
        })],
    });
    let (render, _) = lower_scene_document(&doc);
    let preview = plan_raster(
        &render,
        &BackendCapability::raster_default(),
        &OutputProfile::preview_raster(),
    )
    .unwrap();
    let final_plan = plan_raster(
        &render,
        &BackendCapability::raster_default(),
        &OutputProfile::final_raster(),
    )
    .unwrap();
    let preview_page = emit_raster_page_from_plan(&preview, &doc, 0).unwrap();
    let final_page = emit_raster_page_from_plan(&final_plan, &doc, 0).unwrap();
    assert!(final_page.width > preview_page.width);
    assert!(final_page.losses.losses.iter().any(|l| {
        l.kind == OutputLossKind::SemanticText
            && l.disposition == LossDisposition::RequireExplicitApproval
            && l.profile == ProfileKind::Final
    }));
}
