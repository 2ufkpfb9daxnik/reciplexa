//! Interactive Preview Backend emission (Phase 7 §9.2 item 10).

use reciplexa_scene::Color;
use reciplexa_visual_ir::{RenderDocument, RenderNode};

use crate::emit::EmitError;
use crate::plan::{BackendPlan, Representation};

/// Plan-driven drawable for interactive preview (not egui-specific).
#[derive(Debug, Clone, PartialEq)]
pub enum PreviewDrawable {
    Rect {
        artifact_element_id: String,
        x_mm: f64,
        y_mm: f64,
        width_mm: f64,
        height_mm: f64,
        fill: Color,
        stroke_width_mm: Option<f64>,
        alpha: f64,
    },
    Circle {
        artifact_element_id: String,
        x_mm: f64,
        y_mm: f64,
        radius_mm: f64,
        fill: Color,
        stroke_width_mm: Option<f64>,
        alpha: f64,
    },
    Polygon {
        artifact_element_id: String,
        points_mm: Vec<(f64, f64)>,
        fill: Color,
        stroke_width_mm: Option<f64>,
        alpha: f64,
    },
    Text {
        artifact_element_id: String,
        x_mm: f64,
        y_mm: f64,
        size_mm: f64,
        rotation_deg: f64,
        content: String,
        fill: Color,
        alpha: f64,
    },
    Path {
        artifact_element_id: String,
        points_mm: Vec<(f64, f64)>,
        stroke: Color,
        width_mm: f64,
        closed: bool,
        alpha: f64,
    },
    Image {
        artifact_element_id: String,
        path: String,
        corners_mm: [(f64, f64); 4],
        alpha: f64,
    },
}

/// Emit preview drawables strictly from a plan.
pub fn emit_preview_from_plan(
    plan: &BackendPlan,
    render: &RenderDocument,
) -> Result<Vec<PreviewDrawable>, EmitError> {
    let mut out = Vec::new();
    for pn in &plan.nodes {
        let page = render
            .pages
            .get(pn.page_index)
            .ok_or(EmitError::MissingRenderNode {
                render_id: pn.render_id.0,
            })?;
        let node = page.nodes.iter().find(|n| n.id() == pn.render_id).ok_or(
            EmitError::MissingRenderNode {
                render_id: pn.render_id.0,
            },
        )?;
        out.push(drawable_for(
            pn.representation,
            &pn.artifact_element_id,
            node,
        )?);
    }
    Ok(out)
}

