use reciplexa_gui_runtime::focus::*;
use reciplexa_identity::gui::WidgetKeyPath;

fn key(s: &str) -> WidgetKeyPath {
    WidgetKeyPath::new().push_named(s)
}

#[test]
fn clear_focus_only_matching_owner() {
    let mut focus = FocusState::default();
    let owner = key("field-a");
    let other = key("field-b");
    focus.set_focus(owner.clone());
    focus.clear_focus_if(&other);
    assert_eq!(focus.focused.as_ref().unwrap().key, owner);
    focus.clear_focus_if(&owner);
    assert!(focus.focused.is_none());
}

#[test]
fn release_pointer_only_matching_owner() {
    let mut focus = FocusState::default();
    let owner = key("drag-a");
    let other = key("drag-b");
    focus.capture_pointer(owner.clone());
    focus.release_pointer(&other);
    assert_eq!(focus.pointer_capture.as_ref().unwrap(), &owner);
    focus.release_pointer(&owner);
    assert!(focus.pointer_capture.is_none());
}

#[test]
fn clear_focus_if_also_releases_matching_pointer() {
    let mut focus = FocusState::default();
    let k = key("combo");
    focus.set_focus(k.clone());
    focus.capture_pointer(k.clone());
    focus.clear_focus_if(&k);
    assert!(focus.focused.is_none());
    assert!(focus.pointer_capture.is_none());
}
