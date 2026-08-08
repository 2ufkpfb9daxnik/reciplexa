//! Mounted widget instances and ephemeral state.

use std::collections::HashMap;

use reciplexa_identity::gui::WidgetKeyPath;

use crate::description::GuiNodeKind;

/// Ephemeral view state (scroll, expansion).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ViewState {
    pub scroll_offset: f32,
    pub expanded: bool,
}

/// Per-widget ephemeral state keyed by stable path.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WidgetState {
    pub text_buffer: String,
    pub caret_offset: usize,
    pub selection_start: Option<usize>,
}

/// A mounted GUI instance with widget state.
#[derive(Debug, Clone, PartialEq)]
pub struct MountedInstance {
    pub key: WidgetKeyPath,
    pub kind: GuiNodeKind,
    pub view: ViewState,
    pub widget: WidgetState,
}

#[derive(Debug, Clone, Default)]
pub struct MountedTree {
    pub instances: HashMap<WidgetKeyPath, MountedInstance>,
}

impl MountedTree {
    pub fn get(&self, key: &WidgetKeyPath) -> Option<&MountedInstance> {
        self.instances.get(key)
    }

    pub fn get_mut(&mut self, key: &WidgetKeyPath) -> Option<&mut MountedInstance> {
        self.instances.get_mut(key)
    }
}
