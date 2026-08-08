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
}

/// Encode a document snapshot preserving stable node IDs.
///
/// `PortableSnapshot` only contains JSON-safe scalars/strings, so encoding cannot fail.
pub fn encode_snapshot(snap: &DocumentSnapshot) -> Vec<u8> {
    let portable = snapshot_to_portable_public(snap);
    serde_json::to_vec(&portable).expect("PortableSnapshot is always JSON-serializable")
}

/// Decode a portable snapshot into a document snapshot.
pub fn decode_snapshot(bytes: &[u8]) -> Result<DocumentSnapshot, CodecError> {
    let portable: PortableSnapshot =
        serde_json::from_slice(bytes).map_err(|e| CodecError::Decode(e.to_string()))?;
    if portable.schema_version > SCHEMA_VERSION {
        return Err(CodecError::UnsupportedSchema(portable.schema_version));
    }
    Ok(portable_to_snapshot(&portable))
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

fn portable_to_snapshot(p: &PortableSnapshot) -> DocumentSnapshot {
    let mut store = NodeStore::new();
    let id_map: std::collections::HashMap<u64, StableNodeId> = p
        .nodes
        .iter()
        .map(|n| (n.id, StableNodeId::new(n.id)))
        .collect();
    for pn in &p.nodes {
        let id = StableNodeId::new(pn.id);
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
    DocumentSnapshot {
        identity: DocumentIdentity::new(p.document_id),
        revision: DocumentRevision::new(p.revision),
        nodes: store,
        provenance: Default::default(),
        references: Default::default(),
    }
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
