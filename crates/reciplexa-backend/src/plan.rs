//! Backend planning IR.

use reciplexa_visual_ir::{RenderDocument, RenderNode, RenderNodeId};

use crate::capability::{check_capability_profile, BackendCapability, PlanningError};
use crate::profile::OutputProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendTarget {
    Svg,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Representation {
    SvgCircle,
    SvgRect,
    SvgPolygon,
    SvgText,
    SvgPath,
    SvgImage,
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
        RenderNode::Text { .. } => Representation::SvgText,
        RenderNode::Path { .. } => Representation::SvgPath,
        RenderNode::Image { .. } => Representation::SvgImage,
    };
    (repr, None)
}
