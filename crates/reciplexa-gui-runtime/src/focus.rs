//! Focus and pointer capture ownership.

use reciplexa_identity::gui::WidgetKeyPath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusOwner {
    pub key: WidgetKeyPath,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FocusState {
    pub focused: Option<FocusOwner>,
    pub pointer_capture: Option<WidgetKeyPath>,
}

impl FocusState {
    pub fn set_focus(&mut self, key: WidgetKeyPath) {
        self.focused = Some(FocusOwner { key });
    }

    pub fn clear_focus_if(&mut self, key: &WidgetKeyPath) {
        if self.focused.as_ref().is_some_and(|f| &f.key == key) {
            self.focused = None;
        }
        if self.pointer_capture.as_ref() == Some(key) {
            self.pointer_capture = None;
        }
    }

    pub fn capture_pointer(&mut self, key: WidgetKeyPath) {
        self.pointer_capture = Some(key);
    }

    pub fn release_pointer(&mut self, key: &WidgetKeyPath) {
        if self.pointer_capture.as_ref() == Some(key) {
            self.pointer_capture = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
