use reciplexa_backend::emit::EmitError;
use reciplexa_backend::plan::*;
use reciplexa_backend::preview::*;
use reciplexa_backend::profile::OutputProfile;
use reciplexa_scene::Color;
use reciplexa_visual_ir::render::*;

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

#[test]
fn preview_rejects_missing_page() {
    let mut plan = base_plan(vec![planned(1, Representation::SvgRect)]);
    plan.nodes[0].page_index = 3;
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![RenderNode::Rect {
                id: RenderNodeId::new(1),
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 1.0,
                height_mm: 1.0,
                fill: Color::BLACK,
                stroke_width_mm: None,
                alpha: 1.0,
            }],
        }],
    };
    let err = emit_preview_from_plan(&plan, &render).unwrap_err();
    assert!(matches!(
        err,
        EmitError::MissingRenderNode { render_id: 1 }
    ));
}

