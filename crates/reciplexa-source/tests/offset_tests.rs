//! Integration tests moved from src/offset.rs for region coverage.

use reciplexa_source::offset::*;

#[test]
fn saturating_sub_does_not_underflow() {
    assert_eq!(ByteOffset::new(3).saturating_sub(10), ByteOffset::ZERO);
}

#[test]
fn zero_is_default() {
    assert_eq!(ByteOffset::default(), ByteOffset::ZERO);
}

#[test]
fn checked_add_overflows() {
    assert!(ByteOffset::new(u32::MAX).checked_add(1).is_none());
}

#[test]
fn converts_to_usize() {
    assert_eq!(usize::from(ByteOffset::new(42)), 42);
}

#[test]
fn saturating_add_caps_at_max() {
    assert_eq!(
        ByteOffset::new(u32::MAX - 1).saturating_add(5),
        ByteOffset::new(u32::MAX)
    );
}

#[test]
fn from_u32_roundtrip() {
    let off: ByteOffset = 99u32.into();
    assert_eq!(u32::from(off), 99);
}
