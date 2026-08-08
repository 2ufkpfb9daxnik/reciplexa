use reciplexa_gui_runtime::description::GuiNodeKind;
use reciplexa_gui_runtime::state::*;
use reciplexa_identity::gui::WidgetKeyPath;

#[test]
fn get_miss_returns_none() {
    let tree = MountedTree::default();
    let k = WidgetKeyPath::new().push_named("absent");
    assert!(tree.get(&k).is_none());
}

#[test]
fn get_mut_miss_returns_none() {
    let mut tree = MountedTree::default();
    let k = WidgetKeyPath::new().push_named("absent");
    assert!(tree.get_mut(&k).is_none());
}

#[test]
fn get_mut_hit_returns_mutable_instance() {
    let mut tree = MountedTree::default();
    let k = WidgetKeyPath::new().push_named("present");
    tree.instances.insert(
        k.clone(),
        MountedInstance {
            key: k.clone(),
            kind: GuiNodeKind::Button,
            view: ViewState::default(),
            widget: WidgetState::default(),
        },
    );
    let inst = tree.get_mut(&k).unwrap();
    inst.widget.text_buffer = "edited".into();
    assert_eq!(tree.get(&k).unwrap().widget.text_buffer, "edited");
}
