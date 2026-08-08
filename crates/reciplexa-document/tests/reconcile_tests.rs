use reciplexa_document::*;



#[test]
fn blocks_nested_enter() {
    let mut g = ReconcileGuard::new();
    assert!(g.enter());
    assert!(!g.enter());
    g.leave();
    assert!(g.enter());
}

#[test]
fn leave_saturates_and_is_active_tracks_depth() {
    let mut g = ReconcileGuard::new();
    assert!(!g.is_active());
    assert!(g.enter());
    assert!(g.is_active());
    g.leave();
    g.leave();
    assert!(!g.is_active());
    assert!(g.enter());
}
