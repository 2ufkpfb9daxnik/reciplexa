//! Provenance side table linking IR nodes to source and artifact IDs.

use std::collections::HashMap;

use reciplexa_identity::document::StableNodeId;

use crate::render::RenderNodeId;

/// Provenance for one render node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProvenance {
    pub render_id: RenderNodeId,
    pub stable_node_id: Option<StableNodeId>,
    pub source_byte_start: Option<u32>,
    pub source_byte_end: Option<u32>,
}

/// Maps render nodes to source provenance.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProvenanceMap {
    entries: HashMap<RenderNodeId, RenderProvenance>,
}

impl ProvenanceMap {
    pub fn insert(&mut self, entry: RenderProvenance) {
        self.entries.insert(entry.render_id, entry);
    }

    pub fn get(&self, id: RenderNodeId) -> Option<&RenderProvenance> {
        self.entries.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &RenderProvenance> {
        self.entries.values()
    }
}

/// Artifact element provenance after emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactProvenance {
    pub render_id: RenderNodeId,
    pub artifact_element_id: String,
}
