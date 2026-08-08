//! Register ownership state during analysis and verification.

use std::collections::HashMap;

use crate::reg::Reg;

/// Ownership state of a register at a program point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnState {
    /// Register holds a valid owned or shared reference.
    Alive,
    /// Unique ownership was moved away; must not use.
    Moved,
    /// Value was dropped; must not use.
    Dropped,
    /// Never defined at this point.
    Uninit,
}

/// Map of register → ownership state.
#[derive(Debug, Clone, Default)]
pub struct OwnMap {
    states: HashMap<Reg, OwnState>,
}

impl OwnMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, reg: Reg) -> OwnState {
        self.states.get(&reg).copied().unwrap_or(OwnState::Uninit)
    }

    pub fn set(&mut self, reg: Reg, state: OwnState) {
        self.states.insert(reg, state);
    }

    pub fn define(&mut self, reg: Reg) {
        self.set(reg, OwnState::Alive);
    }

    pub fn move_from(&mut self, src: Reg, dst: Reg) {
        self.set(dst, OwnState::Alive);
        self.set(src, OwnState::Moved);
    }

    pub fn drop_reg(&mut self, reg: Reg) {
        self.set(reg, OwnState::Dropped);
    }

    pub fn alive_regs(&self) -> Vec<Reg> {
        self.states
            .iter()
            .filter(|(_, s)| **s == OwnState::Alive)
            .map(|(r, _)| *r)
            .collect()
    }

    pub fn merge_join(&self, other: &OwnMap) -> OwnMap {
        let mut out = OwnMap::new();
        let keys: std::collections::HashSet<_> =
            self.states.keys().chain(other.states.keys()).collect();
        for k in keys {
            let a = self.get(*k);
            let b = other.get(*k);
            let merged = match (a, b) {
                (OwnState::Uninit, s) | (s, OwnState::Uninit) => s,
                (OwnState::Alive, OwnState::Alive) => OwnState::Alive,
                (OwnState::Dropped, OwnState::Dropped) => OwnState::Dropped,
                _ => OwnState::Alive, // mismatch — verifier will flag
            };
            out.set(*k, merged);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_uninit() {
        let map = OwnMap::new();
        assert_eq!(map.get(Reg(0)), OwnState::Uninit);
    }

    #[test]
    fn define_and_drop() {
        let mut map = OwnMap::new();
        map.define(Reg(0));
        assert_eq!(map.get(Reg(0)), OwnState::Alive);
        map.drop_reg(Reg(0));
        assert_eq!(map.get(Reg(0)), OwnState::Dropped);
    }

    #[test]
    fn move_from_invalidates_source() {
        let mut map = OwnMap::new();
        map.define(Reg(0));
        map.move_from(Reg(0), Reg(1));
        assert_eq!(map.get(Reg(0)), OwnState::Moved);
        assert_eq!(map.get(Reg(1)), OwnState::Alive);
    }

    #[test]
    fn alive_regs_lists_only_alive() {
        let mut map = OwnMap::new();
        map.define(Reg(0));
        map.define(Reg(1));
        map.drop_reg(Reg(1));
        let alive = map.alive_regs();
        assert_eq!(alive, vec![Reg(0)]);
    }

    #[test]
    fn merge_join_combines_branches() {
        let mut left = OwnMap::new();
        left.define(Reg(0));
        left.drop_reg(Reg(1));
        let mut right = OwnMap::new();
        right.define(Reg(0));
        right.define(Reg(1));
        let merged = left.merge_join(&right);
        assert_eq!(merged.get(Reg(0)), OwnState::Alive);
        assert_eq!(merged.get(Reg(1)), OwnState::Alive);
    }

    #[test]
    fn merge_join_dropped_both() {
        let mut left = OwnMap::new();
        left.define(Reg(0));
        left.drop_reg(Reg(0));
        let mut right = OwnMap::new();
        right.define(Reg(0));
        right.drop_reg(Reg(0));
        let merged = left.merge_join(&right);
        assert_eq!(merged.get(Reg(0)), OwnState::Dropped);
    }
}
