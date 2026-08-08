//! Commit barrier and component lifecycle for GUI runtime (Phase 8).

use reciplexa_identity::gui::WidgetKeyPath;

use crate::description::{GuiDescription, GuiNodeKind};
use crate::reconcile::{reconcile, ReconcileError, ReconcilePlan, ReconcileResult};
use crate::state::MountedTree;

/// Lifecycle events emitted when applying a reconcile plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleEvent {
    Mounted {
        key: WidgetKeyPath,
        kind: GuiNodeKind,
    },
    Updated {
        key: WidgetKeyPath,
        kind: GuiNodeKind,
    },
    Unmounted {
        key: WidgetKeyPath,
    },
}

/// Apply reconciliation behind a commit barrier.
///
/// On failure, the previous mounted tree is retained (`kept_previous = true`)
/// and no lifecycle events are emitted — §10.3.
pub fn commit_reconcile(
    previous: &MountedTree,
    old_desc: &GuiDescription,
    new_desc: &GuiDescription,
) -> ReconcileResult {
    if validate_description(new_desc).is_err() {
        return ReconcileResult {
            plan: ReconcilePlan { ops: Vec::new() },
            mounted: previous.clone(),
            kept_previous: true,
        };
    }
    match reconcile(previous, old_desc, new_desc) {
        Ok(result) => result,
        Err(_) => ReconcileResult {
            plan: ReconcilePlan { ops: Vec::new() },
            mounted: previous.clone(),
            kept_previous: true,
        },
    }
}

/// Derive lifecycle events from a successful reconcile result.
pub fn lifecycle_events(
    old_desc: &GuiDescription,
    new_desc: &GuiDescription,
    result: &ReconcileResult,
) -> Vec<LifecycleEvent> {
    if result.kept_previous {
        return Vec::new();
    }
    let mut events = Vec::new();
    for op in &result.plan.ops {
        match op {
            crate::reconcile::ReconcileOp::Mount { key } => {
                if let Some(n) = new_desc.find_by_key(key) {
                    events.push(LifecycleEvent::Mounted {
                        key: key.clone(),
                        kind: n.kind,
                    });
                }
            }
            crate::reconcile::ReconcileOp::Update { key } => {
                if let Some(n) = new_desc.find_by_key(key) {
                    events.push(LifecycleEvent::Updated {
                        key: key.clone(),
                        kind: n.kind,
                    });
                }
            }
            crate::reconcile::ReconcileOp::Unmount { key } => {
                let _ = old_desc;
                events.push(LifecycleEvent::Unmounted { key: key.clone() });
            }
            crate::reconcile::ReconcileOp::MoveState { .. } => {}
        }
    }
    events
}

/// Validate a description before commit (duplicate keys → barrier).
pub fn validate_description(desc: &GuiDescription) -> Result<(), ReconcileError> {
    let mut seen = std::collections::HashSet::new();
    for node in &desc.roots {
        if !seen.insert(node.key.clone()) {
            return Err(ReconcileError::KeyCollision(node.key.clone()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::description::GuiNodeKind;
    use crate::state::{MountedInstance, ViewState, WidgetState};
    use reciplexa_identity::document::StableNodeId;

    fn key(s: &str) -> WidgetKeyPath {
        WidgetKeyPath::new().push_named(s)
    }

    #[test]
    fn commit_barrier_keeps_previous_on_collision() {
        let mut prev = MountedTree::default();
        let k = key("ok");
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
        let bad = GuiDescription::from_stable_nodes(&[
            (key("a"), StableNodeId::new(2), GuiNodeKind::Button),
            (key("a"), StableNodeId::new(3), GuiNodeKind::Button),
        ]);
        assert!(validate_description(&bad).is_err());
        let result = commit_reconcile(&prev, &old, &bad);
        assert!(result.kept_previous);
        assert!(result.mounted.get(&k).is_some());
    }

    #[test]
    fn lifecycle_events_emit_mount_update_unmount() {
        let prev = MountedTree::default();
        let k_old = key("old");
        let k_new = key("new");
        let old = GuiDescription::from_stable_nodes(&[(
            k_old.clone(),
            StableNodeId::new(1),
            GuiNodeKind::Button,
        )]);
        let new = GuiDescription::from_stable_nodes(&[(
            k_new.clone(),
            StableNodeId::new(2),
            GuiNodeKind::TextField,
        )]);
        let result = reconcile(&prev, &old, &new).unwrap();
        let events = lifecycle_events(&old, &new, &result);
        assert!(events.iter().any(|e| matches!(
            e,
            LifecycleEvent::Unmounted { key } if *key == k_old
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            LifecycleEvent::Mounted { key, kind: GuiNodeKind::TextField } if *key == k_new
        )));
    }

    #[test]
    fn lifecycle_events_update_existing() {
        let mut prev = MountedTree::default();
        let k = key("w");
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
        let new = GuiDescription::from_stable_nodes(&[(
            k.clone(),
            StableNodeId::new(1),
            GuiNodeKind::Canvas,
        )]);
        let result = reconcile(&prev, &old, &new).unwrap();
        let events = lifecycle_events(&old, &new, &result);
        assert!(events.iter().any(|e| matches!(
            e,
            LifecycleEvent::Updated { key, kind: GuiNodeKind::Canvas } if *key == k
        )));
    }

    #[test]
    fn lifecycle_events_empty_when_kept_previous() {
        let result = ReconcileResult {
            plan: ReconcilePlan { ops: Vec::new() },
            mounted: MountedTree::default(),
            kept_previous: true,
        };
        let events = lifecycle_events(
            &GuiDescription::default(),
            &GuiDescription::default(),
            &result,
        );
        assert!(events.is_empty());
    }

    #[test]
    fn lifecycle_events_ignore_move_state() {
        let prev = MountedTree::default();
        let old = GuiDescription::from_stable_nodes(&[
            (key("a"), StableNodeId::new(1), GuiNodeKind::Button),
            (key("b"), StableNodeId::new(2), GuiNodeKind::Button),
        ]);
        let new = GuiDescription::from_stable_nodes(&[
            (key("b"), StableNodeId::new(2), GuiNodeKind::Button),
            (key("a"), StableNodeId::new(1), GuiNodeKind::Button),
        ]);
        let result = reconcile(&prev, &old, &new).unwrap();
        let events = lifecycle_events(&old, &new, &result);
        assert!(!events
            .iter()
            .any(|e| matches!(e, LifecycleEvent::Unmounted { .. })));
        let move_count = result
            .plan
            .ops
            .iter()
            .filter(|o| matches!(o, crate::reconcile::ReconcileOp::MoveState { .. }))
            .count();
        assert!(move_count > 0);
    }

    #[test]
    fn commit_reconcile_keeps_previous_on_reconcile_error() {
        let prev = MountedTree::default();
        let old = GuiDescription::default();
        let dup = key("dup");
        let bad = GuiDescription::from_stable_nodes(&[
            (dup.clone(), StableNodeId::new(1), GuiNodeKind::Button),
            (dup, StableNodeId::new(2), GuiNodeKind::Button),
        ]);
        let result = commit_reconcile(&prev, &old, &bad);
        assert!(result.kept_previous);
        assert!(result.plan.ops.is_empty());
    }
}
