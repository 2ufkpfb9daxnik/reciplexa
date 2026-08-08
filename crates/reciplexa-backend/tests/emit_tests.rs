use reciplexa_backend::emit::*;
use reciplexa_backend::plan::*;
use reciplexa_backend::profile::OutputProfile;
use reciplexa_scene::Color;
use reciplexa_visual_ir::render::*;

fn base_plan(nodes: Vec<PlannedNode>) -> BackendPlan {
    BackendPlan {
        target: BackendTarget::Svg,
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
        font_family: if matches!(repr, Representation::SvgText) {
            Some("sans-serif".into())
        } else {
            None
        },
        polygon_sides: None,
    }
}

#[test]
fn emits_upright_text_with_counter_flip() {
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![RenderNode::Text {
                id: RenderNodeId::new(1),
                x_mm: 10.0,
                y_mm: 20.0,
                size_mm: 12.0,
                width_mm: 40.0,
                height_mm: 14.0,
                rotation_deg: 0.0,
                content: "hi&<>".into(),
                fill: Color::BLACK,
                alpha: 1.0,
            }],
        }],
    };
    let plan = base_plan(vec![planned(1, Representation::SvgText)]);
    let svg = emit_svg_from_plan(&plan, &render).unwrap();
    assert!(svg.contains("scale(1 -1)"));
    assert!(svg.contains("hi&amp;&lt;&gt;"));
}

#[test]
fn xml_escape_quotes_and_apostrophe() {
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![RenderNode::Text {
                id: RenderNodeId::new(1),
                x_mm: 0.0,
                y_mm: 0.0,
                size_mm: 12.0,
                width_mm: 10.0,
                height_mm: 10.0,
                rotation_deg: 0.0,
                content: r#"say "hi" & 'bye'"#.into(),
                fill: Color::BLACK,
                alpha: 1.0,
            }],
        }],
    };
    let plan = base_plan(vec![planned(1, Representation::SvgText)]);
    let svg = emit_svg_from_plan(&plan, &render).unwrap();
    assert!(svg.contains("&quot;"));
    assert!(svg.contains("&apos;"));
    assert!(svg.contains("&amp;"));
}

#[test]
fn emits_rect_fill_and_stroke() {
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![
                RenderNode::Rect {
                    id: RenderNodeId::new(1),
                    x_mm: 1.0,
                    y_mm: 2.0,
                    width_mm: 3.0,
                    height_mm: 4.0,
                    fill: Color::RED,
                    stroke_width_mm: None,
                    alpha: 1.0,
                },
                RenderNode::Rect {
                    id: RenderNodeId::new(2),
                    x_mm: 5.0,
                    y_mm: 6.0,
                    width_mm: 7.0,
                    height_mm: 8.0,
                    fill: Color::BLUE,
                    stroke_width_mm: Some(0.5),
                    alpha: 0.8,
                },
            ],
        }],
    };
    let plan = base_plan(vec![
        planned(1, Representation::SvgRect),
        planned(2, Representation::SvgRect),
    ]);
    let svg = emit_svg_from_plan(&plan, &render).unwrap();
    assert!(svg.contains("fill=\"#ff0000\""));
    assert!(svg.contains("stroke=\"#0000ff\""));
}

#[test]
fn emits_circle_fill_and_stroke() {
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![
                RenderNode::Circle {
                    id: RenderNodeId::new(1),
                    x_mm: 1.0,
                    y_mm: 2.0,
                    radius_mm: 3.0,
                    fill: Color::BLACK,
                    stroke_width_mm: None,
                    alpha: 1.0,
                },
                RenderNode::Circle {
                    id: RenderNodeId::new(2),
                    x_mm: 4.0,
                    y_mm: 5.0,
                    radius_mm: 6.0,
                    fill: Color::GREEN,
                    stroke_width_mm: Some(1.0),
                    alpha: 1.0,
                },
            ],
        }],
    };
    let plan = base_plan(vec![
        planned(1, Representation::SvgCircle),
        planned(2, Representation::SvgCircle),
    ]);
    let svg = emit_svg_from_plan(&plan, &render).unwrap();
    assert!(svg.contains("<circle"));
    assert!(svg.contains("stroke=\"#00ff00\""));
}

