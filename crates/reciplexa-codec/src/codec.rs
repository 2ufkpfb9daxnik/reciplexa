//! Canonical portable snapshot encoding.

use reciplexa_document::node::{DocumentNode, DocumentNodeKind, NodeStore};
use reciplexa_document::snapshot::DocumentSnapshot;
use reciplexa_identity::document::{DocumentIdentity, DocumentRevision, StableNodeId};
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortableNode {
    pub id: u64,
    pub kind: String,
    pub parent: Option<u64>,
    pub children: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortableSnapshot {
    pub schema_version: u32,
    pub document_id: u128,
    pub revision: u64,
    pub nodes: Vec<PortableNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    UnsupportedSchema(u32),
    Decode(String),
    Encode(String),
    StableIdLost,
}

/// Encode a document snapshot preserving stable node IDs.
pub fn encode_snapshot(snap: &DocumentSnapshot) -> Result<Vec<u8>, CodecError> {
    let portable = snapshot_to_portable(snap);
    serde_json::to_vec(&portable).map_err(|e| CodecError::Encode(e.to_string()))
}

/// Decode a portable snapshot into a document snapshot.
pub fn decode_snapshot(bytes: &[u8]) -> Result<DocumentSnapshot, CodecError> {
    let portable: PortableSnapshot =
        serde_json::from_slice(bytes).map_err(|e| CodecError::Decode(e.to_string()))?;
    if portable.schema_version > SCHEMA_VERSION {
        return Err(CodecError::UnsupportedSchema(portable.schema_version));
    }
    portable_to_snapshot(&portable)
}

fn snapshot_to_portable(snap: &DocumentSnapshot) -> PortableSnapshot {
    snapshot_to_portable_public(snap)
}

/// Public helper for envelope encoding.
pub fn snapshot_to_portable_public(snap: &DocumentSnapshot) -> PortableSnapshot {
    let nodes = snap
        .nodes
        .iter()
        .map(|node| PortableNode {
            id: node.id.get(),
            kind: format!("{:?}", node.kind),
            parent: node.parent.map(|p| p.get()),
            children: node.children.iter().map(|c| c.get()).collect(),
        })
        .collect();
    PortableSnapshot {
        schema_version: SCHEMA_VERSION,
        document_id: snap.identity.get(),
        revision: snap.revision.get(),
        nodes,
    }
}

fn portable_to_snapshot(p: &PortableSnapshot) -> Result<DocumentSnapshot, CodecError> {
    let mut store = NodeStore::new();
    let id_map: std::collections::HashMap<u64, StableNodeId> = p
        .nodes
        .iter()
        .map(|n| (n.id, StableNodeId::new(n.id)))
        .collect();
    for pn in &p.nodes {
        let id = id_map
            .get(&pn.id)
            .copied()
            .ok_or(CodecError::StableIdLost)?;
        let kind = parse_kind(&pn.kind);
        let mut node = DocumentNode::new(id, kind);
        node.parent = pn.parent.and_then(|pid| id_map.get(&pid).copied());
        node.children = pn
            .children
            .iter()
            .filter_map(|c| id_map.get(c).copied())
            .collect();
        store.insert_preserved(node);
    }
    Ok(DocumentSnapshot {
        identity: DocumentIdentity::new(p.document_id),
        revision: DocumentRevision::new(p.revision),
        nodes: store,
        provenance: Default::default(),
        references: Default::default(),
    })
}

fn parse_kind(s: &str) -> DocumentNodeKind {
    match s {
        "Page" => DocumentNodeKind::Page,
        "Group" => DocumentNodeKind::Group,
        "Rectangle" => DocumentNodeKind::Rectangle,
        "Text" => DocumentNodeKind::Text,
        _ => DocumentNodeKind::Document,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_document::node::DocumentNodeKind;
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
        let bytes = encode_snapshot(&snap).unwrap();
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
        let bytes = encode_snapshot(&snap).unwrap();
        let decoded = decode_snapshot(&bytes).unwrap();
        let rect_node = decoded.nodes.get(rect).unwrap();
        assert_eq!(rect_node.parent, Some(group));
        let group_node = decoded.nodes.get(group).unwrap();
        assert_eq!(decoded.nodes.get(rect).unwrap().parent, Some(group));
    }
}
