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
    pub stable_node_id: Option<StableNodeId>,
    pub source_byte_start: Option<u32>,
    pub source_byte_end: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
