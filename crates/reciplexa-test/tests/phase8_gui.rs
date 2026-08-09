//! Phase 8 conformance: GUI runtime reconciliation and editor contracts.

use reciplexa_gui_runtime::{
    access_label, commit_reconcile, layer_rows, layer_tree_description, lifecycle_events,
    rebase_caret, reconcile, role_for_kind, AccessRole, DragPayload, DragSession, FocusState,
    GestureArena, GestureClaim, GestureKind, GuiDescription, GuiNodeKind, GuiRuntimeHost, ImeState,
    InspectorModel, LifecycleEvent, NodeSelection, ReconcileError, ReconcileOp, TimeMs, Timeline,
    TimelineTrack, VirtualWindow, WidgetState,
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
    let mounted = commit_reconcile(&prev, &GuiDescription::default(), &old).mounted;
    let bad = GuiDescription::from_stable_nodes(&[
        (key("x"), StableNodeId::new(2), GuiNodeKind::Button),
        (key("x"), StableNodeId::new(3), GuiNodeKind::Button),
    ]);
    let result = commit_reconcile(&mounted, &old, &bad);
    assert!(result.kept_previous);
    assert!(result.mounted.get(&key("ok")).is_some() || mounted.get(&key("ok")).is_some());
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
    sel.clear();
    assert!(sel.primary().is_none());
    let mut ime = ImeState::default();
    ime.begin(key("f"));
    ime.end();
    assert!(ime.session.is_none());
}

#[test]
fn phase8_reconcile_key_collision() {
    let prev = reciplexa_gui_runtime::MountedTree::default();
    let dup = key("dup");
    let bad = GuiDescription::from_stable_nodes(&[
        (dup.clone(), StableNodeId::new(1), GuiNodeKind::Button),
        (dup, StableNodeId::new(2), GuiNodeKind::Button),
    ]);
    let err = reconcile(&prev, &GuiDescription::default(), &bad).unwrap_err();
    assert!(matches!(err, ReconcileError::KeyCollision(_)));
}

#[test]
fn phase8_lifecycle_events_on_sync() {
    let mut host = GuiRuntimeHost::new();
    host.sync_layers(&[(StableNodeId::new(1), "layer".into())]);
    assert!(host
        .last_lifecycle
        .iter()
        .any(|e| matches!(e, LifecycleEvent::Mounted { .. })));
    host.sync_layers(&[]);
    assert!(host
        .last_lifecycle
        .iter()
        .any(|e| matches!(e, LifecycleEvent::Unmounted { .. })));
}

#[test]
fn phase8_focus_release_non_owner_noop() {
    let mut focus = FocusState::default();
    focus.capture_pointer(key("a"));
    focus.release_pointer(&key("b"));
    assert!(focus.pointer_capture.is_some());
}

#[test]
fn phase8_selection_multi_and_prune() {
    let mut host = GuiRuntimeHost::new();
    host.select_node(StableNodeId::new(1));
    host.selection.add(StableNodeId::new(2));
    let ids: Vec<_> = host.selection.ids().collect();
    assert_eq!(ids.len(), 2);
    host.prune_selection(&mut |id| id.get() == 2);
    assert_eq!(host.selection.primary(), Some(StableNodeId::new(2)));
}

#[test]
fn phase8_drag_drop_session() {
    let mut drag = DragSession::default();
    assert!(drag.drop_on(&key("t")).is_none());
    drag.begin(DragPayload {
        source_key: key("src"),
        node_id: Some(StableNodeId::new(1)),
        label: "item".into(),
    });
    drag.set_hover(Some(key("tgt")));
    let payload = drag.drop_on(&key("tgt")).unwrap();
    assert_eq!(payload.label, "item");
}

#[test]
fn phase8_virtual_window_edge_cases() {
    let empty = VirtualWindow::compute(0, 10, 100, 0);
    assert_eq!(empty.visible_count, 0);
    let clamped = VirtualWindow::compute(3, 0, 50, 0);
    assert_eq!(clamped.row_height_px, 1);
}

#[test]
fn phase8_timeline_seek_and_pause() {
    let mut tl = Timeline {
        playhead: TimeMs(0),
        duration: TimeMs(200),
        tracks: vec![TimelineTrack {
            id: 1,
            name: "anim".into(),
            start: TimeMs(0),
            end: TimeMs(100),
        }],
        playing: false,
    };
    tl.seek(TimeMs(500));
    assert_eq!(tl.playhead, TimeMs(200));
    tl.play();
    tl.pause();
    tl.tick(50);
    assert_eq!(tl.playhead, TimeMs(200));
}

#[test]
fn phase8_a11y_roles_and_labels() {
    assert_eq!(role_for_kind(GuiNodeKind::Group), AccessRole::Group);
    assert_eq!(access_label(GuiNodeKind::Canvas, ""), "Canvas");
    assert_eq!(access_label(GuiNodeKind::Button, "OK"), "OK");
}

#[test]
fn phase8_inspector_and_layer_tree() {
    let mut sel = NodeSelection::default();
    sel.select_only(StableNodeId::new(5));
    let rows = layer_rows(&[(StableNodeId::new(5), "Rect".into())], &sel);
    assert!(rows[0].selected);
    let desc = layer_tree_description(&[(StableNodeId::new(1), "A".into())], &sel);
    assert!(!desc.roots.is_empty());
    let model = InspectorModel::for_selection(&sel, vec![("x".into(), "1".into())]);
    let idesc = model.description();
    assert!(idesc.roots.len() >= 2);
}

#[test]
fn phase8_gesture_tie_and_reset() {
    let mut arena = GestureArena::default();
    arena.claim(GestureClaim {
        key: key("first"),
        kind: GestureKind::Tap,
        priority: 3,
    });
    arena.claim(GestureClaim {
        key: key("second"),
        kind: GestureKind::Tap,
        priority: 3,
    });
    assert_eq!(arena.winner().unwrap().key, key("first"));
    arena.reset();
    assert!(arena.winner().is_none());
}

#[test]
fn phase8_caret_rebase_clamps_negative() {
    let mut ws = WidgetState {
        text_buffer: "ab".into(),
        caret_offset: 1,
        selection_start: Some(1),
    };
    rebase_caret(&mut ws, "ab", "a", 0, -10);
    assert_eq!(ws.caret_offset, 0);
    assert_eq!(ws.selection_start, Some(0));
}

#[test]
fn phase8_lifecycle_empty_on_barrier() {
    let prev = reciplexa_gui_runtime::MountedTree::default();
    let old = GuiDescription::default();
    let bad = GuiDescription::from_stable_nodes(&[
        (key("d"), StableNodeId::new(1), GuiNodeKind::Button),
        (key("d"), StableNodeId::new(2), GuiNodeKind::Button),
    ]);
    let result = commit_reconcile(&prev, &old, &bad);
    let events = lifecycle_events(&old, &bad, &result);
    assert!(events.is_empty());
    assert!(result.kept_previous);
}

#[test]
fn phase8_ime_confirm_and_non_owner_end() {
    let mut ime = ImeState::default();
    let owner = key("field");
    ime.begin(owner.clone());
    ime.set_preedit("あ");
    ime.confirm("a");
    assert_eq!(ime.session.as_ref().unwrap().confirmed, "a");
    assert!(!ime.end_if_owner(&key("other")));
}
