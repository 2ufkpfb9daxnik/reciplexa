//! Document node kinds and storage.

use std::collections::HashMap;

use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::document::StableNodeIdAllocator;

use crate::property::{LayoutBox, NodeProperty, TextContent};

/// Kinds of nodes in the editable document tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentNodeKind {
    Document,
    Page,
    Group,
    Rectangle,
    Text,
}

/// A node in the ownership tree.
#[derive(Debug, Clone)]
pub struct DocumentNode {
    pub id: StableNodeId,
    pub kind: DocumentNodeKind,
    pub parent: Option<StableNodeId>,
    pub children: Vec<StableNodeId>,
    pub properties: Vec<NodeProperty>,
}

impl DocumentNode {
    pub fn new(id: StableNodeId, kind: DocumentNodeKind) -> Self {
        Self {
            id,
            kind,
            parent: None,
            children: Vec::new(),
            properties: Vec::new(),
        }
    }

    pub fn layout(&self) -> Option<LayoutBox> {
        self.properties.iter().find_map(|p| match p {
            NodeProperty::Layout(box_) => Some(*box_),
            _ => None,
        })
    }

    pub fn text(&self) -> Option<&TextContent> {
        self.properties.iter().find_map(|p| match p {
            NodeProperty::Text(t) => Some(t),
            _ => None,
        })
    }

    pub fn set_layout(&mut self, layout: LayoutBox) {
        if let Some(slot) = self.properties.iter_mut().find_map(|p| match p {
            NodeProperty::Layout(b) => Some(b),
            _ => None,
        }) {
            *slot = layout;
        } else {
            self.properties.push(NodeProperty::Layout(layout));
        }
    }
}

/// Map-backed node storage with stable ids.
#[derive(Debug, Clone, Default)]
pub struct NodeStore {
    nodes: HashMap<StableNodeId, DocumentNode>,
    alloc: StableNodeIdAllocator,
    root: Option<StableNodeId>,
}

impl NodeStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn root_id(&self) -> Option<StableNodeId> {
        self.root
    }

    pub fn get(&self, id: StableNodeId) -> Option<&DocumentNode> {
        self.nodes.get(&id)
    }

    pub fn get_mut(&mut self, id: StableNodeId) -> Option<&mut DocumentNode> {
        self.nodes.get_mut(&id)
    }

    pub fn allocate(&mut self, kind: DocumentNodeKind) -> StableNodeId {
        let id = self.alloc.allocate();
        let node = DocumentNode::new(id, kind);
        if kind == DocumentNodeKind::Document && self.root.is_none() {
            self.root = Some(id);
        }
        self.nodes.insert(id, node);
        id
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Insert a node preserving its stable ID (codec round-trip).
    pub fn insert_preserved(&mut self, node: DocumentNode) {
        let id = node.id;
        if node.kind == DocumentNodeKind::Document {
            self.root = Some(id);
        }
        self.alloc.ensure_next_above(id.get());
        self.nodes.insert(id, node);
    }

    pub fn link_child(&mut self, parent: StableNodeId, child: StableNodeId) {
        if let Some(c) = self.nodes.get_mut(&child) {
            c.parent = Some(parent);
        }
        if let Some(p) = self.nodes.get_mut(&parent) {
            if !p.children.contains(&child) {
                p.children.push(child);
            }
        }
    }

    pub fn insert_child(
        &mut self,
        parent: StableNodeId,
        kind: DocumentNodeKind,
    ) -> Option<StableNodeId> {
        self.nodes.get(&parent)?;
        let child_id = self.allocate(kind);
        {
            let child = self
                .nodes
                .get_mut(&child_id)
                .expect("allocate always inserts");
            child.parent = Some(parent);
        }
        self.nodes
            .get_mut(&parent)
            .expect("parent existence checked above")
            .children
            .push(child_id);
        Some(child_id)
    }

    pub fn remove_subtree(&mut self, id: StableNodeId) {
        let Some(node) = self.nodes.get(&id).cloned() else {
            return;
        };
        for child in node.children {
            self.remove_subtree(child);
        }
        if let Some(parent) = node.parent {
            if let Some(p) = self.nodes.get_mut(&parent) {
                p.children.retain(|c| *c != id);
            }
        }
        self.nodes.remove(&id);
    }

    pub fn duplicate_subtree(&mut self, id: StableNodeId) -> Option<StableNodeId> {
        let source = self.nodes.get(&id)?.clone();
        let parent = source.parent?;
        self.nodes.get(&parent)?;
        let new_id = self.allocate(source.kind);
        {
            let new_node = self
                .nodes
                .get_mut(&new_id)
                .expect("allocate always inserts");
            new_node.properties = source.properties.clone();
            new_node.parent = Some(parent);
        }
        self.nodes
            .get_mut(&parent)
            .expect("parent existence checked above")
            .children
            .push(new_id);
        for child in source.children {
            let dup = self.duplicate_subtree(child)?;
            self.nodes
                .get_mut(&dup)
                .expect("duplicate always inserts")
                .parent = Some(new_id);
            self.nodes
                .get_mut(&new_id)
                .expect("new node still present")
                .children
                .push(dup);
        }
        Some(new_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &DocumentNode> {
        self.nodes.values()
    }

    /// Test/support helper: drop a node id while leaving parent child lists intact.
    pub fn drop_node_keep_links(&mut self, id: StableNodeId) {
        self.nodes.remove(&id);
    }
}
