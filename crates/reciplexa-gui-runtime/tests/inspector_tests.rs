use reciplexa_gui_runtime::description::GuiNodeKind;
use reciplexa_gui_runtime::inspector::*;
use reciplexa_gui_runtime::selection::NodeSelection;
use reciplexa_identity::document::StableNodeId;

#[test]
fn layer_rows_mark_selection() {
    let mut sel = NodeSelection::default();
    sel.select_only(StableNodeId::new(2));
    let rows = layer_rows(
        &[
            (StableNodeId::new(1), "a".into()),
            (StableNodeId::new(2), "b".into()),
        ],
        &sel,
    );
    assert!(!rows[0].selected);
    assert!(rows[1].selected);
}

#[test]
fn layer_tree_description_builds_nested_nodes() {
    let sel = NodeSelection::default();
    let desc = layer_tree_description(
        &[
            (StableNodeId::new(1), "Layer A".into()),
            (StableNodeId::new(2), "Layer B".into()),
        ],
        &sel,
    );
    assert_eq!(desc.roots.len(), 2);
    assert_eq!(desc.roots[0].kind, GuiNodeKind::LayerList);
    assert_eq!(desc.roots[0].children.len(), 1);
    assert_eq!(desc.roots[0].children[0].kind, GuiNodeKind::Button);
    assert!(!desc.roots[0].key.segments().is_empty());
}

#[test]
fn inspector_description_without_target() {
    let model = InspectorModel::default();
    let desc = model.description();
    assert_eq!(desc.roots.len(), 1);
    assert_eq!(desc.roots[0].kind, GuiNodeKind::Properties);
    assert!(desc.roots[0].stable_id.is_none());
}

#[test]
fn inspector_description_with_target_and_fields() {
    let mut sel = NodeSelection::default();
    sel.select_only(StableNodeId::new(42));
    let model = InspectorModel::for_selection(
        &sel,
        vec![
            ("width".into(), "100".into()),
            ("height".into(), "50".into()),
        ],
    );
    let desc = model.description();
    assert_eq!(desc.roots.len(), 3);
    assert_eq!(desc.roots[0].stable_id, Some(StableNodeId::new(42)));
    assert!(desc
        .roots
        .iter()
        .skip(1)
        .all(|n| n.kind == GuiNodeKind::TextField));
}
