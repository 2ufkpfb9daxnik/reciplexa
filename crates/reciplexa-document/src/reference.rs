//! Cross-node reference graph (Phase 3 §5.2).

use std::collections::HashMap;

use reciplexa_identity::document::StableNodeId;

/// Directed references between document nodes (e.g. group → children).
#[derive(Debug, Clone, Default)]
pub struct ReferenceGraph {
    pub edges: HashMap<StableNodeId, Vec<StableNodeId>>,
}

impl ReferenceGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn link(&mut self, from: StableNodeId, to: StableNodeId) {
        self.edges.entry(from).or_default().push(to);
    }

    pub fn children_of(&self, node: StableNodeId) -> &[StableNodeId] {
        self.edges.get(&node).map(|v| v.as_slice()).unwrap_or(&[])
    }
}
