//! Source provenance for document nodes.

use std::collections::HashMap;

use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::package::ModuleId;
use reciplexa_identity::syntax::SyntaxNodeId;
use reciplexa_source::range::TextRange;
use reciplexa_source::resource::SourceResourceId;

/// Origin of a node in source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProvenance {
    pub source_resource_id: SourceResourceId,
    pub module_id: ModuleId,
    pub text_range: TextRange,
    pub syntax_node_id: Option<SyntaxNodeId>,
}

/// Maps stable document nodes back to source.
#[derive(Debug, Clone, Default)]
pub struct NodeProvenance {
    pub by_node: HashMap<StableNodeId, SourceProvenance>,
}

impl NodeProvenance {
    pub fn insert(&mut self, node: StableNodeId, prov: SourceProvenance) {
        self.by_node.insert(node, prov);
    }

    pub fn get(&self, node: StableNodeId) -> Option<&SourceProvenance> {
        self.by_node.get(&node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
