//! Integration tests moved from src/lower.rs for region coverage.

use reciplexa_core::lower::*;
use reciplexa_core::ty::*;
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

#[test]
fn lowers_rect_form() {
    let range = TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap();
    let v = lower_surface_form("rect", 4, range).unwrap();
    assert_eq!(v.ty, CoreType::Shape);
}

#[test]
fn lowers_circle_and_text() {
    let range = TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap();
    assert_eq!(
        lower_surface_form("circle", 3, range).unwrap().ty,
        CoreType::Shape
    );
    assert_eq!(
        lower_surface_form("text", 4, range).unwrap().ty,
        CoreType::Shape
    );
}

#[test]
fn lowers_perform_and_src() {
    let range = TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap();
    assert_eq!(
        lower_surface_form("perform", 1, range).unwrap().ty,
        CoreType::Unit
    );
    assert_eq!(
        lower_surface_form("src", 0, range).unwrap().ty,
        CoreType::Unit
    );
}

#[test]
fn rejects_wrong_arg_count() {
    let range = TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap();
    assert!(lower_surface_form("circle", 1, range).is_err());
    assert!(lower_surface_form("perform", 2, range).is_err());
}

#[test]
fn rejects_unknown_form() {
    let range = TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap();
    let err = lower_surface_form("square", 1, range).unwrap_err();
    assert!(err.message.contains("unsupported"));
    assert_eq!(err.range, range);
}
