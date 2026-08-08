//! Syntax-tree node identity.

use core::fmt;

/// Stable identity for a syntax tree node across incremental re-parses.
///
/// Must not be derived from source line numbers (`roadmap.md` Phase 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxNodeId(pub u64);

impl SyntaxNodeId {
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

impl fmt::Display for SyntaxNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "syntax-node:{}", self.0)
    }
}

/// Allocates monotonically increasing [`SyntaxNodeId`] values for a parse session.
#[derive(Debug, Clone, Default)]
pub struct SyntaxNodeIdAllocator {
    next: u64,
}

impl SyntaxNodeIdAllocator {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn allocate(&mut self) -> SyntaxNodeId {
        let id = SyntaxNodeId::new(self.next);
        self.next = self.next.saturating_add(1);
        id
    }

    pub fn peek_next(&self) -> SyntaxNodeId {
        SyntaxNodeId::new(self.next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_ids_are_unique() {
        let mut alloc = SyntaxNodeIdAllocator::new();
        assert_ne!(alloc.allocate(), alloc.allocate());
    }

    #[test]
    fn syntax_node_id_boundaries_and_display() {
        assert!(!SyntaxNodeId::INVALID.is_valid());
        assert_eq!(SyntaxNodeId::INVALID.get(), 0);
        assert!(!SyntaxNodeId::new(0).is_valid());
        assert!(SyntaxNodeId::new(1).is_valid());
        assert!(SyntaxNodeId::new(u64::MAX).is_valid());
        assert_eq!(SyntaxNodeId::new(3).to_string(), "syntax-node:3");
        assert!(SyntaxNodeId::new(1) < SyntaxNodeId::new(2));
    }

    #[test]
    fn allocator_starts_at_one_and_peeks() {
        let mut alloc = SyntaxNodeIdAllocator::default();
        assert_eq!(alloc.peek_next().get(), 0);
        let mut alloc = SyntaxNodeIdAllocator::new();
        assert_eq!(alloc.peek_next().get(), 1);
        let a = alloc.allocate();
        assert_eq!(a.get(), 1);
        assert_eq!(alloc.peek_next().get(), 2);
        assert_eq!(alloc.allocate().get(), 2);
    }

    #[test]
    fn allocator_saturates_at_u64_max() {
        let mut alloc = SyntaxNodeIdAllocator { next: u64::MAX };
        assert_eq!(alloc.allocate().get(), u64::MAX);
        assert_eq!(alloc.allocate().get(), u64::MAX);
        assert_eq!(alloc.peek_next().get(), u64::MAX);
    }
}
