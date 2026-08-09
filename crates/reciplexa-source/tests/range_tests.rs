//! Integration tests moved from src/range.rs for region coverage.

use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::*;

fn off(n: u32) -> ByteOffset {
    ByteOffset::new(n)
}

#[test]
fn empty_range_has_zero_len() {
    assert!(TextRange::EMPTY.is_empty());
    assert_eq!(TextRange::EMPTY.len(), 0);
}

#[test]
fn const_new_builds_unchecked_range() {
    let r = TextRange::new(off(2), off(5));
    assert_eq!(r.start(), off(2));
    assert_eq!(r.end(), off(5));
    assert_eq!(r.len(), 3);
}

#[test]
fn rejects_end_before_start() {
    assert!(matches!(
        TextRange::try_new(off(5), off(3)),
        Err(TextRangeError::EndBeforeStart { .. })
    ));
}

#[test]
fn contains_offset_is_half_open() {
    let r = TextRange::try_new(off(2), off(5)).unwrap();
    assert!(!r.contains_offset(off(1)));
    assert!(r.contains_offset(off(2)));
    assert!(r.contains_offset(off(4)));
    assert!(!r.contains_offset(off(5)));
}

#[test]
fn union_and_intersect() {
    let a = TextRange::try_new(off(0), off(3)).unwrap();
    let b = TextRange::try_new(off(2), off(6)).unwrap();
    assert_eq!(
        a.union(b).unwrap(),
        TextRange::try_new(off(0), off(6)).unwrap()
    );
    assert_eq!(
        a.intersect(b).unwrap(),
        TextRange::try_new(off(2), off(3)).unwrap()
    );
}

#[test]
fn disjoint_intersect_is_none() {
    let a = TextRange::try_new(off(0), off(2)).unwrap();
    let b = TextRange::try_new(off(3), off(5)).unwrap();
    assert!(a.intersect(b).is_none());
}

#[test]
fn contains_range_and_cover() {
    let outer = TextRange::try_new(off(0), off(10)).unwrap();
    let inner = TextRange::try_new(off(2), off(5)).unwrap();
    assert!(outer.contains_range(inner));
    assert!(!inner.contains_range(outer));
    let covered = outer.cover(off(12)).unwrap();
    assert_eq!(covered, TextRange::try_new(off(0), off(12)).unwrap());
}

#[test]
fn at_offset_is_zero_width() {
    let r = TextRange::at(off(7));
    assert!(r.is_empty());
    assert!(!r.contains_offset(off(7)));
    assert!(!r.contains_offset(off(8)));
}

#[test]
fn display_formats_half_open_range() {
    let r = TextRange::try_new(off(1), off(4)).unwrap();
    assert_eq!(r.to_string(), "[1, 4)");
}
