//! Backend planning IR.

use reciplexa_visual_ir::{RenderDocument, RenderNode, RenderNodeId};

use crate::capability::{
    check_capability_profile, BackendCapability, BackendFamily, PlanningError,
};
use crate::loss::{LossReport, OutputLoss, OutputLossKind};
use crate::profile::OutputProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendTarget {
    Svg,
    Preview,
    Raster,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Representation {
    SvgCircle,
    SvgRect,
    SvgPolygon,
    SvgText,
    SvgPath,
    SvgImage,
    RasterCircle,
    RasterPolygon,
    RasterPath,
    /// Text omitted with an explicit Loss (never silent).
    RasterTextOmit,
    /// Image omitted with an explicit Loss (never silent).
    RasterImageOmit,
}

/// Per-node emission decision recorded before emission.
#[derive(Debug, Clone, PartialEq)]
pub struct PlannedNode {
    pub render_id: RenderNodeId,
    pub page_index: usize,
    pub representation: Representation,
    pub artifact_element_id: String,
    pub font_family: Option<String>,
    pub polygon_sides: Option<u32>,
}

/// Complete backend plan — emitter must not deviate from this.
#[derive(Debug, Clone, PartialEq)]
pub struct BackendPlan {
    pub target: BackendTarget,
    pub profile: OutputProfile,
    pub nodes: Vec<PlannedNode>,
    /// Planned losses (Raster always records Preview/Final disposition; never silent).
    pub losses: LossReport,
}

/// Build a planning IR from validated render document.
pub fn plan_svg(
    render: &RenderDocument,
    cap: &BackendCapability,
    profile: &OutputProfile,
) -> Result<BackendPlan, PlanningError> {
    plan_for_target(render, cap, profile, BackendTarget::Svg)
}

/// Interactive Preview Backend planning (same node decisions, distinct target).
pub fn plan_preview(
    render: &RenderDocument,
    cap: &BackendCapability,
    profile: &OutputProfile,
) -> Result<BackendPlan, PlanningError> {
    plan_for_target(render, cap, profile, BackendTarget::Preview)
}

/// Raster Image Backend planning with explicit Preview/Final loss recording.
pub fn plan_raster(
    render: &RenderDocument,
    cap: &BackendCapability,
    profile: &OutputProfile,
) -> Result<BackendPlan, PlanningError> {
    if cap.family != BackendFamily::Raster {
        return Err(PlanningError::ProfileViolation(
            "plan_raster requires BackendFamily::Raster".into(),
        ));
    }
    check_capability_profile(cap, profile)?;
    let mut nodes = Vec::new();
    let mut losses = LossReport::empty(profile.kind);
    for (page_index, page) in render.pages.iter().enumerate() {
        for node in &page.nodes {
            // Raster representations are always supported once `family == Raster`
            // (checked above); capability flags only affect planned losses.
            let (representation, planned_loss) = raster_representation_for(node, profile, cap);
            if let Some(loss) = planned_loss {
                losses.push(loss);
            }
            let id = node.id();
            nodes.push(PlannedNode {
                render_id: id,
                page_index,
                representation,
                artifact_element_id: format!("rpx-{}", id.0),
                font_family: None,
                polygon_sides: None,
            });
        }
    }
    Ok(BackendPlan {
        target: BackendTarget::Raster,
        profile: profile.clone(),
        nodes,
        losses,
    })
}

fn plan_for_target(
    render: &RenderDocument,
    cap: &BackendCapability,
    profile: &OutputProfile,
    target: BackendTarget,
) -> Result<BackendPlan, PlanningError> {
    check_capability_profile(cap, profile)?;
    let mut nodes = Vec::new();
    for (page_index, page) in render.pages.iter().enumerate() {
        for node in &page.nodes {
            let (representation, polygon_sides) = representation_for(node, profile);
            if !cap.supports(representation) {
                return Err(PlanningError::CapabilityMismatch(vec![
                    crate::capability::CapabilityMismatch {
                        feature: format!("{representation:?}"),
                        required_by_profile: true,
                    },
                ]));
            }
            let id = node.id();
            nodes.push(PlannedNode {
                render_id: id,
                page_index,
                representation,
                artifact_element_id: format!("rpx-{}", id.0),
                font_family: if matches!(representation, Representation::SvgText) {
                    Some(profile.font_family.clone())
                } else {
                    None
                },
                polygon_sides,
            });
        }
    }
    Ok(BackendPlan {
        target,
        profile: profile.clone(),
        nodes,
        losses: LossReport::empty(profile.kind),
    })
}

fn representation_for(
    node: &RenderNode,
    _profile: &OutputProfile,
) -> (Representation, Option<u32>) {
    let repr = match node {
        RenderNode::Rect { .. } => Representation::SvgRect,
        RenderNode::Circle { .. } => Representation::SvgCircle,
        RenderNode::Polygon { .. } => Representation::SvgPolygon,
        RenderNode::Text { .. } | RenderNode::GlyphRun { .. } => Representation::SvgText,
        RenderNode::Path { .. } => Representation::SvgPath,
        RenderNode::Image { .. } => Representation::SvgImage,
    };
    (repr, None)
}

fn raster_representation_for(
    node: &RenderNode,
    profile: &OutputProfile,
    cap: &BackendCapability,
) -> (Representation, Option<OutputLoss>) {
    match node {
        RenderNode::Circle { .. } => (Representation::RasterCircle, None),
        RenderNode::Rect { .. } | RenderNode::Polygon { .. } => {
            (Representation::RasterPolygon, None)
        }
        RenderNode::Path { .. } => {
            let loss = OutputLoss {
                kind: OutputLossKind::VectorRepresentation,
                disposition: crate::loss::LossDisposition::Warn,
                profile: profile.kind,
                detail: "path lowered to stroked polyline".into(),
            };
            (Representation::RasterPath, Some(loss))
        }
        RenderNode::Text { .. } | RenderNode::GlyphRun { .. } => {
            if cap.raster_text {
                (Representation::RasterTextOmit, None)
            } else {
                let loss = OutputLoss {
                    kind: OutputLossKind::SemanticText,
                    disposition: profile.text_omit_disposition,
                    profile: profile.kind,
                    detail: "text skipped by raster backend".into(),
                };
                (Representation::RasterTextOmit, Some(loss))
            }
        }
        RenderNode::Image { .. } => {
            if cap.raster_images {
                (Representation::RasterImageOmit, None)
            } else {
                let loss = OutputLoss {
                    kind: OutputLossKind::Editability,
                    disposition: profile.image_omit_disposition,
                    profile: profile.kind,
                    detail: "image skipped by raster backend".into(),
                };
                (Representation::RasterImageOmit, Some(loss))
            }
        }
    }
}
