use reciplexa_codec::codec::*;
use reciplexa_document::node::DocumentNodeKind;
use reciplexa_document::snapshot::DocumentSnapshot;
use reciplexa_identity::document::{DocumentIdentity, StableNodeId};

#[test]
fn roundtrip_preserves_stable_ids() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(42));
    let root = snap.nodes.root_id().unwrap();
    let child = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Rectangle)
        .unwrap();
    let child_id = child;
    let bytes = encode_snapshot(&snap);
    let decoded = decode_snapshot(&bytes).unwrap();
    assert_eq!(decoded.identity, snap.identity);
    assert_eq!(decoded.nodes.len(), snap.nodes.len());
    assert!(decoded.nodes.get(child_id).is_some());
    assert_eq!(decoded.nodes.get(child_id).unwrap().id, child_id);
}

#[test]
fn unsupported_schema_returns_error() {
    let json = r#"{"schema_version":999,"document_id":1,"revision":0,"nodes":[]}"#;
    let err = decode_snapshot(json.as_bytes()).unwrap_err();
    assert!(matches!(err, CodecError::UnsupportedSchema(999)));
}

#[test]
fn garbage_bytes_decode_error() {
    let err = decode_snapshot(b"not-json{{{").unwrap_err();
    assert!(matches!(err, CodecError::Decode(_)));
}

#[test]
fn parse_kind_known_and_unknown() {
    let kinds = [
        ("Page", DocumentNodeKind::Page),
        ("Group", DocumentNodeKind::Group),
        ("Rectangle", DocumentNodeKind::Rectangle),
        ("Text", DocumentNodeKind::Text),
        ("UnknownKind", DocumentNodeKind::Document),
    ];
    for (kind_str, expected) in kinds {
        let json = format!(
            r#"{{"schema_version":1,"document_id":1,"revision":0,"nodes":[{{"id":1,"kind":"{kind_str}","parent":null,"children":[]}}]}}"#
        );
        let snap = decode_snapshot(json.as_bytes()).unwrap();
        let node = snap.nodes.get(StableNodeId::new(1)).unwrap();
        assert_eq!(node.kind, expected, "kind string {kind_str}");
    }
}

#[test]
fn nested_parent_child_roundtrip() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let root = snap.nodes.root_id().unwrap();
    let group = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Group)
        .unwrap();
    let rect = snap
        .nodes
        .insert_child(group, DocumentNodeKind::Rectangle)
        .unwrap();
    let bytes = encode_snapshot(&snap);
    let decoded = decode_snapshot(&bytes).unwrap();
    let rect_node = decoded.nodes.get(rect).unwrap();
    assert_eq!(rect_node.parent, Some(group));
    let _group_node = decoded.nodes.get(group).unwrap();
    assert_eq!(decoded.nodes.get(rect).unwrap().parent, Some(group));
}
