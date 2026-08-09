//! Integration tests moved from src/identity.rs for region coverage.

use reciplexa_syntax::identity::*;
use reciplexa_syntax::parse_source;
use reciplexa_syntax::unparse;

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
    let parse = parse_source("(markup hi)");
    let range = parse.root.text_range();
    let id = map.intern(range);
    let mut map2 = SyntaxIdentityMap::new();
    map2.insert_known(range, id);
    assert_eq!(map2.get(range), Some(id));
}
