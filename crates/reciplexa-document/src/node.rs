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
        let child_id = self.allocate(kind);
        let child = self.nodes.get_mut(&child_id)?;
        child.parent = Some(parent);
        let parent_node = self.nodes.get_mut(&parent)?;
        parent_node.children.push(child_id);
        Some(child_id)
    }

    pub fn remove_subtree(&mut self, id: StableNodeId) {
        if let Some(node) = self.nodes.get(&id).cloned() {
            for child in node.children.clone() {
                self.remove_subtree(child);
            }
            if let Some(parent) = node.parent {
                if let Some(p) = self.nodes.get_mut(&parent) {
                    p.children.retain(|c| *c != id);
                }
            }
            self.nodes.remove(&id);
        }
    }

    pub fn duplicate_subtree(&mut self, id: StableNodeId) -> Option<StableNodeId> {
        let source = self.nodes.get(&id)?.clone();
        let parent = source.parent?;
        let new_id = self.allocate(source.kind);
        if let Some(new_node) = self.nodes.get_mut(&new_id) {
            new_node.properties = source.properties.clone();
            new_node.parent = Some(parent);
        }
        if let Some(p) = self.nodes.get_mut(&parent) {
            p.children.push(new_id);
        }
        for child in source.children {
            let dup = self.duplicate_subtree(child)?;
            if let Some(n) = self.nodes.get_mut(&dup) {
                n.parent = Some(new_id);
            }
            if let Some(n) = self.nodes.get_mut(&new_id) {
                n.children.push(dup);
            }
        }
        Some(new_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &DocumentNode> {
        self.nodes.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_gets_new_ids() {
        let mut store = NodeStore::new();
        let doc = store.allocate(DocumentNodeKind::Document);
        let rect = store
            .insert_child(doc, DocumentNodeKind::Rectangle)
            .unwrap();
        let dup = store.duplicate_subtree(rect).unwrap();
        assert_ne!(rect, dup);
    }

    #[test]
    fn remove_subtree_deletes_descendants() {
        let mut store = NodeStore::new();
        let doc = store.allocate(DocumentNodeKind::Document);
        let page = store.insert_child(doc, DocumentNodeKind::Page).unwrap();
        let rect = store
            .insert_child(page, DocumentNodeKind::Rectangle)
            .unwrap();
        store.remove_subtree(page);
        assert!(store.get(page).is_none());
        assert!(store.get(rect).is_none());
    }

    #[test]
    fn root_id_is_document() {
        let mut store = NodeStore::new();
        let doc = store.allocate(DocumentNodeKind::Document);
        let root = store.root_id().unwrap();
        assert_eq!(root, doc);
        assert!(matches!(
            store.get(root).unwrap().kind,
            DocumentNodeKind::Document
        ));
    }

    #[test]
    fn layout_and_text_accessors() {
        let mut node = DocumentNode::new(
            reciplexa_identity::document::StableNodeId::new(1),
            DocumentNodeKind::Text,
        );
        assert!(node.layout().is_none());
        let layout = LayoutBox::new(1.0, 2.0, 3.0, 4.0);
        node.set_layout(layout);
        assert_eq!(node.layout(), Some(layout));
        node.properties
            .push(NodeProperty::Text(TextContent { text: "hi".into() }));
        assert_eq!(node.text().unwrap().text, "hi");
        node.set_layout(LayoutBox::new(5.0, 6.0, 7.0, 8.0));
        assert_eq!(node.layout().unwrap().x, 5.0);
    }

    #[test]
    fn insert_preserved_and_link_child() {
        let mut store = NodeStore::new();
        let doc = store.allocate(DocumentNodeKind::Document);
        let id = reciplexa_identity::document::StableNodeId::new(99);
        store.insert_preserved(DocumentNode::new(id, DocumentNodeKind::Page));
        assert_eq!(store.get(id).unwrap().kind, DocumentNodeKind::Page);
        store.link_child(doc, id);
        assert!(store.get(doc).unwrap().children.contains(&id));
    }

    #[test]
    fn insert_child_missing_parent_returns_none() {
        let mut store = NodeStore::new();
        let missing = reciplexa_identity::document::StableNodeId::new(42);
        assert!(store
            .insert_child(missing, DocumentNodeKind::Page)
            .is_none());
    }

    #[test]
    fn duplicate_subtree_copies_children() {
        let mut store = NodeStore::new();
        let doc = store.allocate(DocumentNodeKind::Document);
        let page = store.insert_child(doc, DocumentNodeKind::Page).unwrap();
        let rect = store
            .insert_child(page, DocumentNodeKind::Rectangle)
            .unwrap();
        let dup = store.duplicate_subtree(page).unwrap();
        assert_ne!(page, dup);
        assert_eq!(store.get(dup).unwrap().children.len(), 1);
        assert!(store.get(rect).is_some());
    }

    #[test]
    fn len_and_is_empty() {
        let mut store = NodeStore::new();
        assert!(store.is_empty());
        store.allocate(DocumentNodeKind::Document);
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());
    }
}
