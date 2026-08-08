use reciplexa_gui_runtime::selection::*;
use reciplexa_identity::document::StableNodeId;

#[test]
fn retain_existing_clears_deleted_primary() {
    let mut sel = NodeSelection::default();
    sel.select_only(StableNodeId::new(1));
    sel.add(StableNodeId::new(2));
    sel.retain_existing(|id| id.get() == 2);
    assert_eq!(sel.primary(), Some(StableNodeId::new(2)));
}

#[test]
fn add_dedupes_primary_and_additional() {
    let mut sel = NodeSelection::default();
    let id = StableNodeId::new(1);
    sel.select_only(id);
    sel.add(id);
    sel.add(StableNodeId::new(2));
    sel.add(StableNodeId::new(2));
    assert_eq!(sel.primary(), Some(id));
    let ids: Vec<_> = sel.ids().collect();
    assert_eq!(ids, vec![id, StableNodeId::new(2)]);
}

#[test]
fn ids_iterates_primary_then_additional() {
    let mut sel = NodeSelection::default();
    sel.select_only(StableNodeId::new(10));
    sel.add(StableNodeId::new(20));
    sel.add(StableNodeId::new(30));
    let ids: Vec<_> = sel.ids().collect();
    assert_eq!(ids.len(), 3);
    assert_eq!(ids[0], StableNodeId::new(10));
    assert_eq!(ids[1], StableNodeId::new(20));
    assert_eq!(ids[2], StableNodeId::new(30));
}

#[test]
fn retain_promotes_additional_when_primary_deleted() {
    let mut sel = NodeSelection::default();
    sel.select_only(StableNodeId::new(1));
    sel.add(StableNodeId::new(2));
    sel.add(StableNodeId::new(3));
    sel.retain_existing(|id| id.get() != 1);
    assert_eq!(sel.primary(), Some(StableNodeId::new(2)));
    let ids: Vec<_> = sel.ids().collect();
    assert_eq!(ids, vec![StableNodeId::new(2), StableNodeId::new(3)]);
}

#[test]
fn clear_removes_all() {
    let mut sel = NodeSelection::default();
    sel.select_only(StableNodeId::new(1));
    sel.add(StableNodeId::new(2));
    sel.clear();
    assert!(sel.primary().is_none());
    assert!(sel.ids().next().is_none());
    assert!(!sel.contains(StableNodeId::new(1)));
}

#[test]
fn contains_checks_primary_and_additional() {
    let mut sel = NodeSelection::default();
    assert!(!sel.contains(StableNodeId::new(1)));
    sel.select_only(StableNodeId::new(1));
    assert!(sel.contains(StableNodeId::new(1)));
    sel.add(StableNodeId::new(2));
    assert!(sel.contains(StableNodeId::new(2)));
}

#[test]
fn add_to_empty_sets_primary() {
    let mut sel = NodeSelection::default();
    sel.add(StableNodeId::new(7));
    assert_eq!(sel.primary(), Some(StableNodeId::new(7)));
}

#[test]
fn retain_keeps_existing_primary() {
    let mut sel = NodeSelection::default();
    sel.select_only(StableNodeId::new(1));
    sel.add(StableNodeId::new(2));
    sel.retain_existing(|id| id.get() == 1 || id.get() == 2);
    assert_eq!(sel.primary(), Some(StableNodeId::new(1)));
    let ids: Vec<_> = sel.ids().collect();
    assert_eq!(ids, vec![StableNodeId::new(1), StableNodeId::new(2)]);
}

#[test]
fn retain_empty_selection_is_noop() {
    let mut sel = NodeSelection::default();
    sel.retain_existing(|_| false);
    assert!(sel.primary().is_none());
    assert!(sel.ids().next().is_none());
}

#[test]
fn retain_drops_all_ids() {
    let mut sel = NodeSelection::default();
    sel.select_only(StableNodeId::new(1));
    sel.add(StableNodeId::new(2));
    sel.retain_existing(|_| false);
    assert!(sel.primary().is_none());
    assert!(sel.ids().next().is_none());
}

#[test]
fn retain_drops_additional_keeps_primary() {
    let mut sel = NodeSelection::default();
    sel.select_only(StableNodeId::new(1));
    sel.add(StableNodeId::new(2));
    sel.retain_existing(|id| id.get() == 1);
    assert_eq!(sel.primary(), Some(StableNodeId::new(1)));
    let ids: Vec<_> = sel.ids().collect();
    assert_eq!(ids, vec![StableNodeId::new(1)]);
}

#[test]
fn selection_clone_eq_and_debug() {
    let mut sel = NodeSelection::default();
    sel.select_only(StableNodeId::new(9));
    sel.add(StableNodeId::new(10));
    let cloned = sel.clone();
    assert_eq!(sel, cloned);
    assert_ne!(sel, NodeSelection::default());
    let dbg = format!("{sel:?}");
    assert!(dbg.contains("NodeSelection"));
}
