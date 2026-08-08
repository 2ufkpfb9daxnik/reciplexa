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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reg_display() {
        assert_eq!(Reg(0).to_string(), "r0");
        assert_eq!(Reg(42).to_string(), "r42");
    }

    #[test]
    fn reg_invalid_constant() {
        assert_eq!(Reg::INVALID, Reg(u32::MAX));
    }

    #[test]
    fn reg_alloc_fresh_increments() {
        let mut alloc = RegAlloc::default();
        assert_eq!(alloc.fresh(), Reg(0));
        assert_eq!(alloc.fresh(), Reg(1));
        assert_eq!(Reg::new(7), Reg(7));
    }

    #[test]
    fn reg_ordering() {
        assert!(Reg(1) < Reg(2));
        assert_eq!(Reg(5), Reg(5));
    }
}
