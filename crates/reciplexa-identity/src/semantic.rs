//! Semantic graph identities distinct from document tree nodes.

use core::fmt;

/// Identity for semantic nodes (accessibility, outline, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticNodeId(pub u64);

impl SemanticNodeId {
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

/// Allocates semantic node ids.
#[derive(Debug, Clone, Default)]
pub struct SemanticNodeIdAllocator {
    next: u64,
}

impl SemanticNodeIdAllocator {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn allocate(&mut self) -> SemanticNodeId {
        let id = SemanticNodeId::new(self.next);
        self.next = self.next.saturating_add(1);
        id
    }
}

impl fmt::Display for SemanticNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "semantic:{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_allocator_issues_unique_ids() {
        let mut alloc = SemanticNodeIdAllocator::new();
        assert_ne!(alloc.allocate(), alloc.allocate());
    }

    #[test]
    fn semantic_id_is_opaque() {
        assert!(SemanticNodeId::new(10).is_valid());
    }

    #[test]
    fn semantic_id_boundaries_and_display() {
        assert!(!SemanticNodeId::INVALID.is_valid());
        assert_eq!(SemanticNodeId::INVALID.get(), 0);
        assert!(!SemanticNodeId::new(0).is_valid());
        assert!(SemanticNodeId::new(1).is_valid());
        assert!(SemanticNodeId::new(u64::MAX).is_valid());
        assert_eq!(SemanticNodeId::new(12).to_string(), "semantic:12");
        assert!(SemanticNodeId::new(1) < SemanticNodeId::new(2));
    }

    #[test]
    fn semantic_allocator_default_and_saturate() {
        let mut alloc = SemanticNodeIdAllocator::default();
        assert_eq!(alloc.allocate().get(), 0);
        let mut alloc = SemanticNodeIdAllocator::new();
        assert_eq!(alloc.allocate().get(), 1);
        alloc.next = u64::MAX;
        assert_eq!(alloc.allocate().get(), u64::MAX);
        assert_eq!(alloc.allocate().get(), u64::MAX);
    }
}
