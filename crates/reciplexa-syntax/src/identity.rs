//! Syntax node identity side table (Phase 1 §3.2 item 5).

use std::collections::HashMap;

use rowan::TextRange;

use reciplexa_identity::syntax::{SyntaxNodeId, SyntaxNodeIdAllocator};

/// Maps rowan text ranges to stable syntax node ids for a parse session.
#[derive(Debug, Clone, Default)]
pub struct SyntaxIdentityMap {
    alloc: SyntaxNodeIdAllocator,
    by_range: HashMap<(u32, u32), SyntaxNodeId>,
}

impl SyntaxIdentityMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, range: TextRange) -> SyntaxNodeId {
        let key = (u32::from(range.start()), u32::from(range.end()));
        if let Some(id) = self.by_range.get(&key) {
            return *id;
        }
        let id = self.alloc.allocate();
        self.by_range.insert(key, id);
        id
    }

    pub fn get(&self, range: TextRange) -> Option<SyntaxNodeId> {
        let key = (u32::from(range.start()), u32::from(range.end()));
        self.by_range.get(&key).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_source;

    #[test]
    fn same_range_reuses_id() {
        let parse = parse_source("(page a4)");
        let range = parse.root.text_range();
        let mut map = SyntaxIdentityMap::new();
        let a = map.intern(range);
        let b = map.intern(range);
        assert_eq!(a, b);
    }
}