#[test]
fn emits_polygon_fill_and_stroke() {
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![
                RenderNode::Polygon {
                    id: RenderNodeId::new(1),
                    points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
                    fill: Color::BLACK,
                    stroke_width_mm: None,
                    alpha: 1.0,
                },
                RenderNode::Polygon {
                    id: RenderNodeId::new(2),
                    points_mm: vec![(0.0, 0.0), (2.0, 0.0), (1.0, 2.0)],
                    fill: Color::RED,
                    stroke_width_mm: Some(0.2),
                    alpha: 1.0,
                },
            ],
        }],
    };
    let plan = base_plan(vec![
        planned(1, Representation::SvgPolygon),
        planned(2, Representation::SvgPolygon),
    ]);
    let svg = emit_svg_from_plan(&plan, &render).unwrap();
    assert!(svg.contains("<polygon"));
    assert!(svg.contains("stroke=\"#ff0000\""));
}

#[test]
fn emits_path_open_and_closed() {
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![
                RenderNode::Path {
                    id: RenderNodeId::new(1),
                    points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
                    stroke: Color::BLACK,
                    width_mm: 1.0,
                    closed: false,
                    alpha: 1.0,
                },
                RenderNode::Path {
                    id: RenderNodeId::new(2),
                    points_mm: vec![(0.0, 0.0), (2.0, 0.0), (1.0, 2.0)],
                    stroke: Color::BLUE,
                    width_mm: 0.5,
                    closed: true,
                    alpha: 1.0,
                },
            ],
        }],
    };
    let plan = base_plan(vec![
        planned(1, Representation::SvgPath),
        planned(2, Representation::SvgPath),
    ]);
    let svg = emit_svg_from_plan(&plan, &render).unwrap();
    assert!(svg.contains(r#"d="M 0 0 L 1 1""#));
    assert!(svg.contains("Z"));
}

#[test]
fn empty_path_emits_nothing() {
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![RenderNode::Path {
                id: RenderNodeId::new(1),
                points_mm: vec![],
                stroke: Color::BLACK,
                width_mm: 1.0,
                closed: false,
                alpha: 1.0,
            }],
        }],
    };
    let plan = base_plan(vec![planned(1, Representation::SvgPath)]);
    let svg = emit_svg_from_plan(&plan, &render).unwrap();
    assert!(!svg.contains("<path"));
}

#[test]
fn emits_image() {
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![RenderNode::Image {
                id: RenderNodeId::new(1),
                path: "img.png".into(),
                corners_mm: [(0.0, 0.0), (10.0, 0.0), (10.0, 5.0), (0.0, 5.0)],
                alpha: 0.9,
            }],
        }],
    };
    let plan = base_plan(vec![planned(1, Representation::SvgImage)]);
    let svg = emit_svg_from_plan(&plan, &render).unwrap();
    assert!(svg.contains("<image"));
    assert!(svg.contains("img.png"));
}

#[test]
fn empty_document_uses_a4_default_size() {
    let render = RenderDocument { pages: vec![] };
    let plan = base_plan(vec![]);
    let svg = emit_svg_from_plan(&plan, &render).unwrap();
    assert!(svg.contains(r#"width="210mm" height="297mm""#));
}

#[test]
fn multipage_includes_gap_between_pages() {
    let render = RenderDocument {
        pages: vec![
            RenderPage {
                width_mm: 100.0,
                height_mm: 50.0,
                nodes: vec![],
            },
            RenderPage {
                width_mm: 100.0,
                height_mm: 50.0,
                nodes: vec![],
            },
        ],
    };
    let plan = base_plan(vec![]);
    let svg = emit_svg_from_plan(&plan, &render).unwrap();
    assert!(svg.contains("translate(0 60)"));
}

#[test]
fn rejects_plan_node_mismatch() {
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![RenderNode::Circle {
                id: RenderNodeId::new(1),
                x_mm: 1.0,
                y_mm: 2.0,
                radius_mm: 3.0,
                fill: Color::BLACK,
                stroke_width_mm: None,
                alpha: 1.0,
            }],
        }],
    };
    let plan = base_plan(vec![planned(1, Representation::SvgRect)]);
    let err = emit_svg_from_plan(&plan, &render).unwrap_err();
    assert!(matches!(err, EmitError::PlanNodeMismatch { .. }));
}

#[test]
fn rejects_missing_render_node() {
    let render = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![],
        }],
    };
    let plan = base_plan(vec![planned(99, Representation::SvgRect)]);
    let err = emit_svg_from_plan(&plan, &render).unwrap_err();
    assert!(matches!(
        err,
        EmitError::MissingRenderNode { render_id: 99 }
    ));
}
