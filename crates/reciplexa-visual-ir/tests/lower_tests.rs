use reciplexa_visual_ir::domain::*;
use reciplexa_visual_ir::layout::*;
use reciplexa_visual_ir::lower::*;
use reciplexa_visual_ir::render::*;

use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::{
    Affine, Circle, Color, Document, Image, Line, Page, PaperSize, Polygon, Polyline, Rect, Shape,
    Text,
};

#[test]
fn lowers_rect_to_render_ir() {
    let mut doc = Document::default();
    doc.pages.push(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 10.0,
            y_mm: 20.0,
            width_mm: 30.0,
            height_mm: 40.0,
            fill: Color::RED,
        })],
    });
    let (render, prov) = lower_scene_document(&doc);
    assert_eq!(render.pages.len(), 1);
    assert_eq!(render.pages[0].nodes.len(), 1);
    assert!(matches!(render.pages[0].nodes[0], RenderNode::Rect { .. }));
    assert!(prov.get(RenderNodeId::new(1)).is_some());
}

#[test]
fn lowers_circle_to_render_ir() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Circle(Circle {
            x_mm: 5.0,
            y_mm: 5.0,
            radius_mm: 3.0,
            fill: Color::BLUE,
        })],
    });
    let (render, _) = lower_scene_document(&doc);
    assert!(matches!(
        render.pages[0].nodes[0],
        RenderNode::Circle { .. }
    ));
}

#[test]
fn lowers_text_to_render_ir() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 1.0,
            y_mm: 2.0,
            size_mm: 12.0,
            width_mm: None,
            height_mm: None,
            content: "hello".into(),
            fill: Color::BLACK,
        })],
    });
    let (render, _) = lower_scene_document(&doc);
    assert!(matches!(render.pages[0].nodes[0], RenderNode::Text { .. }));
}

#[test]
fn lowers_path_to_render_ir() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Line(Line {
            x1_mm: 0.0,
            y1_mm: 0.0,
            x2_mm: 10.0,
            y2_mm: 10.0,
            stroke: Color::BLACK,
            width_mm: 0.5,
        })],
    });
    let (render, _) = lower_scene_document(&doc);
    assert!(matches!(render.pages[0].nodes[0], RenderNode::Path { .. }));
}

#[test]
fn lowers_image_to_render_ir() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(Image {
            path: "pic.png".into(),
            x_mm: 1.0,
            y_mm: 2.0,
            width_mm: 30.0,
            height_mm: 20.0,
        })],
    });
    let (render, _) = lower_scene_document(&doc);
    assert!(matches!(render.pages[0].nodes[0], RenderNode::Image { .. }));
}

#[test]
fn lowers_non_axis_aligned_polygon() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Polygon(Polygon {
            points_mm: vec![(0.0, 0.0), (10.0, 0.0), (5.0, 8.0)],
            fill: Color::GREEN,
        })],
    });
    let domain = scene_to_domain(&doc, &LowerOptions::with_ellipse_sides(32));
    assert!(matches!(
        domain.pages[0].nodes[0],
        DomainNode::Polygon { .. }
    ));
    let (render, _) = layout_to_render(&layout_identity(&domain));
    assert!(matches!(
        render.pages[0].nodes[0],
        RenderNode::Polygon { .. }
    ));
}

#[test]
fn axis_aligned_rect_becomes_domain_rect() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 1.0,
            y_mm: 2.0,
            width_mm: 3.0,
            height_mm: 4.0,
            fill: Color::BLACK,
        })],
    });
    let domain = scene_to_domain(&doc, &LowerOptions::with_ellipse_sides(32));
    assert!(matches!(domain.pages[0].nodes[0], DomainNode::Rect { .. }));
}

#[test]
fn rotated_rect_stays_polygon() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Group {
            transform: Affine::rotate_deg(45.0),
            children: vec![Shape::Rect(Rect {
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 10.0,
                height_mm: 5.0,
                fill: Color::BLACK,
            })],
        }],
    });
    let domain = scene_to_domain(&doc, &LowerOptions::with_ellipse_sides(32));
    assert!(matches!(
        domain.pages[0].nodes[0],
        DomainNode::Polygon { .. }
    ));
}

#[test]
fn three_point_polygon_not_recognized_as_rect() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Polygon(Polygon {
            points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
            fill: Color::BLACK,
        })],
    });
    let domain = scene_to_domain(&doc, &LowerOptions::with_ellipse_sides(32));
    assert!(matches!(
        domain.pages[0].nodes[0],
        DomainNode::Polygon { .. }
    ));
}

