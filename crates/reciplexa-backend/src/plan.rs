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
        target: BackendTarget::Svg,
        profile: profile.clone(),
        nodes,
    })
}

fn representation_for(node: &RenderNode, _profile: &OutputProfile) -> (Representation, Option<u32>) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::Color;
    use reciplexa_visual_ir::render::{RenderPage, RenderNode};

    #[test]
    fn plans_rect_node() {
        let render = RenderDocument {
            pages: vec![RenderPage {
                width_mm: 210.0,
                height_mm: 297.0,
                nodes: vec![RenderNode::Rect {
                    id: RenderNodeId::new(1),
                    x_mm: 1.0,
                    y_mm: 2.0,
                    width_mm: 3.0,
                    height_mm: 4.0,
                    fill: Color::BLACK,
                    stroke_width_mm: None,
                    alpha: 1.0,
                }],
            }],
        };
        let plan = plan_svg(&render, &BackendCapability::svg_default(), &OutputProfile::svg_default()).unwrap();
        assert_eq!(plan.nodes.len(), 1);
        assert_eq!(plan.nodes[0].representation, Representation::SvgRect);
    }
}
