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
