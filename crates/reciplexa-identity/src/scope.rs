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

    #[test]
    fn allocator_issues_unique_scopes() {
        let mut alloc = ScopeIdAllocator::new();
        assert_ne!(alloc.allocate(), alloc.allocate());
    }

    #[test]
    fn scope_id_boundaries_and_display() {
        assert!(!ScopeId::INVALID.is_valid());
        assert_eq!(ScopeId::INVALID.get(), 0);
        assert_eq!(ScopeId::ROOT.get(), 1);
        assert!(!ScopeId::new(0).is_valid());
        assert!(ScopeId::new(1).is_valid());
        assert!(ScopeId::new(u64::MAX).is_valid());
        assert_eq!(ScopeId::new(4).to_string(), "scope:4");
        assert!(ScopeId::new(1) < ScopeId::new(2));
    }

    #[test]
    fn scope_allocator_default_and_saturate() {
        let mut alloc = ScopeIdAllocator::default();
        assert_eq!(alloc.allocate().get(), 0);
        let mut alloc = ScopeIdAllocator::new();
        assert_eq!(alloc.allocate().get(), 2);
        alloc.next = u64::MAX;
        assert_eq!(alloc.allocate().get(), u64::MAX);
        assert_eq!(alloc.allocate().get(), u64::MAX);
    }
}
