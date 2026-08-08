use reciplexa_document::*;
use reciplexa_identity::document::DocumentIdentity;
use reciplexa_scene::{Affine, Color, Page, PaperSize, Rect, Shape, Text};
use reciplexa_source::resource::SourceResourceId;

#[test]
fn empty_store_has_no_drawables_or_shapes() {
    let store = NodeStore::new();
    assert!(drawable_node_ids(&store).is_empty());
    assert!(scene_shapes_from_document(&store).is_empty());
}

#[test]
fn roundtrip_rect_through_document() {
    let page = Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 5.0,
            y_mm: 6.0,
            width_mm: 10.0,
            height_mm: 20.0,
            fill: Color::RED,
        })],
    };
    let snap =
        document_from_scene_page(DocumentIdentity::new(1), &page, SourceResourceId::new(1));
    let shapes = scene_shapes_from_document(&snap.nodes);
    assert_eq!(shapes.len(), 1);
}

#[test]
fn drawable_ids_match_shape_count() {
    let page = Page {
        paper: PaperSize::a4(),
        shapes: vec![
            Shape::Rect(Rect {
                x_mm: 1.0,
                y_mm: 2.0,
                width_mm: 3.0,
                height_mm: 4.0,
                fill: Color::BLACK,
            }),
            Shape::Text(Text {
                x_mm: 5.0,
                y_mm: 6.0,
                size_mm: 12.0,
                width_mm: None,
                height_mm: None,
                content: "hi".into(),
                fill: Color::BLACK,
            }),
        ],
    };
    let snap =
        document_from_scene_page(DocumentIdentity::new(1), &page, SourceResourceId::new(1));
    assert_eq!(drawable_node_ids(&snap.nodes).len(), 2);
}

#[test]
fn group_children_project_to_shapes() {
    let page = Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Group {
            transform: Affine::identity(),
            children: vec![Shape::Rect(Rect {
                x_mm: 1.0,
                y_mm: 2.0,
                width_mm: 3.0,
                height_mm: 4.0,
                fill: Color::BLACK,
            })],
        }],
    };
    let snap =
        document_from_scene_page(DocumentIdentity::new(2), &page, SourceResourceId::new(1));
    assert_eq!(scene_shapes_from_document(&snap.nodes).len(), 1);
}

#[test]
fn text_roundtrip_preserves_content_and_color() {
    let page = Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 10.0,
            y_mm: 20.0,
            size_mm: 12.0,
            width_mm: Some(40.0),
            height_mm: Some(20.0),
            content: "hello".into(),
            fill: Color::BLUE,
        })],
    };
    let snap =
        document_from_scene_page(DocumentIdentity::new(3), &page, SourceResourceId::new(1));
    let shapes = scene_shapes_from_document(&snap.nodes);
    match &shapes[0] {
        Shape::Text(t) => {
            assert_eq!(t.content, "hello");
            assert_eq!(t.fill, Color::BLUE);
            assert_eq!(t.width_mm, Some(40.0));
        }
        _ => panic!("expected text"),
    }
}

#[test]
fn text_without_box_uses_default_width() {
    let page = Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 1.0,
            y_mm: 2.0,
            size_mm: 10.0,
            width_mm: None,
            height_mm: None,
            content: "x".into(),
            fill: Color::BLACK,
        })],
    };
    let snap =
        document_from_scene_page(DocumentIdentity::new(4), &page, SourceResourceId::new(1));
    let node = drawable_node_ids(&snap.nodes)[0];
    let layout = snap.nodes.get(node).unwrap().layout().unwrap();
    assert_eq!(layout.width, 100.0);
}

#[test]
fn layer_provenance_maps_byte_range() {
    let page = Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 1.0,
            height_mm: 1.0,
            fill: Color::BLACK,
        })],
    };
    let layers = vec![LayerSpan {
        byte_start: 5,
        byte_end: 10,
        label: "rect".into(),
    }];
    let snap = document_from_scene_page_with_layers(
        DocumentIdentity::new(5),
        &page,
        &layers,
        SourceResourceId::new(1),
        &[],
    );
    let node = drawable_node_ids(&snap.nodes)[0];
    let prov = snap.provenance.get(node).unwrap();
    assert_eq!(prov.text_range.start().get(), 5);
    assert_eq!(prov.text_range.end().get(), 10);
}

#[test]
fn ingest_skips_unsupported_shapes_and_empty_drawables() {
    let page = Page {
        paper: PaperSize::a4(),
        shapes: vec![
            Shape::Circle(reciplexa_scene::Circle {
                x_mm: 0.0,
                y_mm: 0.0,
                radius_mm: 1.0,
                fill: Color::BLACK,
            }),
            Shape::Line(reciplexa_scene::Line {
                x1_mm: 0.0,
                y1_mm: 0.0,
                x2_mm: 1.0,
                y2_mm: 1.0,
                stroke: Color::BLACK,
                width_mm: 1.0,
            }),
        ],
    };
    let snap =
        document_from_scene_page(DocumentIdentity::new(6), &page, SourceResourceId::new(1));
    assert!(drawable_node_ids(&snap.nodes).is_empty());

    let mut empty = DocumentSnapshot::new(DocumentIdentity::new(7));
    let root = empty.nodes.root_id().unwrap();
    let _ = empty
        .nodes
        .insert_child(root, DocumentNodeKind::Rectangle)
        .unwrap();
    assert!(scene_shapes_from_document(&empty.nodes).is_empty());

    // Text without layout is skipped by collect_shapes.
    let tid = empty
        .nodes
        .insert_child(root, DocumentNodeKind::Text)
        .unwrap();
    assert!(empty.nodes.get(tid).unwrap().layout().is_none());
    assert!(scene_shapes_from_document(&empty.nodes).is_empty());
}
