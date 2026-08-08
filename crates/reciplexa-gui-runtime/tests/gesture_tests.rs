use reciplexa_gui_runtime::gesture::*;
use reciplexa_identity::gui::WidgetKeyPath;

fn key(s: &str) -> WidgetKeyPath {
    WidgetKeyPath::new().push_named(s)
}

#[test]
fn higher_priority_wins() {
    let mut arena = GestureArena::default();
    arena.claim(GestureClaim {
        key: key("a"),
        kind: GestureKind::Tap,
        priority: 1,
    });
    arena.claim(GestureClaim {
        key: key("b"),
        kind: GestureKind::Drag,
        priority: 5,
    });
    assert_eq!(arena.winner().unwrap().key, key("b"));
}

#[test]
fn tie_prefers_earlier_claim() {
    let mut arena = GestureArena::default();
    arena.claim(GestureClaim {
        key: key("first"),
        kind: GestureKind::Tap,
        priority: 5,
    });
    arena.claim(GestureClaim {
        key: key("second"),
        kind: GestureKind::Drag,
        priority: 5,
    });
    assert_eq!(arena.winner().unwrap().key, key("first"));
}

#[test]
fn reset_clears_claims_and_winner() {
    let mut arena = GestureArena::default();
    arena.claim(GestureClaim {
        key: key("a"),
        kind: GestureKind::Pan,
        priority: 1,
    });
    assert!(arena.winner().is_some());
    arena.reset();
    assert!(arena.winner().is_none());
}
