//! Register identifiers for ownership IR.

use std::fmt;

/// SSA-style register holding one owned value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reg(pub u32);

impl Reg {
    pub const INVALID: Reg = Reg(u32::MAX);

    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "r{}", self.0)
    }
}

/// Allocates fresh registers.
#[derive(Debug, Default)]
pub struct RegAlloc {
    next: u32,
}

impl RegAlloc {
    pub fn fresh(&mut self) -> Reg {
        let r = Reg(self.next);
        self.next += 1;
        r
    }
}
