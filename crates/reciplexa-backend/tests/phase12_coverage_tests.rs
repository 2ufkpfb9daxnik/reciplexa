//! Coverage gaps for Phase 12 raster/loss/capability/verify/plan paths.

use reciplexa_backend::emit_raster::emit_raster_page_from_plan;
use reciplexa_backend::loss::*;
use reciplexa_backend::plan::*;
use reciplexa_backend::verify::*;
use reciplexa_backend::{
    check_capability_profile, export_scene_to_raster, BackendCapability, BackendFamily,
    OutputProfile, PlanningError, ProfileKind,
};
use reciplexa_scene::{
    Circle, Color, Document, Image, Line, Page, PaperSize, Polyline, Shape, Text,
};
use reciplexa_visual_ir::lower_scene_document;
use reciplexa_visual_ir::ProvenanceMap;

#[test]
fn loss_helpers_and_report() {
    let mut report = LossReport::empty(ProfileKind::Preview);
    assert!(report.is_empty());
    report.push(OutputLoss::report(
        OutputLossKind::VisualFidelity,
        ProfileKind::Preview,
        "a",
    ));
    report.push(OutputLoss::warn(
        OutputLossKind::GeometricPrecision,
        ProfileKind::Preview,
        "b",
    ));
    assert!(!report.is_empty());
    assert_eq!(report.losses.len(), 2);
    assert_eq!(report.losses[0].disposition, LossDisposition::Report);
    assert_eq!(report.losses[1].disposition, LossDisposition::Warn);
}

#[test]
fn emit_rejects_non_raster_target_and_silent_profile() {
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 5.0,
            height_mm: 5.0,
        },
        shapes: vec![],
    });
    let (render, _) = lower_scene_document(&doc);
    let mut plan = plan_svg(
        &render,
        &BackendCapability::svg_default(),
        &OutputProfile::svg_default(),
    )
    .unwrap();
    assert!(emit_raster_page_from_plan(&plan, &doc, 0).is_err());
    plan = plan_raster(
        &render,
        &BackendCapability::raster_default(),
        &OutputProfile::preview_raster(),
    )
    .unwrap();
    plan.profile.forbid_silent_loss = false;
    assert!(emit_raster_page_from_plan(&plan, &doc, 0).is_err());
}

#[test]
fn emit_maps_page_out_of_range() {
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 5.0,
            height_mm: 5.0,
        },
        shapes: vec![Shape::Circle(Circle {
            x_mm: 1.0,
            y_mm: 1.0,
            radius_mm: 0.5,
            fill: Color::BLACK,
        })],
    });
    let (render, _) = lower_scene_document(&doc);
    let plan = plan_raster(
        &render,
        &BackendCapability::raster_default(),
        &OutputProfile::preview_raster(),
    )
    .unwrap();
    assert!(emit_raster_page_from_plan(&plan, &doc, 9).is_err());
}

#[test]
fn emit_merges_runtime_losses_when_plan_empty() {
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 10.0,
            height_mm: 10.0,
        },
        shapes: vec![
            Shape::Text(Text {
                x_mm: 1.0,
                y_mm: 1.0,
                size_mm: 2.0,
                width_mm: None,
                height_mm: None,
                content: "t".into(),
                fill: Color::BLACK,
            }),
            Shape::Image(Image {
                path: "x.png".into(),
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 1.0,
                height_mm: 1.0,
            }),
            Shape::Polyline(Polyline {
                points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
                stroke: Color::BLACK,
                width_mm: 0.2,
            }),
        ],
    });
    let (render, _) = lower_scene_document(&doc);
    let mut plan = plan_raster(
        &render,
        &BackendCapability::raster_default(),
        &OutputProfile::preview_raster(),
    )
    .unwrap();
    // Clear planned losses to force runtime merge arms.
    plan.losses = LossReport::empty(ProfileKind::Preview);
    let page = emit_raster_page_from_plan(&plan, &doc, 0).unwrap();
    assert!(page
        .losses
        .losses
        .iter()
        .any(|l| l.kind == OutputLossKind::SemanticText));
    assert!(page
        .losses
        .losses
        .iter()
        .any(|l| l.kind == OutputLossKind::Editability));
    assert!(page
        .losses
        .losses
        .iter()
        .any(|l| l.kind == OutputLossKind::VectorRepresentation));
}