#[test]
fn zero_area_rect_stays_polygon() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 0.0,
            height_mm: 10.0,
            fill: Color::BLACK,
        })],
    });
    let domain = scene_to_domain(&doc, &LowerOptions::with_ellipse_sides(32));
    assert!(matches!(
        domain.pages[0].nodes[0],
        DomainNode::Polygon { .. }
    ));
}

#[test]
fn ellipse_sides_zero_defaults_to_32() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Polyline(Polyline {
            points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
            stroke: Color::BLACK,
            width_mm: 1.0,
        })],
    });
    let options = LowerOptions {
        ellipse_sides: 0,
        provenance_hints: vec![],
    };
    let (render, _) = lower_scene_document_with_options(&doc, &options);
    assert_eq!(render.pages.len(), 1);
}

#[test]
fn multi_page_provenance_hints_align_sequentially() {
    let mut doc = Document::default();
    doc.pages.push(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 1.0,
            height_mm: 1.0,
            fill: Color::BLACK,
        })],
    });
    doc.pages.push(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Circle(Circle {
            x_mm: 5.0,
            y_mm: 5.0,
            radius_mm: 2.0,
            fill: Color::RED,
        })],
    });
    let options = LowerOptions {
        ellipse_sides: 32,
        provenance_hints: vec![
            Some(NodeSourceHint {
                stable_node_id: StableNodeId::new(1),
                source_byte_start: 0,
                source_byte_end: 5,
            }),
            Some(NodeSourceHint {
                stable_node_id: StableNodeId::new(2),
                source_byte_start: 10,
                source_byte_end: 20,
            }),
        ],
    };
    let (_, prov) = lower_scene_document_with_options(&doc, &options);
    assert_eq!(
        prov.get(RenderNodeId::new(1)).unwrap().stable_node_id,
        Some(StableNodeId::new(1))
    );
    assert_eq!(
        prov.get(RenderNodeId::new(2)).unwrap().stable_node_id,
        Some(StableNodeId::new(2))
    );
}

#[test]
fn ellipse_domain_node_fallback_to_polygon_render() {
    use reciplexa_visual_ir::domain::DomainNodeId;
    use reciplexa_visual_ir::layout::LayoutPage;
    let layout = LayoutDocument {
        pages: vec![LayoutPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![DomainNode::Ellipse {
                id: DomainNodeId::new(9),
                x_mm: 10.0,
                y_mm: 20.0,
                rx_mm: 5.0,
                ry_mm: 3.0,
                fill: Color::BLACK,
                alpha: 1.0,
                stable_node_id: None,
                source_byte_start: None,
                source_byte_end: None,
            }],
        }],
    };
    let (render, prov) = layout_to_render(&layout);
    match &render.pages[0].nodes[0] {
        RenderNode::Polygon { points_mm, .. } => assert_eq!(points_mm.len(), 32),
        other => panic!("expected polygon fallback, got {other:?}"),
    }
    assert!(prov.get(RenderNodeId::new(9)).is_some());
}

#[test]
fn provenance_none_when_no_hints() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 1.0,
            y_mm: 2.0,
            width_mm: 3.0,
            height_mm: 4.0,
            fill: Color::BLACK,
        })],
    });
    let (_, prov) = lower_scene_document(&doc);
    let entry = prov.get(RenderNodeId::new(1)).unwrap();
    assert_eq!(entry.stable_node_id, None);
    assert_eq!(entry.source_byte_start, None);
    assert_eq!(entry.source_byte_end, None);
}

#[test]
fn attaches_provenance_hints() {
    let mut doc = Document::default();
    doc.pages.push(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 1.0,
            y_mm: 2.0,
            width_mm: 3.0,
            height_mm: 4.0,
            fill: Color::BLACK,
        })],
    });
    let options = LowerOptions {
        ellipse_sides: 32,
        provenance_hints: vec![Some(NodeSourceHint {
            stable_node_id: StableNodeId::new(42),
            source_byte_start: 5,
            source_byte_end: 20,
        })],
    };
    let (_, prov) = lower_scene_document_with_options(&doc, &options);
    let entry = prov.get(RenderNodeId::new(1)).unwrap();
    assert_eq!(entry.stable_node_id, Some(StableNodeId::new(42)));
    assert_eq!(entry.source_byte_start, Some(5));
    assert_eq!(entry.source_byte_end, Some(20));
}
