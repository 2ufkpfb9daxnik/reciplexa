//! End-to-end export pipeline: scene → render → plan → emit → verify.

use reciplexa_scene::Document;
use reciplexa_visual_ir::{
    lower_scene_document, validate_render_document, ArtifactProvenance, ProvenanceMap,
    RenderValidationError,
};

use crate::capability::{BackendCapability, PlanningError};
use crate::emit::{emit_svg_from_plan, EmitError};
use crate::plan::plan_svg;
use crate::profile::OutputProfile;
use crate::verify::{validate_svg_artifact, ArtifactValidationError};

#[derive(Debug, Clone, PartialEq)]
pub struct VerifiedSvgArtifact {
    pub svg: String,
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
    let (render, prov) = lower_scene_document(doc);
    validate_render_document(&render).map_err(ExportError::Render)?;
    let plan = plan_svg(&render, cap, profile).map_err(ExportError::Planning)?;
    let svg = emit_svg_from_plan(&plan, &render).map_err(ExportError::Emit)?;
    validate_svg_artifact(&svg).map_err(ExportError::Artifact)?;
    let provenance = build_artifact_provenance(&plan, &prov);
    Ok(VerifiedSvgArtifact { svg, provenance })
}

fn build_artifact_provenance(
    plan: &crate::plan::BackendPlan,
    _prov: &ProvenanceMap,
) -> Vec<ArtifactProvenance> {
    plan.nodes
        .iter()
        .map(|n| ArtifactProvenance {
            render_id: n.render_id,
            artifact_element_id: n.artifact_element_id.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