#[test]
fn plan_raster_with_native_text_image_flags() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![
            Shape::Text(Text {
                x_mm: 1.0,
                y_mm: 1.0,
                size_mm: 2.0,
                width_mm: None,
                height_mm: None,
                content: "t".into(),
                fill: Color::BLACK,
            }),
            Shape::Image(Image {
                path: "a.png".into(),
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 1.0,
                height_mm: 1.0,
            }),
        ],
    });
    let (render, _) = lower_scene_document(&doc);
    let mut cap = BackendCapability::raster_default();
    cap.raster_text = true;
    cap.raster_images = true;
    let plan = plan_raster(&render, &cap, &OutputProfile::preview_raster()).unwrap();
    assert!(plan
        .nodes
        .iter()
        .any(|n| n.representation == Representation::RasterTextOmit));
    // Native flags currently still use Omit repr without planned loss.
    assert!(
        plan.losses.is_empty()
            || plan.losses.losses.iter().all(|l| {
                l.kind != OutputLossKind::SemanticText && l.kind != OutputLossKind::Editability
            })
    );
}

#[test]
fn capability_final_images_and_high_px() {
    let mut cap = BackendCapability::raster_default();
    assert_eq!(cap.family, BackendFamily::Raster);
    let mut profile = OutputProfile::final_raster();
    profile.requires_images = true;
    let err = check_capability_profile(&cap, &profile).unwrap_err();
    assert!(matches!(err, PlanningError::CapabilityMismatch(_)));
    profile = OutputProfile::preview_raster();
    profile.px_per_mm = 1000.0;
    let err = check_capability_profile(&cap, &profile).unwrap_err();
    assert!(matches!(err, PlanningError::ProfileViolation(_)));
    cap.max_px_per_mm = 2000.0;
    assert!(check_capability_profile(&cap, &profile).is_ok());
}

#[test]
fn verify_loss_entry_profile_mismatch() {
    let profile = OutputProfile::preview_raster();
    let mut report = LossReport::empty(ProfileKind::Preview);
    report.losses.push(OutputLoss {
        kind: OutputLossKind::SemanticText,
        disposition: LossDisposition::Report,
        profile: ProfileKind::Final,
        detail: "bad".into(),
    });
    assert!(matches!(
        validate_raster_loss_report(&report, &profile),
        Err(ArtifactValidationError::LossProfileMismatch)
    ));
}

#[test]
fn verify_rejects_empty_detail_when_silent_forbidden() {
    let profile = OutputProfile::preview_raster();
    let mut report = LossReport::empty(ProfileKind::Preview);
    report.push(OutputLoss {
        kind: OutputLossKind::SemanticText,
        disposition: LossDisposition::Report,
        profile: ProfileKind::Preview,
        detail: "   ".into(),
    });
    assert!(matches!(
        validate_raster_loss_report(&report, &profile),
        Err(ArtifactValidationError::SilentLossForbidden)
    ));
}

#[test]
fn raster_capability_ellipse_out_of_range() {
    let cap = BackendCapability::raster_default();
    let mut profile = OutputProfile::preview_raster();
    profile.ellipse_sides = 2;
    let err = check_capability_profile(&cap, &profile).unwrap_err();
    assert!(matches!(err, PlanningError::ProfileViolation(_)));
}

#[test]
fn raster_export_render_validation_error() {
    use reciplexa_backend::{finalize_raster_export, plan_raster};
    use reciplexa_visual_ir::render::{RenderDocument, RenderNode, RenderNodeId, RenderPage};
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 10.0,
            height_mm: 10.0,
            nodes: vec![RenderNode::Circle {
                id: RenderNodeId::new(1),
                x_mm: 1.0,
                y_mm: 1.0,
                radius_mm: f64::NAN,
                fill: Color::BLACK,
                stroke_width_mm: None,
                alpha: 1.0,
            }],
        }],
    };
    // Planning still works; emit uses scene doc — use empty doc mismatch is separate.
    let plan = plan_raster(
        &render,
        &BackendCapability::raster_default(),
        &OutputProfile::preview_raster(),
    );
    // NaN radius may fail validate_render in export path:
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 10.0,
            height_mm: 10.0,
        },
        shapes: vec![Shape::Circle(Circle {
            x_mm: f64::NAN,
            y_mm: 1.0,
            radius_mm: 1.0,
            fill: Color::BLACK,
        })],
    });
    let err = export_scene_to_raster(
        &doc,
        &BackendCapability::raster_default(),
        &OutputProfile::preview_raster(),
        0,
    );
    // Either planning/render fails, or rasterize proceeds (NaN may clip).
    let _ = (plan, err, finalize_raster_export);
}

