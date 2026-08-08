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
