use reciplexa_gui_runtime::description::{GuiDescription, GuiNodeKind};
use reciplexa_gui_runtime::lifecycle::*;
use reciplexa_gui_runtime::reconcile::{reconcile, ReconcileOp, ReconcilePlan, ReconcileResult};
use reciplexa_gui_runtime::state::{MountedInstance, MountedTree, ViewState, WidgetState};
use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::gui::WidgetKeyPath;

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
    assert!(validate_description(&old).is_ok());
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
        .filter(|o| matches!(o, ReconcileOp::MoveState { .. }))
        .count();
    assert!(move_count > 0);
}

#[test]
fn lifecycle_events_skip_ops_missing_from_description() {
    let ghost = key("ghost");
    let result = ReconcileResult {
        plan: ReconcilePlan {
            ops: vec![
                ReconcileOp::Mount { key: ghost.clone() },
                ReconcileOp::Update { key: ghost },
            ],
        },
        mounted: MountedTree::default(),
        kept_previous: false,
    };
    let events = lifecycle_events(
        &GuiDescription::default(),
        &GuiDescription::default(),
        &result,
    );
    assert!(events.is_empty());
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
