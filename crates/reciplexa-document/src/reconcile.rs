//! Reentrancy guard for source↔document reconciliation (Phase 4 §6.6).

/// Prevents parse-during-drag feedback loops.
#[derive(Debug, Clone, Default)]
pub struct ReconcileGuard {
    depth: u32,
}

impl ReconcileGuard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enter(&mut self) -> bool {
        if self.depth > 0 {
            return false;
        }
        self.depth += 1;
        true
    }

    pub fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    pub fn is_active(&self) -> bool {
        self.depth > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
