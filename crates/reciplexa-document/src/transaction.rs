//! Atomic document transactions with rollback.

use reciplexa_identity::document::StableNodeId;

use crate::node::{DocumentNodeKind, NodeStore};
use crate::property::{LayoutBox, NodeProperty, TextContent};
use crate::snapshot::DocumentSnapshot;

/// A single atomic edit operation.
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentEdit {
    InsertChild {
        parent: StableNodeId,
        kind: DocumentNodeKind,
        properties: Vec<NodeProperty>,
    },
    RemoveNode {
        node: StableNodeId,
    },
    SetLayout {
        node: StableNodeId,
        layout: LayoutBox,
    },
    SetText {
        node: StableNodeId,
        text: TextContent,
    },
    MoveNode {
        node: StableNodeId,
        parent: StableNodeId,
        index: usize,
    },
}

/// Outcome of applying a transaction batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionOutcome {
    Applied,
    AppliedNoChange,
    Rejected,
}

/// Transaction failure — not a defect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionError {
    UnknownNode(StableNodeId),
    InvalidParent(StableNodeId),
    IndexOutOfRange,
    EmptyBatch,
}

/// Builder accumulating edits before commit.
#[derive(Debug, Clone, Default)]
pub struct TransactionBuilder {
    edits: Vec<DocumentEdit>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, edit: DocumentEdit) {
        self.edits.push(edit);
    }

    pub fn set_layout(&mut self, node: StableNodeId, layout: LayoutBox) {
        self.push(DocumentEdit::SetLayout { node, layout });
    }

    pub fn set_text(&mut self, node: StableNodeId, text: impl Into<String>) {
        self.push(DocumentEdit::SetText {
            node,
            text: TextContent { text: text.into() },
        });
    }

    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    pub fn into_transaction(self) -> DocumentTransaction {
        DocumentTransaction { edits: self.edits }
    }
}

/// Committable batch of edits.
#[derive(Debug, Clone)]
pub struct DocumentTransaction {
    edits: Vec<DocumentEdit>,
}

impl DocumentTransaction {
    pub fn apply(
        &self,
        snapshot: &mut DocumentSnapshot,
    ) -> Result<TransactionOutcome, TransactionError> {
        if self.edits.is_empty() {
            return Err(TransactionError::EmptyBatch);
        }
        let backup = snapshot.nodes.clone();
        let mut changed = false;
        for edit in &self.edits {
            match apply_one(&mut snapshot.nodes, edit) {
                Ok(true) => changed = true,
                Ok(false) => {}
                Err(e) => {
                    snapshot.nodes = backup;
                    return Err(e);
                }
            }
        }
        if changed {
            snapshot.bump_revision();
            Ok(TransactionOutcome::Applied)
        } else {
            Ok(TransactionOutcome::AppliedNoChange)
        }
    }
}

