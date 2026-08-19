use reciplexa_scene::Color;
use reciplexa_visual_ir::domain::*;

use reciplexa_identity::document::StableNodeId;

fn base_fields() -> (Option<StableNodeId>, Option<u32>, Option<u32>) {
    (Some(StableNodeId::new(7)), Some(10), Some(20))
}

#[test]
fn circle_id_stable_and_source_bytes() {
    let (sid, start, end) = base_fields();
    let node = DomainNode::Circle {
        id: DomainNodeId::new(1),
        x_mm: 0.0,
        y_mm: 0.0,
        radius_mm: 1.0,
        fill: Color::BLACK,
        stroke_width_mm: None,
        alpha: 1.0,
        stable_node_id: sid,
        source_byte_start: start,
        source_byte_end: end,
    };
    assert_eq!(node.id(), DomainNodeId::new(1));
    assert_eq!(node.stable_node_id(), sid);
    assert_eq!(node.source_bytes(), (start, end));
}

#[test]
fn rect_id_stable_and_source_bytes() {
    let (sid, start, end) = base_fields();
    let node = DomainNode::Rect {
        id: DomainNodeId::new(2),
        x_mm: 0.0,
        y_mm: 0.0,
        width_mm: 1.0,
        height_mm: 1.0,
        fill: Color::BLACK,
        stroke_width_mm: None,
        alpha: 1.0,
        stable_node_id: sid,
        source_byte_start: start,
        source_byte_end: end,
    };
    assert_eq!(node.id(), DomainNodeId::new(2));
    assert_eq!(node.stable_node_id(), sid);
    assert_eq!(node.source_bytes(), (start, end));
}

#[test]
fn ellipse_id_stable_and_source_bytes() {
    let (sid, start, end) = base_fields();
    let node = DomainNode::Ellipse {
        id: DomainNodeId::new(3),
        x_mm: 0.0,
        y_mm: 0.0,
        rx_mm: 1.0,
        ry_mm: 2.0,
        fill: Color::BLACK,
        alpha: 1.0,
        stable_node_id: sid,
        source_byte_start: start,
        source_byte_end: end,
    };
    assert_eq!(node.id(), DomainNodeId::new(3));
    assert_eq!(node.stable_node_id(), sid);
    assert_eq!(node.source_bytes(), (start, end));
}

#[test]
fn polygon_id_stable_and_source_bytes() {
    let (sid, start, end) = base_fields();
    let node = DomainNode::Polygon {
        id: DomainNodeId::new(4),
        points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
        fill: Color::BLACK,
        stroke_width_mm: None,
        alpha: 1.0,
        stable_node_id: sid,
        source_byte_start: start,
        source_byte_end: end,
    };
    assert_eq!(node.id(), DomainNodeId::new(4));
    assert_eq!(node.stable_node_id(), sid);
    assert_eq!(node.source_bytes(), (start, end));
}

#[test]
fn text_id_stable_and_source_bytes() {
    let (sid, start, end) = base_fields();
    let node = DomainNode::Text {
        id: DomainNodeId::new(5),
        x_mm: 0.0,
        y_mm: 0.0,
        size_mm: 12.0,
        width_mm: 10.0,
        height_mm: 10.0,
        rotation_deg: 0.0,
        content: "x".into(),
        fill: Color::BLACK,
        alpha: 1.0,
        stable_node_id: sid,
        source_byte_start: start,
        source_byte_end: end,
        glyph_ids: None,
        font_digest: None,
        glyph_advances_mm: None,
    };
    assert_eq!(node.id(), DomainNodeId::new(5));
    assert_eq!(node.stable_node_id(), sid);
    assert_eq!(node.source_bytes(), (start, end));
}

#[test]
fn path_id_stable_and_source_bytes() {
    let (sid, start, end) = base_fields();
    let node = DomainNode::Path {
        id: DomainNodeId::new(6),
        points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
        stroke: Color::BLACK,
        width_mm: 1.0,
        closed: false,
        alpha: 1.0,
        stable_node_id: sid,
        source_byte_start: start,
        source_byte_end: end,
    };
    assert_eq!(node.id(), DomainNodeId::new(6));
    assert_eq!(node.stable_node_id(), sid);
    assert_eq!(node.source_bytes(), (start, end));
}

#[test]
fn image_id_stable_and_source_bytes() {
    let (sid, start, end) = base_fields();
    let node = DomainNode::Image {
        id: DomainNodeId::new(7),
        path: "a.png".into(),
        corners_mm: [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
        alpha: 1.0,
        stable_node_id: sid,
        source_byte_start: start,
        source_byte_end: end,
    };
    assert_eq!(node.id(), DomainNodeId::new(7));
    assert_eq!(node.stable_node_id(), sid);
    assert_eq!(node.source_bytes(), (start, end));
}

#[test]
fn none_provenance_fields() {
    let node = DomainNode::Rect {
        id: DomainNodeId::new(8),
        x_mm: 0.0,
        y_mm: 0.0,
        width_mm: 1.0,
        height_mm: 1.0,
        fill: Color::BLACK,
        stroke_width_mm: None,
        alpha: 1.0,
        stable_node_id: None,
        source_byte_start: None,
        source_byte_end: None,
    };
    assert_eq!(node.stable_node_id(), None);
    assert_eq!(node.source_bytes(), (None, None));
}