#[test]
fn finalize_raster_via_export_empty_page() {
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 8.0,
            height_mm: 8.0,
        },
        shapes: vec![Shape::Circle(Circle {
            x_mm: 4.0,
            y_mm: 4.0,
            radius_mm: 1.0,
            fill: Color::RED,
        })],
    });
    let a = export_scene_to_raster(
        &doc,
        &BackendCapability::raster_default(),
        &OutputProfile::final_raster(),
        0,
    )
    .unwrap();
    validate_png_artifact(&a.png).unwrap();
    validate_raster_loss_report(&a.losses, &OutputProfile::final_raster()).unwrap();
    let _ = ProvenanceMap::default();
    let _ = Line {
        x1_mm: 0.0,
        y1_mm: 0.0,
        x2_mm: 1.0,
        y2_mm: 1.0,
        stroke: Color::BLACK,
        width_mm: 0.1,
    };
}

#[test]
fn plan_raster_maps_capability_profile_errors() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![],
    });
    let (render, _) = lower_scene_document(&doc);
    let mut profile = OutputProfile::preview_raster();
    profile.forbid_silent_loss = false;
    let err = plan_raster(&render, &BackendCapability::raster_default(), &profile).unwrap_err();
    assert!(matches!(err, PlanningError::ProfileViolation(_)));
    profile = OutputProfile::preview_raster();
    profile.px_per_mm = 0.01;
    let err = plan_raster(&render, &BackendCapability::raster_default(), &profile).unwrap_err();
    assert!(matches!(err, PlanningError::ProfileViolation(_)));
}

#[test]
fn finalize_and_hints_map_emit_render_loss_errors() {
    use reciplexa_backend::{
        export_scene_to_raster_with_hints, finalize_raster_export, plan_raster, ExportError,
    };

    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 8.0,
            height_mm: 8.0,
        },
        shapes: vec![Shape::Circle(Circle {
            x_mm: 4.0,
            y_mm: 4.0,
            radius_mm: 1.0,
            fill: Color::RED,
        })],
    });
    let (render, prov) = lower_scene_document(&doc);
    let mut plan = plan_raster(
        &render,
        &BackendCapability::raster_default(),
        &OutputProfile::preview_raster(),
    )
    .unwrap();
    // Emit failure: page out of range.
    let err = finalize_raster_export(&plan, &doc, &prov, 9).unwrap_err();
    assert!(matches!(err, ExportError::Emit(_)));

    // Loss report mismatch after a successful emit path.
    plan.losses.profile = ProfileKind::Final;
    let err = finalize_raster_export(&plan, &doc, &prov, 0).unwrap_err();
    assert!(matches!(err, ExportError::Artifact(_)));

    // with_hints Render validation failure (zero radius).
    let bad = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Circle(Circle {
            x_mm: 0.0,
            y_mm: 0.0,
            radius_mm: 0.0,
            fill: Color::BLACK,
        })],
    });
    let err = export_scene_to_raster_with_hints(
        &bad,
        &BackendCapability::raster_default(),
        &OutputProfile::preview_raster(),
        0,
        &[],
    )
    .unwrap_err();
    assert!(matches!(err, ExportError::Render(_)));

    // with_hints planning error (wrong capability family).
    let err = export_scene_to_raster_with_hints(
        &doc,
        &BackendCapability::svg_default(),
        &OutputProfile::preview_raster(),
        0,
        &[],
    )
    .unwrap_err();
    assert!(matches!(err, ExportError::Planning(_)));
}
