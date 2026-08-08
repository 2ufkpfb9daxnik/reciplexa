use reciplexa_backend::*;
use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::{Color, Document, Page, PaperSize, Rect, Shape};
use reciplexa_visual_ir::NodeSourceHint;
use reciplexa_visual_ir::ProvenanceMap;

#[test]
fn exports_simple_rect() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 10.0,
            y_mm: 20.0,
            width_mm: 30.0,
            height_mm: 40.0,
            fill: Color::RED,
        })],
    });
    let artifact = export_scene_to_svg(
        &doc,
        &BackendCapability::svg_default(),
        &OutputProfile::svg_default(),
    )
    .unwrap();
    assert!(artifact.svg.contains("rpx-"));
    assert!(artifact.svg.contains("<rect"));
    assert!(!artifact.provenance.is_empty());
}

#[test]
fn preview_export_without_hints() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 1.0,
            y_mm: 2.0,
            width_mm: 3.0,
            height_mm: 4.0,
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
}

#[test]
fn preview_export_tracks_provenance_hints() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 1.0,
            y_mm: 2.0,
            width_mm: 3.0,
            height_mm: 4.0,
            fill: Color::BLACK,
        })],
    });
    let hints = vec![Some(NodeSourceHint {
        stable_node_id: StableNodeId::new(7),
        source_byte_start: 10,
        source_byte_end: 30,
    })];
    let artifact = export_scene_to_preview_with_hints(
        &doc,
        &BackendCapability::svg_default(),
        &OutputProfile::svg_default(),
        &hints,
    )
    .unwrap();
    assert_eq!(artifact.drawables.len(), 1);
    assert_eq!(
        artifact.provenance[0].stable_node_id,
        Some(StableNodeId::new(7))
    );
    assert_eq!(artifact.provenance[0].source_byte_start, Some(10));
}

#[test]
fn export_maps_render_validation_error() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Circle(reciplexa_scene::Circle {
            x_mm: 0.0,
            y_mm: 0.0,
            radius_mm: 0.0,
            fill: Color::BLACK,
        })],
    });
    let err = export_scene_to_svg(
        &doc,
        &BackendCapability::svg_default(),
        &OutputProfile::svg_default(),
    )
    .unwrap_err();
    assert!(matches!(err, ExportError::Render(_)));
}

#[test]
fn preview_maps_render_validation_error() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Circle(reciplexa_scene::Circle {
            x_mm: 0.0,
            y_mm: 0.0,
            radius_mm: 0.0,
            fill: Color::BLACK,
        })],
    });
    let err = export_scene_to_preview(
        &doc,
        &BackendCapability::svg_default(),
        &OutputProfile::svg_default(),
    )
    .unwrap_err();
    assert!(matches!(err, ExportError::Render(_)));
}

#[test]
fn export_maps_planning_error_when_images_disabled() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(reciplexa_scene::Image {
            path: "a.png".into(),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 10.0,
            height_mm: 10.0,
        })],
    });
    let mut cap = BackendCapability::svg_default();
    cap.svg_images = false;
    let err = export_scene_to_svg(&doc, &cap, &OutputProfile::svg_default()).unwrap_err();
    assert!(matches!(
        err,
        ExportError::Planning(PlanningError::CapabilityMismatch(_))
    ));
}

#[test]
fn preview_maps_planning_error_when_images_disabled() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(reciplexa_scene::Image {
            path: "a.png".into(),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 10.0,
            height_mm: 10.0,
        })],
    });
    let mut cap = BackendCapability::svg_default();
    cap.svg_images = false;
    let err = export_scene_to_preview(&doc, &cap, &OutputProfile::svg_default()).unwrap_err();
    assert!(matches!(
        err,
        ExportError::Planning(PlanningError::CapabilityMismatch(_))
    ));
}

#[test]
fn finalize_svg_maps_emit_error_on_plan_mismatch() {
    use reciplexa_backend::plan::{plan_svg, Representation};
    use reciplexa_scene::Circle;
    use reciplexa_visual_ir::{
        lower_scene_document_with_options, validate_render_document, LowerOptions,
    };

    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Circle(Circle {
            x_mm: 1.0,
            y_mm: 2.0,
            radius_mm: 3.0,
            fill: Color::BLACK,
        })],
    });
    let profile = OutputProfile::svg_default();
    let cap = BackendCapability::svg_default();
    let options = LowerOptions {
        ellipse_sides: profile.ellipse_sides,
        provenance_hints: vec![],
    };
    let (render, prov) = lower_scene_document_with_options(&doc, &options);
    validate_render_document(&render).unwrap();
    let mut plan = plan_svg(&render, &cap, &profile).unwrap();
    plan.nodes[0].representation = Representation::SvgRect;
    let err = finalize_svg_export(&plan, &render, &prov).unwrap_err();
    assert!(matches!(err, ExportError::Emit(_)));
}

#[test]
fn finalize_preview_maps_emit_error_on_plan_mismatch() {
    use reciplexa_backend::plan::{plan_preview, Representation};
    use reciplexa_scene::Circle;
    use reciplexa_visual_ir::{
        lower_scene_document_with_options, validate_render_document, LowerOptions,
    };

    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Circle(Circle {
            x_mm: 1.0,
            y_mm: 2.0,
            radius_mm: 3.0,
            fill: Color::BLACK,
        })],
    });
    let profile = OutputProfile::svg_default();
    let cap = BackendCapability::svg_default();
    let options = LowerOptions {
        ellipse_sides: profile.ellipse_sides,
        provenance_hints: vec![],
    };
    let (render, prov) = lower_scene_document_with_options(&doc, &options);
    validate_render_document(&render).unwrap();
    let mut plan = plan_preview(&render, &cap, &profile).unwrap();
    plan.nodes[0].representation = Representation::SvgRect;
    let err = finalize_preview_export(&plan, &render, &prov).unwrap_err();
    assert!(matches!(err, ExportError::Emit(_)));
}

#[test]
fn finalize_svg_maps_artifact_error_on_non_finite_geometry() {
    use reciplexa_backend::plan::plan_svg;
    use reciplexa_visual_ir::render::{RenderDocument, RenderNode, RenderNodeId, RenderPage};

    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![RenderNode::Rect {
                id: RenderNodeId::new(1),
                x_mm: f64::NAN,
                y_mm: 0.0,
                width_mm: 10.0,
                height_mm: 10.0,
                fill: Color::BLACK,
                stroke_width_mm: None,
                alpha: 1.0,
            }],
        }],
    };
    let plan = plan_svg(
        &render,
        &BackendCapability::svg_default(),
        &OutputProfile::svg_default(),
    )
    .unwrap();
    let err = finalize_svg_export(&plan, &render, &ProvenanceMap::default()).unwrap_err();
    assert!(matches!(err, ExportError::Artifact(_)));
}

#[test]
fn export_maps_artifact_validation_error() {
    let bad_svg = r#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10">
  <rect x="Infinity" y="0" width="1" height="1"/>
</svg>"#;
    let artifact_err = validate_svg_artifact(bad_svg).unwrap_err();
    assert!(matches!(
        ExportError::Artifact(artifact_err),
        ExportError::Artifact(ArtifactValidationError::ContainsNaN)
    ));
}
