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
    fn semantic_id_is_opaque() {
        assert!(SemanticNodeId::new(10).is_valid());
    }
}
