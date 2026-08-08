//! Phase 7 conformance: layered IR, planning, SVG export, preview, provenance.

use reciplexa_backend::{
    export_scene_to_preview, export_scene_to_svg, export_scene_to_svg_with_hints,
    validate_svg_artifact, BackendCapability, OutputProfile, PlanningError,
};
use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::{Color, Document, Ellipse, Page, PaperSize, Rect, Shape};
use reciplexa_visual_ir::{
    layout_identity, lower_scene_document, scene_to_domain, validate_render_document, LowerOptions,
    NodeSourceHint,
};

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
fn phase7_domain_layout_pipeline() {
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
    let domain = scene_to_domain(&doc, &LowerOptions::with_ellipse_sides(32));
    let layout = layout_identity(&domain);
    assert_eq!(layout.pages[0].nodes.len(), 1);
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
    assert!(artifact.svg.contains("<rect"));
}

#[test]
fn phase7_tracks_source_provenance_to_artifact() {
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
    let hints = vec![Some(NodeSourceHint {
        stable_node_id: StableNodeId::new(99),
        source_byte_start: 3,
        source_byte_end: 17,
    })];
    let artifact = export_scene_to_svg_with_hints(
        &doc,
        &BackendCapability::svg_default(),
        &OutputProfile::svg_default(),
        &hints,
    )
    .unwrap();
    assert_eq!(
        artifact.provenance[0].stable_node_id,
        Some(StableNodeId::new(99))
    );
    assert_eq!(artifact.provenance[0].source_byte_start, Some(3));
    assert_eq!(artifact.provenance[0].source_byte_end, Some(17));
}

#[test]
fn phase7_preview_backend_emits_plan_driven_drawables() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 5.0,
            y_mm: 5.0,
            width_mm: 10.0,
            height_mm: 10.0,
            fill: Color::BLACK,
        })],
    });
    let artifact = export_scene_to_preview(
        &doc,
        &BackendCapability::svg_default(),
        &OutputProfile::svg_default(),
    )
    .unwrap();
    assert_eq!(artifact.drawables.len(), 1);
    assert!(!artifact.provenance.is_empty());
}

#[test]
fn phase7_honors_ellipse_sides_from_profile() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Ellipse(Ellipse {
            x_mm: 50.0,
            y_mm: 50.0,
            rx_mm: 20.0,
            ry_mm: 10.0,
            fill: Color::BLACK,
        })],
    });
    let mut profile = OutputProfile::svg_default();
    profile.ellipse_sides = 16;
    let artifact = export_scene_to_svg(&doc, &BackendCapability::svg_default(), &profile).unwrap();
    // 16-gon → one polygon with 16 coordinate pairs
    let poly = artifact
        .svg
        .lines()
        .find(|l| l.contains("<polygon"))
        .expect("ellipse becomes polygon");
    let pairs = poly.matches(',').count();
    assert_eq!(pairs, 16);
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
    assert!(matches!(
        err,
        reciplexa_backend::ExportError::Planning(PlanningError::CapabilityMismatch(_))
    ));
}
