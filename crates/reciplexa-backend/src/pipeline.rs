//! End-to-end export pipeline: scene → render → plan → emit → verify.

use reciplexa_scene::Document;
use reciplexa_visual_ir::{
    lower_scene_document_with_options, validate_render_document, ArtifactProvenance, LowerOptions,
    NodeSourceHint, ProvenanceMap, RenderDocument, RenderValidationError,
};

use crate::capability::{BackendCapability, PlanningError};
use crate::emit::{emit_svg_from_plan, EmitError};
use crate::plan::{plan_preview, plan_svg, BackendPlan};
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
    finalize_svg_export(&plan, &render, &prov)
}

/// Emit + validate SVG from an existing plan (exposed for mismatch / artifact tests).
pub fn finalize_svg_export(
    plan: &BackendPlan,
    render: &RenderDocument,
    prov: &ProvenanceMap,
) -> Result<VerifiedSvgArtifact, ExportError> {
    let svg = emit_svg_from_plan(plan, render).map_err(ExportError::Emit)?;
    validate_svg_artifact(&svg).map_err(ExportError::Artifact)?;
    let provenance = build_artifact_provenance(plan, prov);
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
    finalize_preview_export(&plan, &render, &prov)
}

/// Emit preview drawables from an existing plan (exposed for mismatch tests).
pub fn finalize_preview_export(
    plan: &BackendPlan,
    render: &RenderDocument,
    prov: &ProvenanceMap,
) -> Result<VerifiedPreviewArtifact, ExportError> {
    let drawables = emit_preview_from_plan(plan, render).map_err(ExportError::Emit)?;
    let provenance = build_artifact_provenance(plan, prov);
    Ok(VerifiedPreviewArtifact {
        drawables,
        provenance,
    })
}

fn build_artifact_provenance(
    plan: &BackendPlan,
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
