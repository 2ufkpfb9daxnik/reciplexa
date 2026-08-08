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
