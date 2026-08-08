//! Accessibility labels and roles for GUI nodes (Phase 8).

use reciplexa_identity::gui::WidgetKeyPath;

use crate::description::GuiNodeKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessRole {
    Canvas,
    List,
    ListItem,
    Button,
    TextBox,
    Group,
    Properties,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessNode {
    pub key: WidgetKeyPath,
    pub role: AccessRole,
    pub name: String,
    pub selected: bool,
}

pub fn role_for_kind(kind: GuiNodeKind) -> AccessRole {
    match kind {
        GuiNodeKind::Canvas => AccessRole::Canvas,
        GuiNodeKind::LayerList => AccessRole::List,
        GuiNodeKind::Properties => AccessRole::Properties,
        GuiNodeKind::TextField => AccessRole::TextBox,
        GuiNodeKind::Button => AccessRole::Button,
        GuiNodeKind::Group => AccessRole::Group,
    }
}

pub fn access_label(kind: GuiNodeKind, fallback: &str) -> String {
    if fallback.is_empty() {
        format!("{kind:?}")
    } else {
        fallback.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_kinds_to_roles() {
        assert_eq!(role_for_kind(GuiNodeKind::TextField), AccessRole::TextBox);
        assert_eq!(role_for_kind(GuiNodeKind::Canvas), AccessRole::Canvas);
    }
}
