//! Phase 4 vertical slice smoke tests.

use reciplexa::document_pipeline::document_snapshot_from_source;
use reciplexa_identity::document::DocumentIdentity;
use reciplexa_test::ConformanceId;

#[test]
fn test_slice_source_to_document() {
    let _ = ConformanceId::new("TEST-SLICE-001");
    let snap = document_snapshot_from_source(
        "(page a4 (rect 10 20 30 40))",
        DocumentIdentity::new(1),
    )
    .expect("snapshot");
    assert!(snap.revision.get() == 0);
    assert!(snap.nodes.iter().count() >= 3);
}

#[test]
fn test_slice_text_source() {
    let snap = document_snapshot_from_source(
        r#"(page a4 (text 10 20 12 "Hello"))"#,
        DocumentIdentity::new(2),
    )
    .expect("snapshot");
    let has_text = snap.nodes.iter().any(|n| n.text().is_some());
    assert!(has_text);
}
