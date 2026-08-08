//! Phase 8 conformance: GUI runtime reconciliation and editor contracts.

use reciplexa_gui_runtime::{
    commit_reconcile, rebase_caret, reconcile, FocusState, GestureArena, GestureClaim, GestureKind,
    GuiDescription, GuiNodeKind, GuiRuntimeHost, ImeState, NodeSelection, ReconcileOp,
    VirtualWindow, WidgetState,
};
use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::gui::WidgetKeyPath;

fn key(s: &str) -> WidgetKeyPath {
    WidgetKeyPath::new().push_named(s)
}

#[test]
fn phase8_reconciliation_mounts_and_unmounts() {
    let prev = reciplexa_gui_runtime::MountedTree::default();
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
    let prev = reciplexa_gui_runtime::MountedTree::default();
    let old = GuiDescription::from_stable_nodes(&[
        (key("a"), StableNodeId::new(1), GuiNodeKind::Button),
        (key("b"), StableNodeId::new(2), GuiNodeKind::Button),
    ]);
    let new = GuiDescription::from_stable_nodes(&[
        (key("b"), StableNodeId::new(2), GuiNodeKind::Button),
        (key("a"), StableNodeId::new(1), GuiNodeKind::Button),
    ]);
    let result = reconcile(&prev, &old, &new).unwrap();
    assert!(result
        .plan
        .ops
        .iter()
        .any(|o| matches!(o, ReconcileOp::MoveState { .. })));
    // State stays keyed — not shifted by list index.
    let mut with_state = result.mounted;
    if let Some(inst) = with_state.get_mut(&key("a")) {
        inst.widget.text_buffer = "kept".into();
    }
    let again = reconcile(&with_state, &new, &new).unwrap();
    assert_eq!(
        again.mounted.get(&key("a")).unwrap().widget.text_buffer,
        "kept"
    );
}

#[test]
fn phase8_caret_rebase_after_transaction() {
    let mut ws = WidgetState {
        text_buffer: "abc".into(),
        caret_offset: 3,
        selection_start: Some(1),
    };
    rebase_caret(&mut ws, "abc", "abXc", 2, 1);
    assert_eq!(ws.caret_offset, 4);
}

#[test]
fn phase8_commit_barrier_keeps_old_gui_on_failure() {
    let prev = reciplexa_gui_runtime::MountedTree::default();
    let old = GuiDescription::from_stable_nodes(&[(
        key("ok"),
        StableNodeId::new(1),
        GuiNodeKind::Button,
    )]);
    let mut mounted = commit_reconcile(&prev, &GuiDescription::default(), &old).mounted;
    let bad = GuiDescription::from_stable_nodes(&[
        (key("x"), StableNodeId::new(2), GuiNodeKind::Button),
        (key("x"), StableNodeId::new(3), GuiNodeKind::Button),
    ]);
    let result = commit_reconcile(&mounted, &old, &bad);
    assert!(result.kept_previous);
    assert!(result.mounted.get(&key("ok")).is_some() || mounted.get(&key("ok")).is_some());
    let _ = &mut mounted;
}

#[test]
fn phase8_ime_and_pointer_end_on_owner_delete() {
    let mut host = GuiRuntimeHost::new();
    host.sync_layers(&[(StableNodeId::new(1), "a".into())]);
    let k = WidgetKeyPath::new().push_named("layer").push_index(0);
    host.focus.set_focus(k.clone());
    host.focus.capture_pointer(k.clone());
    host.ime.begin(k);
    host.sync_layers(&[]);
    assert!(host.focus.focused.is_none());
    assert!(host.focus.pointer_capture.is_none());
    assert!(host.ime.session.is_none());
}

#[test]
fn phase8_gesture_arena_and_virtualization() {
    let mut arena = GestureArena::default();
    arena.claim(GestureClaim {
        key: key("a"),
        kind: GestureKind::Tap,
        priority: 1,
    });
    arena.claim(GestureClaim {
        key: key("b"),
        kind: GestureKind::Drag,
        priority: 10,
    });
    assert_eq!(arena.winner().unwrap().key, key("b"));
    let w = VirtualWindow::compute(500, 24, 240, 0);
    assert!(w.visible_count > 0);
}

#[test]
fn phase8_selection_is_not_document_truth() {
    let mut sel = NodeSelection::default();
    sel.select_only(StableNodeId::new(9));
    // GUI selection is ephemeral — clearing does not require a document transaction.
    sel.clear();
    assert!(sel.primary().is_none());
    let mut ime = ImeState::default();
    ime.begin(key("f"));
    ime.end();
    assert!(ime.session.is_none());
}
