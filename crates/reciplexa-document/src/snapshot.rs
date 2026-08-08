//! Immutable document snapshot with revision.

use reciplexa_identity::document::{DocumentIdentity, DocumentRevision};

use crate::node::NodeStore;
use crate::provenance::NodeProvenance;
use crate::reference::ReferenceGraph;

/// Point-in-time document state.
#[derive(Debug, Clone)]
pub struct DocumentSnapshot {
    pub identity: DocumentIdentity,
    pub revision: DocumentRevision,
    pub nodes: NodeStore,
    pub provenance: NodeProvenance,
    pub references: ReferenceGraph,
}

impl DocumentSnapshot {
    pub fn new(identity: DocumentIdentity) -> Self {
        let mut nodes = NodeStore::new();
        nodes.allocate(crate::node::DocumentNodeKind::Document);
        Self {
            identity,
            revision: DocumentRevision::ZERO,
            nodes,
            provenance: NodeProvenance::default(),
            references: ReferenceGraph::new(),
        }
    }

    pub fn bump_revision(&mut self) {
        self.revision = self.revision.next();
    }
}
