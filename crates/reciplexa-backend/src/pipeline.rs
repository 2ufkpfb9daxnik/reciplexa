//! End-to-end export pipeline: scene → render → plan → emit → verify.

use reciplexa_scene::Document;
use reciplexa_visual_ir::{
    lower_scene_document_with_options, validate_render_document, ArtifactProvenance, LowerOptions,
    NodeSourceHint, ProvenanceMap, RenderValidationError,
};

use crate::capability::{BackendCapability, PlanningError};
use crate::emit::{emit_svg_from_plan, EmitError};
use crate::plan::{plan_preview, plan_svg};
use crate::preview::{emit_preview_from_plan, PreviewDrawable};
use crate::profile::OutputProfile;
use crate::verify::{validate_svg_artifact, ArtifactValidationError};

#[derive(Debug, Clone, PartialEq)]
pub struct VerifiedSvgArtifact {
    pub svg: String,
    pub provenance: Vec<ArtifactProvenance>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifiedPreviewArtifact {
    pub drawables: Vec<PreviewDrawable>,
    pub provenance: Vec<ArtifactProvenance>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportError {
    Render(RenderValidationError),
    Planning(PlanningError),
    Emit(EmitError),
    Artifact(ArtifactValidationError),
}

/// Full Phase 7 export path with planning and artifact validation.
pub fn export_scene_to_svg(
    doc: &Document,
    cap: &BackendCapability,
    profile: &OutputProfile,
) -> Result<VerifiedSvgArtifact, ExportError> {
    export_scene_to_svg_with_hints(doc, cap, profile, &[])
}

/// SVG export with paint-order source provenance hints.
pub fn export_scene_to_svg_with_hints(
    doc: &Document,
    cap: &BackendCapability,
    profile: &OutputProfile,
    hints: &[Option<NodeSourceHint>],
) -> Result<VerifiedSvgArtifact, ExportError> {
    let options = LowerOptions {
        ellipse_sides: profile.ellipse_sides,
        provenance_hints: hints.to_vec(),
    };
    let (render, prov) = lower_scene_document_with_options(doc, &options);
    validate_render_document(&render).map_err(ExportError::Render)?;
    let plan = plan_svg(&render, cap, profile).map_err(ExportError::Planning)?;
    let svg = emit_svg_from_plan(&plan, &render).map_err(ExportError::Emit)?;
    validate_svg_artifact(&svg).map_err(ExportError::Artifact)?;
    let provenance = build_artifact_provenance(&plan, &prov);
    Ok(VerifiedSvgArtifact { svg, provenance })
}

/// Interactive Preview Backend: plan-driven drawable list (no ad-hoc fallback).
pub fn export_scene_to_preview(
    doc: &Document,
    cap: &BackendCapability,
    profile: &OutputProfile,
) -> Result<VerifiedPreviewArtifact, ExportError> {
    export_scene_to_preview_with_hints(doc, cap, profile, &[])
}

/// Preview export with paint-order source provenance hints.
pub fn export_scene_to_preview_with_hints(
    doc: &Document,
    cap: &BackendCapability,
    profile: &OutputProfile,
    hints: &[Option<NodeSourceHint>],
) -> Result<VerifiedPreviewArtifact, ExportError> {
    let options = LowerOptions {
        ellipse_sides: profile.ellipse_sides,
        provenance_hints: hints.to_vec(),
    };
    let (render, prov) = lower_scene_document_with_options(doc, &options);
    validate_render_document(&render).map_err(ExportError::Render)?;
    let plan = plan_preview(&render, cap, profile).map_err(ExportError::Planning)?;
    let drawables = emit_preview_from_plan(&plan, &render).map_err(ExportError::Emit)?;
    let provenance = build_artifact_provenance(&plan, &prov);
    Ok(VerifiedPreviewArtifact {
        drawables,
        provenance,
    })
}

fn build_artifact_provenance(
    plan: &crate::plan::BackendPlan,
    prov: &ProvenanceMap,
) -> Vec<ArtifactProvenance> {
    plan.nodes
        .iter()
        .map(|n| {
            let src = prov.get(n.render_id);
            ArtifactProvenance {
                render_id: n.render_id,
                artifact_element_id: n.artifact_element_id.clone(),
                stable_node_id: src.and_then(|s| s.stable_node_id),
                source_byte_start: src.and_then(|s| s.source_byte_start),
                source_byte_end: src.and_then(|s| s.source_byte_end),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_identity::document::StableNodeId;
    use reciplexa_scene::{Color, Document, Page, PaperSize, Rect, Shape};

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
}
