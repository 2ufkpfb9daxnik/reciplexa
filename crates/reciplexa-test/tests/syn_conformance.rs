//! Conformance tests for UTF-8 source rules (TEST-SYN-001 family).

use reciplexa_source::resource::{SourceDecodeError, SourceResource, SourceResourceId};
use reciplexa_test::{ConformanceId, SpecSection, TestSubject};

#[test]
fn test_syn_utf8_invalid_bytes_rejected() {
  let subject = TestSubject::new("TEST-SYN-UTF8-001", "specification.md SYN-001 §1.1");
  let _ = subject;
  let err = SourceResource::from_bytes(SourceResourceId::new(1), b"\xff").unwrap_err();
  assert!(matches!(err, SourceDecodeError::InvalidUtf8 { .. }));
}

#[test]
fn test_syn_bom_shebang_conflict() {
  let subject = TestSubject::new("TEST-SYN-BOM-001", "specification.md SYN-001 §1.2");
  let _ = subject;
  let err = SourceResource::from_bytes(
    SourceResourceId::new(1),
    "\u{feff}#!/usr/bin/env rpx".as_bytes(),
  )
  .unwrap_err();
  assert_eq!(err, SourceDecodeError::BomAndShebang);
}

#[test]
fn conformance_id_links_to_spec_section() {
  let id = ConformanceId::new("TEST-SYN-UTF8-001");
  let section = SpecSection::new("SYN-001 §1.1");
  assert!(id.to_string().starts_with("TEST-SYN"));
  assert!(section.to_string().contains("SYN-001"));
}
