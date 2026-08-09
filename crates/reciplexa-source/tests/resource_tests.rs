//! Integration tests moved from src/resource.rs for region coverage.

use reciplexa_source::resource::*;

#[test]
fn accepts_valid_utf8() {
    let src = SourceResource::from_utf8(SourceResourceId::new(1), "(page a4)").unwrap();
    assert_eq!(src.text(), "(page a4)");
    assert_eq!(src.id(), SourceResourceId::new(1));
    assert!(!src.has_bom());
}

#[test]
fn rejects_invalid_utf8() {
    let bytes = b"(page \xff)";
    let err = SourceResource::from_bytes(SourceResourceId::new(1), bytes).unwrap_err();
    assert!(matches!(err, SourceDecodeError::InvalidUtf8 { .. }));
}

#[test]
fn bom_at_start_is_recorded() {
    let src = SourceResource::from_utf8(SourceResourceId::new(1), "\u{feff}(doc)").unwrap();
    assert!(src.has_bom());
}

#[test]
fn bom_and_shebang_conflict() {
    let err = SourceResource::from_utf8(SourceResourceId::new(1), "\u{feff}#!/usr/bin/env rpx")
        .unwrap_err();
    assert_eq!(err, SourceDecodeError::BomAndShebang);
}

#[test]
fn shebang_without_bom_is_ok() {
    let src =
        SourceResource::from_utf8(SourceResourceId::new(2), "#!/usr/bin/env rpx\n(page a4)")
            .unwrap();
    assert!(src.has_shebang());
    assert!(!src.has_bom());
}

#[test]
fn resource_id_display_and_validity() {
    let id = SourceResourceId::new(42);
    assert_eq!(id.to_string(), "source-resource:42");
    assert!(id.is_valid());
    assert!(!SourceResourceId::INVALID.is_valid());
    assert_eq!(id.get(), 42);
}

#[test]
fn range_for_validates_offsets() {
    let src = SourceResource::from_utf8(SourceResourceId::new(1), "abcdef").unwrap();
    let range = src.range_for(1, 4).unwrap();
    assert_eq!(range.start().get(), 1);
    assert_eq!(range.end().get(), 4);
}

#[test]
fn from_bytes_roundtrip() {
    let bytes = b"(page a4)";
    let src = SourceResource::from_bytes(SourceResourceId::new(3), bytes).unwrap();
    assert_eq!(src.text(), "(page a4)");
    assert_eq!(src.len_bytes(), 9);
    assert!(!src.is_empty());
}

#[test]
fn decode_error_display() {
    let err = SourceDecodeError::InvalidUtf8 { offset: 7 };
    assert!(err.to_string().contains("7"));
    assert!(SourceDecodeError::BomAndShebang.to_string().contains("BOM"));
}

#[test]
fn range_for_rejects_inverted_offsets() {
    let src = SourceResource::from_utf8(SourceResourceId::new(1), "abcdef").unwrap();
    assert!(src.range_for(4, 2).is_err());
}
