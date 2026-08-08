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
                if let Some(inst) = mounted.instances.get_mut(&node.key) {
                    inst.kind = node.kind;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::description::GuiNodeKind;
    use reciplexa_identity::document::StableNodeId;

    fn key(s: &str) -> WidgetKeyPath {
        WidgetKeyPath::new().push_named(s)
    }

    #[test]
    fn mounts_new_nodes() {
        let prev = MountedTree::default();
        let old = GuiDescription { roots: vec![] };
        let new = GuiDescription::from_stable_nodes(&[(
            key("layer-1"),
            StableNodeId::new(1),
            GuiNodeKind::LayerList,
        )]);
        let result = reconcile(&prev, &old, &new).unwrap();
        assert!(result
            .plan
            .ops
            .iter()
            .any(|o| matches!(o, ReconcileOp::Mount { .. })));
        assert!(result.mounted.get(&key("layer-1")).is_some());
    }

    #[test]
    fn unmounts_removed_nodes() {
        let mut prev = MountedTree::default();
        let k = key("a");
        prev.instances.insert(
            k.clone(),
            MountedInstance {
                key: k.clone(),
                kind: GuiNodeKind::Button,
                view: ViewState::default(),
                widget: WidgetState::default(),
            },
        );
        let old = GuiDescription::from_stable_nodes(&[(
            k.clone(),
            StableNodeId::new(1),
            GuiNodeKind::Button,
        )]);
        let new = GuiDescription { roots: vec![] };
        let result = reconcile(&prev, &old, &new).unwrap();
        assert!(result
            .plan
            .ops
            .iter()
            .any(|o| matches!(o, ReconcileOp::Unmount { .. })));
        assert!(result.mounted.get(&k).is_none());
    }

    #[test]
    fn rebase_caret_after_insert() {
        let mut ws = WidgetState {
            text_buffer: "hello".into(),
            caret_offset: 5,
            selection_start: None,
        };
        rebase_caret(&mut ws, "hello", "hello!", 5, 1);
        assert_eq!(ws.caret_offset, 6);
    }

    #[test]
    fn updates_existing_mounted() {
        let mut prev = MountedTree::default();
        let k = key("btn");
        prev.instances.insert(
            k.clone(),
            MountedInstance {
                key: k.clone(),
                kind: GuiNodeKind::Button,
                view: ViewState::default(),
                widget: WidgetState {
                    text_buffer: "saved".into(),
                    ..Default::default()
                },
            },
        );
        let old = GuiDescription::from_stable_nodes(&[(
            k.clone(),
            StableNodeId::new(1),
            GuiNodeKind::Button,
        )]);
        let new = GuiDescription::from_stable_nodes(&[(
            k.clone(),
            StableNodeId::new(1),
            GuiNodeKind::TextField,
        )]);
        let result = reconcile(&prev, &old, &new).unwrap();
        assert!(result
            .plan
            .ops
            .iter()
            .any(|o| matches!(o, ReconcileOp::Update { .. })));
        assert_eq!(
            result.mounted.get(&k).unwrap().kind,
            GuiNodeKind::TextField
        );
        assert_eq!(
            result.mounted.get(&k).unwrap().widget.text_buffer,
            "saved"
        );
    }

    #[test]
    fn remounts_when_key_in_old_but_not_mounted() {
        let prev = MountedTree::default();
        let k = key("ghost");
        let old = GuiDescription::from_stable_nodes(&[(
            k.clone(),
            StableNodeId::new(1),
            GuiNodeKind::Button,
        )]);
        let new = old.clone();
        let result = reconcile(&prev, &old, &new).unwrap();
        assert!(result
            .plan
            .ops
            .iter()
            .any(|o| matches!(o, ReconcileOp::Mount { .. })));
        assert!(result.mounted.get(&k).is_some());
    }

    #[test]
    fn key_collision_returns_error() {
        let prev = MountedTree::default();
        let old = GuiDescription { roots: vec![] };
        let dup = key("dup");
        let new = GuiDescription::from_stable_nodes(&[
            (dup.clone(), StableNodeId::new(1), GuiNodeKind::Button),
            (dup, StableNodeId::new(2), GuiNodeKind::Button),
        ]);
        let err = reconcile(&prev, &old, &new).unwrap_err();
        assert!(matches!(err, ReconcileError::KeyCollision(_)));
    }

    #[test]
    fn rebase_caret_before_edit_at_unchanged() {
        let mut ws = WidgetState {
            text_buffer: "abcdef".into(),
            caret_offset: 2,
            selection_start: Some(1),
        };
        rebase_caret(&mut ws, "abcdef", "abXcdef", 4, 1);
        assert_eq!(ws.caret_offset, 2);
        assert_eq!(ws.selection_start, Some(1));
    }

    #[test]
    fn rebase_caret_negative_delta_clamps_to_zero() {
        let mut ws = WidgetState {
            text_buffer: "ab".into(),
            caret_offset: 1,
            selection_start: Some(1),
        };
        rebase_caret(&mut ws, "ab", "a", 0, -5);
        assert_eq!(ws.caret_offset, 0);
        assert_eq!(ws.selection_start, Some(0));
    }

    #[test]
    fn rebase_selection_start_at_edit_point() {
        let mut ws = WidgetState {
            text_buffer: "hello".into(),
            caret_offset: 5,
            selection_start: Some(3),
        };
        rebase_caret(&mut ws, "hello", "helXlo", 3, 1);
        assert_eq!(ws.selection_start, Some(4));
    }

    #[test]
    fn orphan_state_variant_is_documented() {
        let err = ReconcileError::OrphanState(key("missing"));
        assert!(matches!(err, ReconcileError::OrphanState(_)));
    }
}
