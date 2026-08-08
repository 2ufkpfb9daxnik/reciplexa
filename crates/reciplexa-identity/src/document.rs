//! Document identity and revision.

use core::fmt;

/// Stable identity for an editable document across sessions and views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentIdentity(pub u128);

impl DocumentIdentity {
    pub const INVALID: Self = Self(0);

    pub const fn new(value: u128) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u128 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl fmt::Display for DocumentIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "document:{}", self.0)
    }
}

/// Monotonic revision counter for a document snapshot.
///
/// Revisions are not source line numbers and must not be reused after rollback
/// unless the specification explicitly allows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DocumentRevision(pub u64);

impl DocumentRevision {
    pub const ZERO: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for DocumentRevision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rev:{}", self.0)
    }
}

/// Stable node identity within a document tree.
///
/// Must survive reordering and non-destructive edits (`specification.md` GUI
/// State Identity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StableNodeId(pub u64);

impl StableNodeId {
    pub const INVALID: Self = Self(0);

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

impl fmt::Display for StableNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node:{}", self.0)
    }
}

/// Allocates monotonically increasing [`StableNodeId`] values.
#[derive(Debug, Clone, Default)]
pub struct StableNodeIdAllocator {
    next: u64,
}

impl StableNodeIdAllocator {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn allocate(&mut self) -> StableNodeId {
        let id = StableNodeId::new(self.next);
        self.next = self.next.saturating_add(1);
        id
    }

    pub fn peek_next(&self) -> StableNodeId {
        StableNodeId::new(self.next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_never_returns_invalid() {
        let mut alloc = StableNodeIdAllocator::new();
        let a = alloc.allocate();
        let b = alloc.allocate();
        assert!(a.is_valid());
        assert!(b.is_valid());
        assert_ne!(a, b);
    }

    #[test]
    fn revision_increments() {
        assert_eq!(DocumentRevision::ZERO.next().get(), 1);
    }
}
