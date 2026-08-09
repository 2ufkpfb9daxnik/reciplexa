//! Syntax node identity side table (Phase 1 §3.2 item 5).

use std::collections::HashMap;

use rowan::TextRange;

use reciplexa_identity::syntax::{SyntaxNodeId, SyntaxNodeIdAllocator};

use crate::kind::SyntaxNode;
use crate::unparse;

/// Maps rowan text ranges to stable syntax node ids for a parse session.
#[derive(Debug, Clone, Default)]
pub struct SyntaxIdentityMap {
    alloc: SyntaxNodeIdAllocator,
    by_range: HashMap<(u32, u32), SyntaxNodeId>,
    by_id: HashMap<SyntaxNodeId, (u32, u32)>,
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
        self.by_id.insert(id, key);
        id
    }

    pub fn insert_known(&mut self, range: TextRange, id: SyntaxNodeId) {
        let key = (u32::from(range.start()), u32::from(range.end()));
        self.by_range.insert(key, id);
        self.by_id.insert(id, key);
    }

    pub fn get(&self, range: TextRange) -> Option<SyntaxNodeId> {
        let key = (u32::from(range.start()), u32::from(range.end()));
        self.by_range.get(&key).copied()
    }

    pub fn get_byte_offsets(&self, start: u32, end: u32) -> Option<SyntaxNodeId> {
        self.by_range.get(&(start, end)).copied()
    }

    pub fn get_by_id(&self, id: SyntaxNodeId) -> Option<TextRange> {
        self.by_id
            .get(&id)
            .map(|(s, e)| TextRange::new((*s).into(), (*e).into()))
    }

    pub fn len(&self) -> usize {
        self.by_range.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_range.is_empty()
    }
}

/// Assign stable ids to every syntax subtree in `root`.
pub fn build_identity_map(root: &SyntaxNode) -> SyntaxIdentityMap {
    let mut map = SyntaxIdentityMap::new();
    walk_intern(root, &mut map);
    map
}

fn walk_intern(node: &SyntaxNode, map: &mut SyntaxIdentityMap) {
    map.intern(node.text_range());
    for child in node.children() {
        walk_intern(&child, map);
    }
}

/// Re-parse identity preservation: reuse ids for subtrees whose structural text is unchanged.
pub fn preserve_identity_on_reparse(
    prev: &SyntaxIdentityMap,
    old_root: &SyntaxNode,
    new_root: &SyntaxNode,
) -> SyntaxIdentityMap {
    let mut old_by_sig: HashMap<String, SyntaxNodeId> = HashMap::new();
    collect_structural_ids(old_root, prev, &mut old_by_sig);

    let mut next = SyntaxIdentityMap::new();
    assign_preserved(new_root, &old_by_sig, &mut next);
    next
}

fn structural_signature(node: &SyntaxNode) -> String {
    unparse(node)
}

fn collect_structural_ids(
    node: &SyntaxNode,
    prev: &SyntaxIdentityMap,
    out: &mut HashMap<String, SyntaxNodeId>,
) {
    let sig = structural_signature(node);
    // `prev` was built from the same old tree we are walking.
    let id = prev
        .get(node.text_range())
        .expect("old tree ranges are present in the previous identity map");
    out.entry(sig).or_insert(id);
    for child in node.children() {
        collect_structural_ids(&child, prev, out);
    }
}

fn assign_preserved(
    node: &SyntaxNode,
    old_by_sig: &HashMap<String, SyntaxNodeId>,
    map: &mut SyntaxIdentityMap,
) {
    let sig = structural_signature(node);
    if let Some(&id) = old_by_sig.get(&sig) {
        map.insert_known(node.text_range(), id);
    } else {
        map.intern(node.text_range());
    }
    for child in node.children() {
        assign_preserved(&child, old_by_sig, map);
    }
}