fn drawable_for(
    repr: Representation,
    artifact_element_id: &str,
    node: &RenderNode,
) -> Result<PreviewDrawable, EmitError> {
    let id = artifact_element_id.to_string();
    match (repr, node) {
        (
            Representation::SvgRect,
            RenderNode::Rect {
                x_mm,
                y_mm,
                width_mm,
                height_mm,
                fill,
                stroke_width_mm,
                alpha,
                ..
            },
        ) => Ok(PreviewDrawable::Rect {
            artifact_element_id: id,
            x_mm: *x_mm,
            y_mm: *y_mm,
            width_mm: *width_mm,
            height_mm: *height_mm,
            fill: *fill,
            stroke_width_mm: *stroke_width_mm,
            alpha: *alpha,
        }),
        (
            Representation::SvgCircle,
            RenderNode::Circle {
                x_mm,
                y_mm,
                radius_mm,
                fill,
                stroke_width_mm,
                alpha,
                ..
            },
        ) => Ok(PreviewDrawable::Circle {
            artifact_element_id: id,
            x_mm: *x_mm,
            y_mm: *y_mm,
            radius_mm: *radius_mm,
            fill: *fill,
            stroke_width_mm: *stroke_width_mm,
            alpha: *alpha,
        }),
        (
            Representation::SvgPolygon,
            RenderNode::Polygon {
                points_mm,
                fill,
                stroke_width_mm,
                alpha,
                ..
            },
        ) => Ok(PreviewDrawable::Polygon {
            artifact_element_id: id,
            points_mm: points_mm.clone(),
            fill: *fill,
            stroke_width_mm: *stroke_width_mm,
            alpha: *alpha,
        }),
        (
            Representation::SvgText,
            RenderNode::Text {
                x_mm,
                y_mm,
                size_mm,
                rotation_deg,
                content,
                fill,
                alpha,
                ..
            },
        ) => Ok(PreviewDrawable::Text {
            artifact_element_id: id,
            x_mm: *x_mm,
            y_mm: *y_mm,
            size_mm: *size_mm,
            rotation_deg: *rotation_deg,
            content: content.clone(),
            fill: *fill,
            alpha: *alpha,
        }),
        (
            Representation::SvgPath,
            RenderNode::Path {
                points_mm,
                stroke,
                width_mm,
                closed,
                alpha,
                ..
            },
        ) => Ok(PreviewDrawable::Path {
            artifact_element_id: id,
            points_mm: points_mm.clone(),
            stroke: *stroke,
            width_mm: *width_mm,
            closed: *closed,
            alpha: *alpha,
        }),
        (
            Representation::SvgImage,
            RenderNode::Image {
                path,
                corners_mm,
                alpha,
                ..
            },
        ) => Ok(PreviewDrawable::Image {
            artifact_element_id: id,
            path: path.clone(),
            corners_mm: *corners_mm,
            alpha: *alpha,
        }),
        (repr, _) => Err(EmitError::PlanNodeMismatch {
            render_id: node.id().0,
            representation: format!("{repr:?}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::Color;
    use reciplexa_visual_ir::render::{RenderDocument, RenderNode, RenderNodeId, RenderPage};

    use crate::plan::{BackendPlan, BackendTarget, PlannedNode, Representation};
    use crate::profile::OutputProfile;

    fn base_plan(nodes: Vec<PlannedNode>) -> BackendPlan {
        BackendPlan {
            target: BackendTarget::Preview,
            profile: OutputProfile::svg_default(),
            nodes,
        }
    }

    fn planned(id: u64, repr: Representation) -> PlannedNode {
        PlannedNode {
            render_id: RenderNodeId::new(id),
            page_index: 0,
            representation: repr,
            artifact_element_id: format!("rpx-{id}"),
            font_family: None,
            polygon_sides: None,
        }
    }

    fn single_node(render_node: RenderNode, repr: Representation) -> (BackendPlan, RenderDocument) {
        let id = render_node.id();
        (
            base_plan(vec![planned(id.0, repr)]),
            RenderDocument {
                pages: vec![RenderPage {
                    width_mm: 210.0,
                    height_mm: 297.0,
                    nodes: vec![render_node],
                }],
            },
        )
    }

    #[test]
    fn preview_rect_drawable() {
        let (plan, render) = single_node(
            RenderNode::Rect {
                id: RenderNodeId::new(1),
                x_mm: 1.0,
                y_mm: 2.0,
                width_mm: 3.0,
                height_mm: 4.0,
                fill: Color::RED,
                stroke_width_mm: Some(0.5),
                alpha: 1.0,
            },
            Representation::SvgRect,
        );
        let out = emit_preview_from_plan(&plan, &render).unwrap();
        assert!(matches!(out[0], PreviewDrawable::Rect { .. }));
    }

    #[test]
    fn preview_circle_drawable() {
        let (plan, render) = single_node(
            RenderNode::Circle {
                id: RenderNodeId::new(1),
                x_mm: 0.0,
                y_mm: 0.0,
                radius_mm: 5.0,
                fill: Color::BLACK,
                stroke_width_mm: None,
                alpha: 1.0,
            },
            Representation::SvgCircle,
        );
        let out = emit_preview_from_plan(&plan, &render).unwrap();
        assert!(matches!(out[0], PreviewDrawable::Circle { .. }));
    }

    #[test]
    fn preview_polygon_drawable() {
        let (plan, render) = single_node(
            RenderNode::Polygon {
                id: RenderNodeId::new(1),
                points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
                fill: Color::BLACK,
                stroke_width_mm: None,
                alpha: 1.0,
            },
            Representation::SvgPolygon,
        );
        let out = emit_preview_from_plan(&plan, &render).unwrap();
        assert!(matches!(out[0], PreviewDrawable::Polygon { .. }));
    }

    #[test]
    fn preview_text_drawable() {
        let (plan, render) = single_node(
            RenderNode::Text {
                id: RenderNodeId::new(1),
                x_mm: 0.0,
                y_mm: 0.0,
                size_mm: 12.0,
                width_mm: 10.0,
                height_mm: 10.0,
                rotation_deg: 15.0,
                content: "hi".into(),
                fill: Color::BLACK,
                alpha: 1.0,
            },
            Representation::SvgText,
        );
        let out = emit_preview_from_plan(&plan, &render).unwrap();
        assert!(matches!(out[0], PreviewDrawable::Text { .. }));
    }

    #[test]
    fn preview_path_drawable() {
        let (plan, render) = single_node(
            RenderNode::Path {
                id: RenderNodeId::new(1),
                points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
                stroke: Color::BLUE,
                width_mm: 1.0,
                closed: true,
                alpha: 1.0,
            },
            Representation::SvgPath,
        );
        let out = emit_preview_from_plan(&plan, &render).unwrap();
        assert!(matches!(out[0], PreviewDrawable::Path { closed: true, .. }));
    }

    #[test]
    fn preview_image_drawable() {
        let (plan, render) = single_node(
            RenderNode::Image {
                id: RenderNodeId::new(1),
                path: "a.png".into(),
                corners_mm: [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
                alpha: 0.5,
            },
            Representation::SvgImage,
        );
        let out = emit_preview_from_plan(&plan, &render).unwrap();
        assert!(matches!(out[0], PreviewDrawable::Image { .. }));
    }

    #[test]
    fn preview_rejects_mismatch() {
        let (plan, render) = single_node(
            RenderNode::Circle {
                id: RenderNodeId::new(1),
                x_mm: 0.0,
                y_mm: 0.0,
                radius_mm: 1.0,
                fill: Color::BLACK,
                stroke_width_mm: None,
                alpha: 1.0,
            },
            Representation::SvgRect,
        );
        let err = emit_preview_from_plan(&plan, &render).unwrap_err();
        assert!(matches!(err, EmitError::PlanNodeMismatch { .. }));
    }

    #[test]
    fn preview_rejects_missing_node() {
        let plan = base_plan(vec![planned(42, Representation::SvgRect)]);
        let render = RenderDocument {
            pages: vec![RenderPage {
                width_mm: 210.0,
                height_mm: 297.0,
                nodes: vec![],
            }],
        };
        let err = emit_preview_from_plan(&plan, &render).unwrap_err();
        assert!(matches!(
            err,
            EmitError::MissingRenderNode { render_id: 42 }
        ));
    }
}
