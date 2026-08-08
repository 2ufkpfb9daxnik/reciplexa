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
    if let Some(id) = prev.get(node.text_range()) {
        out.entry(sig).or_insert(id);
    }
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

    #[test]
    fn build_identity_map_covers_subtrees() {
        let parse = parse_source("(page a4 (circle 1 2 3) (text 1 2 12 \"hi\"))");
        assert!(parse.errors.is_empty(), "{:?}", parse.errors);
        let map = build_identity_map(&parse.root);
        assert!(
            map.len() >= 3,
            "expected multiple syntax subtrees, got {}",
            map.len()
        );
    }

    #[test]
    fn small_edit_preserves_unchanged_subtree_identity() {
        let src1 = "(page a4 (rect 10 20 30 40))";
        let src2 = "(page a4 (rect 10 20 50 40))";
        let p1 = parse_source(src1);
        let p2 = parse_source(src2);
        let map1 = build_identity_map(&p1.root);
        let map2 = preserve_identity_on_reparse(&map1, &p1.root, &p2.root);

        let page1 = p1.root.children().next().unwrap();
        let page2 = p2.root.children().next().unwrap();
        assert_eq!(
            map1.get(page1.text_range()),
            map2.get(page2.text_range()),
            "page subtree unchanged — id should be preserved"
        );
    }

    #[test]
    fn format_roundtrip_preserves_identity() {
        let src = "(page a4 (circle 1 2 3))";
        let p1 = parse_source(src);
        let formatted = unparse(&p1.root);
        let p2 = parse_source(&formatted);
        let map1 = build_identity_map(&p1.root);
        let map2 = preserve_identity_on_reparse(&map1, &p1.root, &p2.root);
        assert_eq!(map1.len(), map2.len());
    }

    #[test]
    fn identity_map_get_by_id_and_is_empty() {
        let mut map = SyntaxIdentityMap::new();
        assert!(map.is_empty());
        let parse = parse_source("(page a4)");
        let range = parse.root.text_range();
        let id = map.intern(range);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_by_id(id), Some(range));
        assert_eq!(
            map.get_byte_offsets(u32::from(range.start()), u32::from(range.end())),
            Some(id)
        );
    }

    #[test]
    fn insert_known_roundtrip() {
        let mut map = SyntaxIdentityMap::new();
        let parse = parse_source("(doc hi)");
        let range = parse.root.text_range();
        let id = map.intern(range);
        let mut map2 = SyntaxIdentityMap::new();
        map2.insert_known(range, id);
        assert_eq!(map2.get(range), Some(id));
    }
}
