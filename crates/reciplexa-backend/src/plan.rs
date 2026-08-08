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

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::Color;
    use reciplexa_visual_ir::render::{RenderNode, RenderNodeId, RenderPage};

    fn all_kinds_render() -> RenderDocument {
        RenderDocument {
            pages: vec![RenderPage {
                width_mm: 210.0,
                height_mm: 297.0,
                nodes: vec![
                    RenderNode::Rect {
                        id: RenderNodeId::new(1),
                        x_mm: 0.0,
                        y_mm: 0.0,
                        width_mm: 1.0,
                        height_mm: 1.0,
                        fill: Color::BLACK,
                        stroke_width_mm: None,
                        alpha: 1.0,
                    },
                    RenderNode::Circle {
                        id: RenderNodeId::new(2),
                        x_mm: 0.0,
                        y_mm: 0.0,
                        radius_mm: 1.0,
                        fill: Color::BLACK,
                        stroke_width_mm: None,
                        alpha: 1.0,
                    },
                    RenderNode::Polygon {
                        id: RenderNodeId::new(3),
                        points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
                        fill: Color::BLACK,
                        stroke_width_mm: None,
                        alpha: 1.0,
                    },
                    RenderNode::Text {
                        id: RenderNodeId::new(4),
                        x_mm: 0.0,
                        y_mm: 0.0,
                        size_mm: 12.0,
                        width_mm: 10.0,
                        height_mm: 10.0,
                        rotation_deg: 0.0,
                        content: "x".into(),
                        fill: Color::BLACK,
                        alpha: 1.0,
                    },
                    RenderNode::Path {
                        id: RenderNodeId::new(5),
                        points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
                        stroke: Color::BLACK,
                        width_mm: 1.0,
                        closed: false,
                        alpha: 1.0,
                    },
                    RenderNode::Image {
                        id: RenderNodeId::new(6),
                        path: "a.png".into(),
                        corners_mm: [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
                        alpha: 1.0,
                    },
                ],
            }],
        }
    }

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
        let plan = plan_svg(
            &render,
            &BackendCapability::svg_default(),
            &OutputProfile::svg_default(),
        )
        .unwrap();
        assert_eq!(plan.nodes.len(), 1);
        assert_eq!(plan.nodes[0].representation, Representation::SvgRect);
    }

    #[test]
    fn plans_all_node_kinds() {
        let render = all_kinds_render();
        let plan = plan_svg(
            &render,
            &BackendCapability::svg_default(),
            &OutputProfile::svg_default(),
        )
        .unwrap();
        assert_eq!(plan.nodes.len(), 6);
        assert_eq!(plan.nodes[0].representation, Representation::SvgRect);
        assert_eq!(plan.nodes[1].representation, Representation::SvgCircle);
        assert_eq!(plan.nodes[2].representation, Representation::SvgPolygon);
        assert_eq!(plan.nodes[3].representation, Representation::SvgText);
        assert_eq!(plan.nodes[4].representation, Representation::SvgPath);
        assert_eq!(plan.nodes[5].representation, Representation::SvgImage);
    }

    #[test]
    fn plan_preview_sets_preview_target() {
        let render = all_kinds_render();
        let plan = plan_preview(
            &render,
            &BackendCapability::svg_default(),
            &OutputProfile::svg_default(),
        )
        .unwrap();
        assert_eq!(plan.target, BackendTarget::Preview);
    }

    #[test]
    fn rejects_image_when_capability_disallows() {
        let render = RenderDocument {
            pages: vec![RenderPage {
                width_mm: 210.0,
                height_mm: 297.0,
                nodes: vec![RenderNode::Image {
                    id: RenderNodeId::new(1),
                    path: "a.png".into(),
                    corners_mm: [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
                    alpha: 1.0,
                }],
            }],
        };
        let mut cap = BackendCapability::svg_default();
        cap.svg_images = false;
        let err = plan_svg(&render, &cap, &OutputProfile::svg_default()).unwrap_err();
        assert!(matches!(err, PlanningError::CapabilityMismatch(_)));
    }
}
