//! IME session ownership (Phase 8 §10.2).

use reciplexa_identity::gui::WidgetKeyPath;

/// Active IME composition owned by a widget key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImeSession {
    pub owner: WidgetKeyPath,
    pub preedit: String,
    pub confirmed: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImeState {
    pub session: Option<ImeSession>,
}

impl ImeState {
    pub fn begin(&mut self, owner: WidgetKeyPath) {
        self.session = Some(ImeSession {
            owner,
            preedit: String::new(),
            confirmed: String::new(),
        });
    }

    pub fn set_preedit(&mut self, text: impl Into<String>) {
        if let Some(s) = &mut self.session {
            s.preedit = text.into();
        }
    }

    pub fn confirm(&mut self, text: impl Into<String>) {
        if let Some(s) = &mut self.session {
            s.confirmed.push_str(&text.into());
            s.preedit.clear();
        }
    }

    /// End IME safely when the owner widget is removed (§10.3).
    pub fn end_if_owner(&mut self, key: &WidgetKeyPath) -> bool {
        if self.session.as_ref().is_some_and(|s| &s.owner == key) {
            self.session = None;
            true
        } else {
            false
        }
    }

    pub fn end(&mut self) {
        self.session = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
