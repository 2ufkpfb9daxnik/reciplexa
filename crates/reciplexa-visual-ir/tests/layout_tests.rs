use reciplexa_scene::Color;
use reciplexa_visual_ir::domain::{DomainDocument, DomainNode, DomainNodeId, DomainPage};
use reciplexa_visual_ir::layout::*;

#[test]
fn identity_layout_preserves_nodes() {
    let domain = DomainDocument {
        pages: vec![DomainPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes: vec![DomainNode::Circle {
                id: DomainNodeId::new(1),
                x_mm: 1.0,
                y_mm: 2.0,
                radius_mm: 3.0,
                fill: Color::BLACK,
                stroke_width_mm: None,
                alpha: 1.0,
                stable_node_id: None,
                source_byte_start: None,
                source_byte_end: None,
            }],
        }],
    };
    let layout = layout_identity(&domain);
    assert_eq!(layout.pages[0].nodes.len(), 1);
}

#[test]
fn empty_domain_produces_empty_layout() {
    let domain = DomainDocument { pages: vec![] };
    let layout = layout_identity(&domain);
    assert!(layout.pages.is_empty());
    assert!(layout_pages(&layout).is_empty());
}
