use reciplexa_gui_runtime::ime::*;
use reciplexa_identity::gui::WidgetKeyPath;

#[test]
fn ends_when_owner_removed() {
    let mut ime = ImeState::default();
    let k = WidgetKeyPath::new().push_named("field");
    ime.begin(k.clone());
    ime.set_preedit("あ");
    assert!(ime.end_if_owner(&k));
    assert!(ime.session.is_none());
}

#[test]
fn confirm_appends_and_clears_preedit() {
    let mut ime = ImeState::default();
    let k = WidgetKeyPath::new().push_named("field");
    ime.begin(k);
    ime.set_preedit("か");
    ime.confirm("ka");
    let s = ime.session.as_ref().unwrap();
    assert_eq!(s.confirmed, "ka");
    assert!(s.preedit.is_empty());
}

#[test]
fn set_preedit_without_session_is_noop() {
    let mut ime = ImeState::default();
    ime.set_preedit("ignored");
    assert!(ime.session.is_none());
}

#[test]
fn end_if_owner_false_for_non_owner() {
    let mut ime = ImeState::default();
    let owner = WidgetKeyPath::new().push_named("a");
    let other = WidgetKeyPath::new().push_named("b");
    ime.begin(owner);
    assert!(!ime.end_if_owner(&other));
    assert!(ime.session.is_some());
}

#[test]
fn end_clears_session() {
    let mut ime = ImeState::default();
    let k = WidgetKeyPath::new().push_named("field");
    ime.begin(k);
    ime.end();
    assert!(ime.session.is_none());
}

#[test]
fn confirm_without_session_is_noop() {
    let mut ime = ImeState::default();
    ime.confirm("ignored");
    assert!(ime.session.is_none());
}
