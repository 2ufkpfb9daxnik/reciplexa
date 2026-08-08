use reciplexa_gui_runtime::a11y::*;
use reciplexa_gui_runtime::description::GuiNodeKind;

#[test]
fn maps_kinds_to_roles() {
    assert_eq!(role_for_kind(GuiNodeKind::TextField), AccessRole::TextBox);
    assert_eq!(role_for_kind(GuiNodeKind::Canvas), AccessRole::Canvas);
}

#[test]
fn all_gui_node_kinds_map_to_roles() {
    assert_eq!(role_for_kind(GuiNodeKind::Canvas), AccessRole::Canvas);
    assert_eq!(role_for_kind(GuiNodeKind::LayerList), AccessRole::List);
    assert_eq!(
        role_for_kind(GuiNodeKind::Properties),
        AccessRole::Properties
    );
    assert_eq!(role_for_kind(GuiNodeKind::TextField), AccessRole::TextBox);
    assert_eq!(role_for_kind(GuiNodeKind::Button), AccessRole::Button);
    assert_eq!(role_for_kind(GuiNodeKind::Group), AccessRole::Group);
}

#[test]
fn access_label_empty_uses_kind_debug() {
    let label = access_label(GuiNodeKind::Button, "");
    assert_eq!(label, "Button");
}

#[test]
fn access_label_non_empty_uses_fallback() {
    let label = access_label(GuiNodeKind::TextField, "Name field");
    assert_eq!(label, "Name field");
}
