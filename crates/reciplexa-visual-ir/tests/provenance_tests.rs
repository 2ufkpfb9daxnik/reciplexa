use reciplexa_identity::document::StableNodeId;
use reciplexa_visual_ir::provenance::*;
use reciplexa_visual_ir::render::RenderNodeId;

#[test]
fn insert_and_get_provenance() {
    let mut map = ProvenanceMap::default();
    let entry = RenderProvenance {
        render_id: RenderNodeId::new(1),
        stable_node_id: Some(StableNodeId::new(5)),
        source_byte_start: Some(1),
        source_byte_end: Some(2),
    };
    map.insert(entry);
    assert_eq!(
        map.get(RenderNodeId::new(1)).unwrap().source_byte_start,
        Some(1)
    );
    assert_eq!(map.iter().count(), 1);
}

#[test]
fn insert_overwrites_existing_provenance() {
    let mut map = ProvenanceMap::default();
    map.insert(RenderProvenance {
        render_id: RenderNodeId::new(1),
        stable_node_id: Some(StableNodeId::new(1)),
        source_byte_start: Some(10),
        source_byte_end: Some(20),
    });
    map.insert(RenderProvenance {
        render_id: RenderNodeId::new(1),
        stable_node_id: Some(StableNodeId::new(99)),
        source_byte_start: Some(30),
        source_byte_end: Some(40),
    });
    let got = map.get(RenderNodeId::new(1)).unwrap();
    assert_eq!(got.stable_node_id, Some(StableNodeId::new(99)));
    assert_eq!(got.source_byte_start, Some(30));
    assert_eq!(map.iter().count(), 1);
}