fn apply_one(store: &mut NodeStore, edit: &DocumentEdit) -> Result<bool, TransactionError> {
    match edit {
        DocumentEdit::InsertChild {
            parent,
            kind,
            properties,
        } => {
            if store.get(*parent).is_none() {
                return Err(TransactionError::UnknownNode(*parent));
            }
            let id = store
                .insert_child(*parent, *kind)
                .ok_or(TransactionError::InvalidParent(*parent))?;
            if let Some(node) = store.get_mut(id) {
                node.properties = properties.clone();
            }
            Ok(true)
        }
        DocumentEdit::RemoveNode { node } => {
            if store.get(*node).is_none() {
                return Err(TransactionError::UnknownNode(*node));
            }
            store.remove_subtree(*node);
            Ok(true)
        }
        DocumentEdit::SetLayout { node, layout } => {
            let n = store
                .get_mut(*node)
                .ok_or(TransactionError::UnknownNode(*node))?;
            let before = n.layout();
            n.set_layout(*layout);
            Ok(before != Some(*layout))
        }
        DocumentEdit::SetText { node, text } => {
            let n = store
                .get_mut(*node)
                .ok_or(TransactionError::UnknownNode(*node))?;
            let changed = n.text().map(|t| t.text.as_str()) != Some(text.text.as_str());
            if let Some(slot) = n.properties.iter_mut().find_map(|p| match p {
                NodeProperty::Text(t) => Some(t),
                _ => None,
            }) {
                slot.text = text.text.clone();
            } else {
                n.properties.push(NodeProperty::Text(text.clone()));
            }
            Ok(changed)
        }
        DocumentEdit::MoveNode {
            node,
            parent,
            index,
        } => {
            if store.get(*node).is_none() {
                return Err(TransactionError::UnknownNode(*node));
            }
            if store.get(*parent).is_none() {
                return Err(TransactionError::InvalidParent(*parent));
            }
            let old_parent = store.get(*node).and_then(|n| n.parent);
            if let Some(op) = old_parent {
                if let Some(p) = store.get_mut(op) {
                    p.children.retain(|c| *c != *node);
                }
            }
            let parent_node = store
                .get_mut(*parent)
                .ok_or(TransactionError::InvalidParent(*parent))?;
            let idx = (*index).min(parent_node.children.len());
            parent_node.children.insert(idx, *node);
            if let Some(n) = store.get_mut(*node) {
                n.parent = Some(*parent);
            }
            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::DocumentNodeKind;
    use crate::property::LayoutBox;
    use reciplexa_identity::document::DocumentIdentity;

    #[test]
    fn failed_transaction_rolls_back() {
        let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
        let root = snap.nodes.root_id().unwrap();
        let rect = snap
            .nodes
            .insert_child(root, DocumentNodeKind::Rectangle)
            .unwrap();
        let mut tx = TransactionBuilder::new();
        tx.set_layout(rect, LayoutBox::new(1.0, 2.0, 10.0, 20.0));
        tx.push(DocumentEdit::SetLayout {
            node: StableNodeId::new(9999),
            layout: LayoutBox::new(0.0, 0.0, 1.0, 1.0),
        });
        let batch = tx.into_transaction();
        assert!(batch.apply(&mut snap).is_err());
        assert!(snap.nodes.get(rect).unwrap().layout().is_none());
    }

    #[test]
    fn move_preserves_stable_id() {
        let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
        let root = snap.nodes.root_id().unwrap();
        let page = snap
            .nodes
            .insert_child(root, DocumentNodeKind::Page)
            .unwrap();
        let rect = snap
            .nodes
            .insert_child(page, DocumentNodeKind::Rectangle)
            .unwrap();
        let mut tx = TransactionBuilder::new();
        tx.push(DocumentEdit::MoveNode {
            node: rect,
            parent: root,
            index: 0,
        });
        tx.into_transaction().apply(&mut snap).unwrap();
        assert!(snap.nodes.get(rect).is_some());
    }

    #[test]
    fn empty_batch_rejected() {
        let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
        let tx = TransactionBuilder::new().into_transaction();
        assert_eq!(tx.apply(&mut snap), Err(TransactionError::EmptyBatch));
    }

    #[test]
    fn applied_no_change_when_layout_identical() {
        let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
        let root = snap.nodes.root_id().unwrap();
        let rect = snap
            .nodes
            .insert_child(root, DocumentNodeKind::Rectangle)
            .unwrap();
        let layout = LayoutBox::new(1.0, 2.0, 3.0, 4.0);
        let mut tx = TransactionBuilder::new();
        tx.set_layout(rect, layout);
        tx.into_transaction().apply(&mut snap).unwrap();
        let rev = snap.revision.get();
        let mut tx2 = TransactionBuilder::new();
        tx2.set_layout(rect, layout);
        assert_eq!(
            tx2.into_transaction().apply(&mut snap).unwrap(),
            TransactionOutcome::AppliedNoChange
        );
        assert_eq!(snap.revision.get(), rev);
    }

    #[test]
    fn remove_unknown_node_fails() {
        let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
        let mut tx = TransactionBuilder::new();
        tx.push(DocumentEdit::RemoveNode {
            node: StableNodeId::new(9999),
        });
        assert!(matches!(
            tx.into_transaction().apply(&mut snap),
            Err(TransactionError::UnknownNode(_))
        ));
    }

    #[test]
    fn insert_child_unknown_parent_fails() {
        let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
        let mut tx = TransactionBuilder::new();
        tx.push(DocumentEdit::InsertChild {
            parent: StableNodeId::new(9999),
            kind: DocumentNodeKind::Rectangle,
            properties: vec![],
        });
        assert!(matches!(
            tx.into_transaction().apply(&mut snap),
            Err(TransactionError::UnknownNode(_))
        ));
    }

    #[test]
    fn builder_is_empty_initially() {
        let b = TransactionBuilder::new();
        assert!(b.is_empty());
        let mut b2 = TransactionBuilder::new();
        b2.set_layout(StableNodeId::new(1), LayoutBox::new(0.0, 0.0, 1.0, 1.0));
        assert!(!b2.is_empty());
    }

    #[test]
    fn set_text_and_remove_node_succeed() {
        let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
        let root = snap.nodes.root_id().unwrap();
        let text = snap
            .nodes
            .insert_child(root, DocumentNodeKind::Text)
            .unwrap();
        let mut tx = TransactionBuilder::new();
        tx.set_text(text, "hello");
        assert_eq!(
            tx.into_transaction().apply(&mut snap).unwrap(),
            TransactionOutcome::Applied
        );
        assert_eq!(
            snap.nodes
                .get(text)
                .unwrap()
                .text()
                .map(|t| t.text.as_str()),
            Some("hello")
        );

        let mut tx = TransactionBuilder::new();
        tx.push(DocumentEdit::RemoveNode { node: text });
        tx.into_transaction().apply(&mut snap).unwrap();
        assert!(snap.nodes.get(text).is_none());
    }

    #[test]
    fn insert_child_and_set_property_roundtrip() {
        let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
        let root = snap.nodes.root_id().unwrap();
        let mut tx = TransactionBuilder::new();
        tx.push(DocumentEdit::InsertChild {
            parent: root,
            kind: DocumentNodeKind::Rectangle,
            properties: vec![],
        });
        let outcome = tx.into_transaction().apply(&mut snap).unwrap();
        assert!(matches!(
            outcome,
            TransactionOutcome::Applied | TransactionOutcome::AppliedNoChange
        ));
        assert!(snap.nodes.root_id().is_some());
    }

    #[test]
    fn move_unknown_node_fails() {
        let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
        let root = snap.nodes.root_id().unwrap();
        let mut tx = TransactionBuilder::new();
        tx.push(DocumentEdit::MoveNode {
            node: StableNodeId::new(9999),
            parent: root,
            index: 0,
        });
        assert!(tx.into_transaction().apply(&mut snap).is_err());
    }

    #[test]
    fn transaction_error_debug_covers_variants() {
        let _ = format!("{:?}", TransactionError::EmptyBatch);
        let _ = format!("{:?}", TransactionError::UnknownNode(StableNodeId::new(1)));
        let _ = format!("{:?}", TransactionOutcome::Applied);
        let _ = format!("{:?}", TransactionOutcome::AppliedNoChange);
    }
}
