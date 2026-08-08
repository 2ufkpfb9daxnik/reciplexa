//! Gesture arena — resolve competing pointer gestures (Phase 8).

use reciplexa_identity::gui::WidgetKeyPath;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GestureKind {
    Tap,
    Drag,
    Pan,
    Pinch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GestureClaim {
    pub key: WidgetKeyPath,
    pub kind: GestureKind,
    /// Higher wins; ties broken by claim order (earlier wins).
    pub priority: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GestureArena {
    claims: Vec<GestureClaim>,
    winner: Option<GestureClaim>,
}

impl GestureArena {
    pub fn reset(&mut self) {
        self.claims.clear();
        self.winner = None;
    }

    pub fn claim(&mut self, claim: GestureClaim) {
        self.claims.push(claim);
        self.resolve();
    }

    pub fn winner(&self) -> Option<&GestureClaim> {
        self.winner.as_ref()
    }

    fn resolve(&mut self) {
        self.winner = self
            .claims
            .iter()
            .enumerate()
            .max_by(|(i, a), (j, b)| {
                a.priority.cmp(&b.priority).then_with(|| j.cmp(i)) // earlier claim wins on tie
            })
            .map(|(_, c)| c.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
