use reciplexa_backend::capability::*;
use reciplexa_backend::plan::*;
use reciplexa_backend::profile::OutputProfile;
use reciplexa_scene::Color;
use reciplexa_visual_ir::render::*;

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

#[test]
fn rejects_profile_ellipse_sides_out_of_range() {
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![],
        }],
    };
    let mut profile = OutputProfile::svg_default();
    profile.ellipse_sides = 4;
    let err = plan_svg(&render, &BackendCapability::svg_default(), &profile).unwrap_err();
    assert!(matches!(err, PlanningError::ProfileViolation(_)));
}

