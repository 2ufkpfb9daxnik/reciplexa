use reciplexa_gui_runtime::description::{GuiDescription, GuiNodeKind};
use reciplexa_gui_runtime::reconcile::*;
use reciplexa_gui_runtime::state::{MountedInstance, MountedTree, ViewState, WidgetState};
use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::gui::WidgetKeyPath;

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
    assert_eq!(result.mounted.get(&k).unwrap().kind, GuiNodeKind::TextField);
    assert_eq!(result.mounted.get(&k).unwrap().widget.text_buffer, "saved");
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

#[test]
fn partial_reorder_emits_move_only_for_changed_slots() {
    let prev = MountedTree::default();
    let old = GuiDescription::from_stable_nodes(&[
        (key("a"), StableNodeId::new(1), GuiNodeKind::Button),
        (key("b"), StableNodeId::new(2), GuiNodeKind::Button),
        (key("c"), StableNodeId::new(3), GuiNodeKind::Button),
    ]);
    let new = GuiDescription::from_stable_nodes(&[
        (key("a"), StableNodeId::new(1), GuiNodeKind::Button),
        (key("c"), StableNodeId::new(3), GuiNodeKind::Button),
        (key("b"), StableNodeId::new(2), GuiNodeKind::Button),
    ]);
    let result = reconcile(&prev, &old, &new).unwrap();
    let moves: Vec<_> = result
        .plan
        .ops
        .iter()
        .filter_map(|o| match o {
            ReconcileOp::MoveState { from, to } => Some((from.clone(), to.clone())),
            _ => None,
        })
        .collect();
    assert!(!moves.iter().any(|(f, t)| f == t));
    assert!(moves.iter().any(|(f, t)| f == &key("b") && t == &key("c")));
}

#[test]
fn same_length_different_keys_skips_move_state() {
    let prev = MountedTree::default();
    let old = GuiDescription::from_stable_nodes(&[
        (key("a"), StableNodeId::new(1), GuiNodeKind::Button),
        (key("b"), StableNodeId::new(2), GuiNodeKind::Button),
    ]);
    let new = GuiDescription::from_stable_nodes(&[
        (key("a"), StableNodeId::new(1), GuiNodeKind::Button),
        (key("c"), StableNodeId::new(3), GuiNodeKind::Button),
    ]);
    let result = reconcile(&prev, &old, &new).unwrap();
    assert!(!result
        .plan
        .ops
        .iter()
        .any(|o| matches!(o, ReconcileOp::MoveState { .. })));
}
