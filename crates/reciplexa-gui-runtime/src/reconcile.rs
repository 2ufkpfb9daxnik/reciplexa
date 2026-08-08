//! Reconciliation plan between GUI descriptions.

use reciplexa_identity::gui::WidgetKeyPath;

use crate::description::GuiDescription;
use crate::state::{MountedInstance, MountedTree, ViewState, WidgetState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileOp {
    Mount {
        key: WidgetKeyPath,
    },
    Unmount {
        key: WidgetKeyPath,
    },
    Update {
        key: WidgetKeyPath,
    },
    MoveState {
        from: WidgetKeyPath,
        to: WidgetKeyPath,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcilePlan {
    pub ops: Vec<ReconcileOp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileError {
    KeyCollision(WidgetKeyPath),
    OrphanState(WidgetKeyPath),
}

#[derive(Debug, Clone)]
pub struct ReconcileResult {
    pub plan: ReconcilePlan,
    pub mounted: MountedTree,
    pub kept_previous: bool,
}

/// Compute reconciliation between old and new GUI descriptions.
pub fn reconcile(
    previous: &MountedTree,
    old_desc: &GuiDescription,
    new_desc: &GuiDescription,
) -> Result<ReconcileResult, ReconcileError> {
    let mut seen = std::collections::HashSet::new();
    for node in &new_desc.roots {
        if !seen.insert(node.key.clone()) {
            return Err(ReconcileError::KeyCollision(node.key.clone()));
        }
    }

    let mut ops = Vec::new();
    let mut mounted = previous.clone();
    let old_keys: Vec<_> = old_desc.roots.iter().map(|n| n.key.clone()).collect();
    let new_keys: Vec<_> = new_desc.roots.iter().map(|n| n.key.clone()).collect();

    for key in &old_keys {
        if !new_keys.contains(key) {
            ops.push(ReconcileOp::Unmount { key: key.clone() });
            mounted.instances.remove(key);
        }
    }

    for node in &new_desc.roots {
        if old_keys.contains(&node.key) {
            if mounted.instances.contains_key(&node.key) {
                ops.push(ReconcileOp::Update {
                    key: node.key.clone(),
                });
                // `contains_key` just succeeded; update in place.
                mounted
                    .instances
                    .get_mut(&node.key)
                    .expect("key present")
                    .kind = node.kind;
            } else {
                ops.push(ReconcileOp::Mount {
                    key: node.key.clone(),
                });
                mounted.instances.insert(
                    node.key.clone(),
                    MountedInstance {
                        key: node.key.clone(),
                        kind: node.kind,
                        view: ViewState::default(),
                        widget: WidgetState::default(),
                    },
                );
            }
        } else {
            ops.push(ReconcileOp::Mount {
                key: node.key.clone(),
            });
            mounted.instances.insert(
                node.key.clone(),
                MountedInstance {
                    key: node.key.clone(),
                    kind: node.kind,
                    view: ViewState::default(),
                    widget: WidgetState::default(),
                },
            );
        }
    }

    // Detect list reorder: same keys, different order — move state with key
    if old_keys.len() == new_keys.len() && old_keys != new_keys {
        let old_set: std::collections::HashSet<_> = old_keys.iter().cloned().collect();
        let new_set: std::collections::HashSet<_> = new_keys.iter().cloned().collect();
        if old_set == new_set {
            for (old_k, new_k) in old_keys.iter().zip(new_keys.iter()) {
                if old_k != new_k {
                    ops.push(ReconcileOp::MoveState {
                        from: old_k.clone(),
                        to: new_k.clone(),
                    });
                }
            }
        }
    }

    Ok(ReconcileResult {
        plan: ReconcilePlan { ops },
        mounted,
        kept_previous: false,
    })
}

/// Rebase text caret after a document transaction.
pub fn rebase_caret(
    state: &mut WidgetState,
    old_text: &str,
    new_text: &str,
    edit_at: usize,
    delta: isize,
) {
    let _ = (old_text, new_text);
    if state.caret_offset >= edit_at {
        state.caret_offset = (state.caret_offset as isize + delta).max(0) as usize;
    }
    if let Some(sel) = state.selection_start {
        if sel >= edit_at {
            state.selection_start = Some((sel as isize + delta).max(0) as usize);
        }
    }
}
