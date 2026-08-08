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
        let keys: std::collections::HashSet<_> = self.states.keys().chain(other.states.keys()).collect();
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
