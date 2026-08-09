use reciplexa_gui_runtime::description::{GuiDescription, GuiNodeKind};
use reciplexa_gui_runtime::host::*;
use reciplexa_gui_runtime::lifecycle::commit_reconcile;
use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::gui::WidgetKeyPath;

#[test]
fn sync_layers_mounts_and_clears_focus_on_remove() {
    let mut host = GuiRuntimeHost::new();
    host.sync_layers(&[(StableNodeId::new(1), "a".into())]);
    let k = WidgetKeyPath::new().push_named("layer").push_index(0);
    host.focus.set_focus(k.clone());
    host.ime.begin(k);
    host.sync_layers(&[]);
    assert!(host.focus.focused.is_none());
    assert!(host.ime.session.is_none());
    assert!(host.mounted.instances.is_empty());
}

#[test]
fn select_node_sets_primary() {
    let mut host = GuiRuntimeHost::new();
    let id = StableNodeId::new(7);
    host.select_node(id);
    assert_eq!(host.selection.primary(), Some(id));
}

#[test]
fn prune_selection_drops_missing_ids() {
    let mut host = GuiRuntimeHost::new();
    host.select_node(StableNodeId::new(1));
    host.selection.add(StableNodeId::new(2));
    host.prune_selection(&mut |id| id.get() == 2);
    assert_eq!(host.selection.primary(), Some(StableNodeId::new(2)));
}

#[test]
fn barrier_keeps_description_on_collision() {
    let mut host = GuiRuntimeHost::new();
    host.sync_layers(&[(StableNodeId::new(1), "a".into())]);
    let good_desc = host.description.clone();
    // Force a bad reconcile via duplicate keys in a custom description path:
    // sync_layers always produces valid keys, so we test via commit barrier directly.
    let bad = GuiDescription::from_stable_nodes(&[
        (
            WidgetKeyPath::new().push_named("dup"),
            StableNodeId::new(2),
            GuiNodeKind::Button,
        ),
        (
            WidgetKeyPath::new().push_named("dup"),
            StableNodeId::new(3),
            GuiNodeKind::Button,
        ),
    ]);
    let result = commit_reconcile(&host.mounted, &good_desc, &bad);
    host.apply_reconcile(&good_desc, &bad, result);
    assert_eq!(host.description, good_desc);
    assert!(
        host.mounted
            .get(&WidgetKeyPath::new().push_named("layer").push_index(0))
            .is_some()
            || host.mounted.instances.len() == 1
    );
}
