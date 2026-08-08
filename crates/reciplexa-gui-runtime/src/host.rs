//! Aggregated GUI runtime host used by the editor (Phase 8).

use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::gui::WidgetKeyPath;

use crate::description::{GuiDescription, GuiNodeKind};
use crate::dragdrop::DragSession;
use crate::focus::FocusState;
use crate::gesture::GestureArena;
use crate::ime::ImeState;
use crate::lifecycle::{commit_reconcile, lifecycle_events, LifecycleEvent};
use crate::reconcile::ReconcileResult;
use crate::selection::NodeSelection;
use crate::state::MountedTree;
use crate::timeline::Timeline;

/// Ephemeral GUI runtime — never the document source of truth (§10.3).
#[derive(Debug, Clone, Default)]
pub struct GuiRuntimeHost {
    pub mounted: MountedTree,
    pub description: GuiDescription,
    pub focus: FocusState,
    pub ime: ImeState,
    pub selection: NodeSelection,
    pub gestures: GestureArena,
    pub drag: DragSession,
    pub timeline: Timeline,
    pub last_lifecycle: Vec<LifecycleEvent>,
}

impl GuiRuntimeHost {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rebuild description from document-stable layer ids and reconcile.
    pub fn sync_layers(&mut self, layers: &[(StableNodeId, String)]) {
        let new_desc = GuiDescription::from_stable_nodes(
            &layers
                .iter()
                .enumerate()
                .map(|(i, (id, _))| {
                    (
                        WidgetKeyPath::new()
                            .push_named("layer")
                            .push_index(i as u32),
                        *id,
                        GuiNodeKind::LayerList,
                    )
                })
                .collect::<Vec<_>>(),
        );
        let old = self.description.clone();
        let result = commit_reconcile(&self.mounted, &old, &new_desc);
        self.apply_reconcile(&old, &new_desc, result);
    }

    pub fn apply_reconcile(
        &mut self,
        old: &GuiDescription,
        new: &GuiDescription,
        result: ReconcileResult,
    ) {
        self.last_lifecycle = lifecycle_events(old, new, &result);
        for ev in &self.last_lifecycle {
            if let LifecycleEvent::Unmounted { key } = ev {
                self.focus.clear_focus_if(key);
                self.ime.end_if_owner(key);
                self.focus.release_pointer(key);
            }
        }
        if !result.kept_previous {
            self.mounted = result.mounted;
            self.description = new.clone();
        }
    }

    pub fn select_node(&mut self, id: StableNodeId) {
        self.selection.select_only(id);
    }

    pub fn prune_selection<F>(&mut self, exists: F)
    where
        F: FnMut(StableNodeId) -> bool,
    {
        self.selection.retain_existing(exists);
    }
}
