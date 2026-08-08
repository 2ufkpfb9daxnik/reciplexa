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
