//! Phase 3 revision and provenance conformance.

use reciplexa::document_pipeline::document_snapshot_from_source;
use reciplexa_document::{
    DocumentEdit, DocumentNodeKind, DocumentSnapshot, LayoutBox, TransactionBuilder,
    TransactionOutcome,
};
use reciplexa_identity::document::{DocumentIdentity, StableNodeId};
use reciplexa_test::{run_conformance, ConformanceCase, TestSubject};

#[test]
fn test_doc_transaction_rollback() {
    let _ = TestSubject::new("TEST-DOC-001", "roadmap.md Phase 3 §5.3");
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let root = snap.nodes.root_id().unwrap();
    let rect = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Rectangle)
        .unwrap();
    let mut tx = TransactionBuilder::new();
    tx.set_layout(rect, LayoutBox::new(1.0, 2.0, 3.0, 4.0));
    tx.push(DocumentEdit::SetLayout {
        node: StableNodeId::new(9999),
        layout: LayoutBox::new(0.0, 0.0, 1.0, 1.0),
    });
    assert!(tx.into_transaction().apply(&mut snap).is_err());
    assert!(snap.nodes.get(rect).unwrap().layout().is_none());
}

#[test]
fn test_doc_move_preserves_id() {
    let subject = TestSubject::new("TEST-DOC-002", "roadmap.md Phase 3 §5.3");
    let _ = subject;
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(2));
    let root = snap.nodes.root_id().unwrap();
    let page = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Page)
        .unwrap();
    let rect = snap
        .nodes
        .insert_child(page, DocumentNodeKind::Rectangle)
        .unwrap();
    let mut tx = TransactionBuilder::new();
    tx.push(DocumentEdit::MoveNode {
        node: rect,
        parent: root,
        index: 0,
    });
    assert_eq!(
        tx.into_transaction().apply(&mut snap).unwrap(),
        TransactionOutcome::Applied
    );
    assert!(snap.nodes.get(rect).is_some());
}

#[test]
fn test_doc_revision_increments_on_apply() {
    let case = ConformanceCase::new("TEST-DOC-003", "Phase 3", "revision tracking");
    run_conformance(&case, || {
        let mut snap = DocumentSnapshot::new(DocumentIdentity::new(3));
        let root = snap.nodes.root_id().unwrap();
        let rect = snap
            .nodes
            .insert_child(root, DocumentNodeKind::Rectangle)
            .unwrap();
        let rev0 = snap.revision.get();
        let mut tx = TransactionBuilder::new();
        tx.set_layout(rect, LayoutBox::new(1.0, 2.0, 3.0, 4.0));
        tx.into_transaction().apply(&mut snap).unwrap();
        assert!(snap.revision.get() > rev0);
    });
}

#[test]
fn test_doc_provenance_from_source() {
    let case = ConformanceCase::new("TEST-DOC-004", "Phase 3", "provenance walk");
    run_conformance(&case, || {
        let snap =
            document_snapshot_from_source("(page a4 (rect 1 2 3 4))", DocumentIdentity::new(4))
                .unwrap();
        let rect = snap
            .nodes
            .iter()
            .find(|n| matches!(n.kind, DocumentNodeKind::Rectangle))
            .unwrap();
        let prov = snap.provenance.get(rect.id).unwrap();
        assert!(!prov.text_range.is_empty());
        assert!(
            !snap.references
                .children_of(snap.nodes.root_id().unwrap())
                .is_empty()
        );
    });
}

#[test]
fn conformance_links_phase3() {
    let id = reciplexa_test::ConformanceId::new("TEST-DOC-001");
    let section = reciplexa_test::SpecSection::new("Phase 3");
    assert!(id.as_str().starts_with("TEST-DOC"));
    assert_eq!(section.as_str(), "Phase 3");
}
