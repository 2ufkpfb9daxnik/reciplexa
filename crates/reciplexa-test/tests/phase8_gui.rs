//! Phase 8 conformance: GUI runtime reconciliation.

use reciplexa_gui_runtime::{
    reconcile, FocusState, GuiDescription, GuiNodeKind, ReconcileOp, WidgetState,
};
use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::gui::WidgetKeyPath;

fn key(s: &str) -> WidgetKeyPath {
    WidgetKeyPath::new().push_named(s)
}

#[test]
fn phase8_reconciliation_mounts_and_unmounts() {
    let prev = reciplexa_gui_runtime::state::MountedTree::default();
    let old = GuiDescription { roots: vec![] };
    let new = GuiDescription::from_stable_nodes(&[(
        key("canvas"),
        StableNodeId::new(1),
        GuiNodeKind::Canvas,
    )]);
    let result = reconcile(&prev, &old, &new).unwrap();
    assert!(result.mounted.get(&key("canvas")).is_some());
    let old2 = new;
    let new2 = GuiDescription { roots: vec![] };
    let result2 = reconcile(&result.mounted, &old2, &new2).unwrap();
    assert!(result2.mounted.get(&key("canvas")).is_none());
}

#[test]
fn phase8_focus_cleared_when_owner_removed() {
    let mut focus = FocusState::default();
    let k = key("text-field");
    focus.set_focus(k.clone());
    focus.capture_pointer(k.clone());
    focus.clear_focus_if(&k);
    assert!(focus.focused.is_none());
    assert!(focus.pointer_capture.is_none());
}

#[test]
fn phase8_list_reorder_preserves_state_by_key() {
    let prev = reciplexa_gui_runtime::state::MountedTree::default();
    let old = GuiDescription::from_stable_nodes(&[
        (key("a"), StableNodeId::new(1), GuiNodeKind::Button),
        (key("b"), StableNodeId::new(2), GuiNodeKind::Button),
    ]);
    let new = GuiDescription::from_stable_nodes(&[
        (key("b"), StableNodeId::new(2), GuiNodeKind::Button),
        (key("a"), StableNodeId::new(1), GuiNodeKind::Button),
    ]);
    let result = reconcile(&prev, &old, &new).unwrap();
    assert!(result.plan.ops.iter().any(|o| matches!(o, ReconcileOp::MoveState { .. })));
}

#[test]
fn phase8_caret_rebase_after_transaction() {
    use reciplexa_gui_runtime::reconcile::rebase_caret;
    let mut ws = WidgetState {
        text_buffer: "abc".into(),
        caret_offset: 3,
        selection_start: Some(1),
    };
    rebase_caret(&mut ws, "abc", "abXc", 2, 1);
    assert_eq!(ws.caret_offset, 4);
}
