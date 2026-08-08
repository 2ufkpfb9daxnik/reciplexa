//! Phase 7 conformance: layered IR, planning, SVG export.

use reciplexa_backend::{
    export_scene_to_svg, validate_svg_artifact, BackendCapability, OutputProfile, PlanningError,
};
use reciplexa_scene::{Color, Document, Page, PaperSize, Rect, Shape};
use reciplexa_visual_ir::{lower_scene_document, validate_render_document};

#[test]
fn phase7_lowers_scene_to_render_ir() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 1.0,
            y_mm: 2.0,
            width_mm: 3.0,
            height_mm: 4.0,
            fill: Color::BLUE,
        })],
    });
    let (render, _) = lower_scene_document(&doc);
    validate_render_document(&render).unwrap();
}

#[test]
fn phase7_exports_validated_svg() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 10.0,
            y_mm: 10.0,
            width_mm: 50.0,
            height_mm: 30.0,
            fill: Color::RED,
        })],
    });
    let artifact = export_scene_to_svg(
        &doc,
        &BackendCapability::svg_default(),
        &OutputProfile::svg_default(),
    )
    .unwrap();
    validate_svg_artifact(&artifact.svg).unwrap();
    assert!(artifact.svg.contains("rpx-"));
}

#[test]
fn phase7_detects_capability_mismatch_at_planning() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 10.0,
            height_mm: 10.0,
            fill: Color::BLACK,
        })],
    });
    let mut cap = BackendCapability::svg_default();
    cap.svg_text = false;
    let mut profile = OutputProfile::svg_default();
    profile.requires_text = true;
    let err = export_scene_to_svg(&doc, &cap, &profile).unwrap_err();
    assert!(matches!(err, reciplexa_backend::ExportError::Planning(PlanningError::CapabilityMismatch(_))));
}
