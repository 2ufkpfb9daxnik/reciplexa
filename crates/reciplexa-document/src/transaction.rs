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
                .expect("parent existence checked above");
            store.get_mut(id).expect("just inserted").properties = properties.clone();
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
            let Some(n) = store.get_mut(*node) else {
                return Err(TransactionError::UnknownNode(*node));
            };
            let before = n.layout();
            n.set_layout(*layout);
            Ok(before != Some(*layout))
        }
        DocumentEdit::SetText { node, text } => {
            let Some(n) = store.get_mut(*node) else {
                return Err(TransactionError::UnknownNode(*node));
            };
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
                .expect("parent existence checked above");
            let idx = (*index).min(parent_node.children.len());
            parent_node.children.insert(idx, *node);
            store
                .get_mut(*node)
                .expect("node existence checked above")
                .parent = Some(*parent);
            Ok(true)
        }
    }
}
