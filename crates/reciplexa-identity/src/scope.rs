//! Scope identity for name resolution.

use core::fmt;

/// Stable identity for a lexical scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(pub u64);

impl ScopeId {
    pub const INVALID: Self = Self(0);
    pub const ROOT: Self = Self(1);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl fmt::Display for ScopeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "scope:{}", self.0)
    }
}

/// Allocates nested scope ids.
#[derive(Debug, Clone, Default)]
pub struct ScopeIdAllocator {
    next: u64,
}

impl ScopeIdAllocator {
    pub fn new() -> Self {
        Self { next: 2 }
    }

    pub fn allocate(&mut self) -> ScopeId {
        let id = ScopeId::new(self.next);
        self.next = self.next.saturating_add(1);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_scope_is_reserved() {
        assert!(ScopeId::ROOT.is_valid());
        assert_eq!(ScopeIdAllocator::new().allocate().get(), 2);
    }
}
