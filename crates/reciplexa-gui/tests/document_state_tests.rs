use reciplexa_gui::document_state::DocumentPathState;
use reciplexa_identity::document::StableNodeId;

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
    let mut state = DocumentPathState::disabled();
    state.layer_to_node = vec![StableNodeId::new(10), StableNodeId::new(20)];
    state.select_layer(999);
    assert!(state.selected_nodes.is_empty());
    state.select_layer(0);
    let id = state.selected_nodes[0];
    state.toggle_layer(0);
    assert!(state.selected_nodes.is_empty());
    state.toggle_layer(0);
    assert_eq!(state.selected_nodes, vec![id]);
    state.toggle_layer(1);
    assert_eq!(state.selected_nodes.len(), 2);
    state.toggle_layer(999);
    assert_eq!(state.selected_nodes.len(), 2);
    state.select_layers(&[0, 1, 999]);
    assert_eq!(state.selected_nodes.len(), 2);
    assert_eq!(state.primary_node(), state.selected_nodes.last().copied());
    state.clear_selection();
    assert!(state.selected_nodes.is_empty());
    assert!(state.primary_node().is_none());
    state.sync_selection_indices(&[]);
    assert!(state.selected_nodes.is_empty());
    state.select_layer(0);
    state.sync_selection_indices(&[0]);
    assert_eq!(state.selected_nodes.len(), 1);
    state.sync_selection_indices(&[0, 1, 50]);
    assert_eq!(state.selected_nodes.len(), 2);
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
