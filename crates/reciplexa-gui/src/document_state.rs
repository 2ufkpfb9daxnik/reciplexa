//! Optional document-model path alongside CST sync (Phase 4 strangler).

use reciplexa::document_pipeline::document_snapshot_from_source;
use reciplexa_document::{drawable_node_ids, DocumentSnapshot};
use reciplexa_identity::document::{DocumentIdentity, StableNodeId};

/// Parallel state for the document-mediated editing path.
#[derive(Debug, Clone)]
pub struct DocumentPathState {
    pub enabled: bool,
    pub snapshot: Option<DocumentSnapshot>,
    /// Flatten index → stable node id (paint order).
    pub layer_to_node: Vec<StableNodeId>,
    pub selected_nodes: Vec<StableNodeId>,
}

impl DocumentPathState {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            snapshot: None,
            layer_to_node: Vec::new(),
            selected_nodes: Vec::new(),
        }
    }

    pub fn from_env() -> Self {
        let enabled = std::env::var("RECIPLEXA_DOCUMENT_PATH")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let mut state = Self::disabled();
        state.enabled = enabled;
        state
    }

    pub fn rebuild(&mut self, source: &str) {
        if !self.enabled {
            self.snapshot = None;
            self.layer_to_node.clear();
            self.selected_nodes.clear();
            return;
        }
        match document_snapshot_from_source(source, DocumentIdentity::new(1)) {
            Ok(snap) => {
                self.layer_to_node = drawable_node_ids(&snap.nodes);
                self.snapshot = Some(snap);
            }
            Err(_) => {
                self.snapshot = None;
                self.layer_to_node.clear();
            }
        }
        self.sync_selection_indices(&[]);
    }

    pub fn select_layer(&mut self, index: usize) {
        if let Some(id) = self.layer_to_node.get(index) {
            self.selected_nodes = vec![*id];
        } else {
            self.selected_nodes.clear();
        }
    }

    pub fn toggle_layer(&mut self, index: usize) {
        if let Some(id) = self.layer_to_node.get(index) {
            if let Some(pos) = self.selected_nodes.iter().position(|n| n == id) {
                self.selected_nodes.remove(pos);
            } else {
                self.selected_nodes.push(*id);
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
    }

    pub fn select_layers(&mut self, indices: &[usize]) {
        self.selected_nodes = indices
            .iter()
            .filter_map(|i| self.layer_to_node.get(*i).copied())
            .collect();
    }

    /// Keep stable ids when layer indices change (e.g. after delete).
    pub fn sync_selection_indices(&mut self, layer_indices: &[usize]) {
        if layer_indices.is_empty() {
            self.selected_nodes.clear();
            return;
        }
        self.selected_nodes = layer_indices
            .iter()
            .filter_map(|i| self.layer_to_node.get(*i).copied())
            .collect();
    }

    pub fn primary_node(&self) -> Option<StableNodeId> {
        self.selected_nodes.last().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebuild_maps_layers_to_stable_ids() {
        let mut state = DocumentPathState {
            enabled: true,
            ..DocumentPathState::disabled()
        };
        state.rebuild("(page a4 (rect 10 20 30 40))");
        assert!(state.snapshot.is_some());
        assert!(!state.layer_to_node.is_empty());
        state.select_layer(0);
        assert_eq!(state.selected_nodes.len(), 1);
    }

    #[test]
    fn disabled_rebuild_clears_state() {
        let mut state = DocumentPathState {
            enabled: true,
            ..DocumentPathState::disabled()
        };
        state.rebuild("(page a4 (circle 1 2 3))");
        assert!(state.snapshot.is_some());
        state.enabled = false;
        state.rebuild("(page a4 (circle 1 2 3))");
        assert!(state.snapshot.is_none());
        assert!(state.layer_to_node.is_empty());
        assert!(state.selected_nodes.is_empty());
    }

    #[test]
    fn rebuild_invalid_source_clears_snapshot() {
        let mut state = DocumentPathState {
            enabled: true,
            ..DocumentPathState::disabled()
        };
        state.rebuild("(page a4 (circle 1 2 3))");
        state.select_layer(0);
        state.rebuild("(page a4");
        assert!(state.snapshot.is_none());
        assert!(state.layer_to_node.is_empty());
    }

    #[test]
    fn selection_toggle_clear_and_bounds() {
        let mut state = DocumentPathState {
            enabled: true,
            ..DocumentPathState::disabled()
        };
        state.rebuild("(page a4 (circle 1 2 3) (rect 0 0 1 1))");
        assert!(state.layer_to_node.len() >= 1);
        state.select_layer(999);
        assert!(state.selected_nodes.is_empty());
        state.select_layer(0);
        let id = state.selected_nodes[0];
        state.toggle_layer(0);
        assert!(state.selected_nodes.is_empty());
        state.toggle_layer(0);
        assert_eq!(state.selected_nodes, vec![id]);
        if state.layer_to_node.len() > 1 {
            state.toggle_layer(1);
            assert_eq!(state.selected_nodes.len(), 2);
            state.select_layers(&[0, 1, 999]);
            assert_eq!(state.selected_nodes.len(), 2);
        }
        assert_eq!(state.primary_node(), state.selected_nodes.last().copied());
        state.clear_selection();
        assert!(state.selected_nodes.is_empty());
        assert!(state.primary_node().is_none());
        state.sync_selection_indices(&[]);
        assert!(state.selected_nodes.is_empty());
        state.select_layer(0);
        state.sync_selection_indices(&[0]);
        assert_eq!(state.selected_nodes.len(), 1);
    }

    #[test]
    fn from_env_reads_document_path_flag() {
        // Equivalence: unset / false-ish → disabled; "1"/"true" → enabled.
        unsafe {
            std::env::remove_var("RECIPLEXA_DOCUMENT_PATH");
        }
        assert!(!DocumentPathState::from_env().enabled);
        unsafe {
            std::env::set_var("RECIPLEXA_DOCUMENT_PATH", "0");
        }
        assert!(!DocumentPathState::from_env().enabled);
        unsafe {
            std::env::set_var("RECIPLEXA_DOCUMENT_PATH", "1");
        }
        assert!(DocumentPathState::from_env().enabled);
        unsafe {
            std::env::set_var("RECIPLEXA_DOCUMENT_PATH", "TRUE");
        }
        assert!(DocumentPathState::from_env().enabled);
        unsafe {
            std::env::remove_var("RECIPLEXA_DOCUMENT_PATH");
        }
    }
}
