use reciplexa_document::*;
use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::package::ModuleId;
use reciplexa_identity::syntax::SyntaxNodeId;
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;
use reciplexa_source::resource::SourceResourceId;

fn sample_provenance() -> SourceProvenance {
    SourceProvenance {
        source_resource_id: SourceResourceId::new(1),
        module_id: ModuleId::new(2),
        text_range: TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(10)).unwrap(),
        syntax_node_id: Some(SyntaxNodeId::new(3)),
        kind: None,
    }
}

#[test]
fn insert_and_get_roundtrip() {
    let mut map = NodeProvenance::default();
    let node = StableNodeId::new(1);
    let prov = sample_provenance();
    map.insert(node, prov.clone());
    assert_eq!(map.get(node), Some(&prov));
}

#[test]
fn get_missing_returns_none() {
    let map = NodeProvenance::default();
    assert!(map.get(StableNodeId::new(99)).is_none());
}

#[test]
fn insert_overwrites_existing() {
    let mut map = NodeProvenance::default();
    let node = StableNodeId::new(1);
    map.insert(node, sample_provenance());
    let mut updated = sample_provenance();
    updated.syntax_node_id = None;
    map.insert(node, updated.clone());
    assert_eq!(map.get(node), Some(&updated));
}
