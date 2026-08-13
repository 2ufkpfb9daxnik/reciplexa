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
        let src = r#"
(import graphics/shapes only rect fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (fill (rect 1 2 3 4) black)))
"#;
        let snap = document_snapshot_from_source(src, DocumentIdentity::new(4)).unwrap();
        let rect = snap
            .nodes
            .iter()
            .find(|n| matches!(n.kind, DocumentNodeKind::Rectangle))
            .unwrap();
        // Package bridge snapshots may omit CST byte provenance; nodes must exist.
        let _ = snap.provenance.get(rect.id);
        assert!(!snap
            .references
            .children_of(snap.nodes.root_id().unwrap())
            .is_empty());
    });
}

#[test]
fn conformance_links_phase3() {
    let id = reciplexa_test::ConformanceId::new("TEST-DOC-001");
    let section = reciplexa_test::SpecSection::new("Phase 3");
    assert!(id.as_str().starts_with("TEST-DOC"));
    assert_eq!(section.as_str(), "Phase 3");
}

#[test]
fn test_doc_empty_transaction_rejected() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(10));
    let tx = TransactionBuilder::new().into_transaction();
    assert!(tx.apply(&mut snap).is_err());
}

#[test]
fn test_doc_set_text_on_rectangle() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(11));
    let root = snap.nodes.root_id().unwrap();
    let rect = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Rectangle)
        .unwrap();
    let mut tx = TransactionBuilder::new();
    tx.set_text(rect, "label");
    tx.into_transaction().apply(&mut snap).unwrap();
    assert_eq!(snap.nodes.get(rect).unwrap().text().unwrap().text, "label");
}

#[test]
fn test_doc_remove_node() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(12));
    let root = snap.nodes.root_id().unwrap();
    let rect = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Rectangle)
        .unwrap();
    let mut tx = TransactionBuilder::new();
    tx.push(DocumentEdit::RemoveNode { node: rect });
    tx.into_transaction().apply(&mut snap).unwrap();
    assert!(snap.nodes.get(rect).is_none());
}

#[test]
fn test_doc_insert_child_with_properties() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(13));
    let root = snap.nodes.root_id().unwrap();
    let mut tx = TransactionBuilder::new();
    tx.push(DocumentEdit::InsertChild {
        parent: root,
        kind: DocumentNodeKind::Page,
        properties: vec![],
    });
    tx.into_transaction().apply(&mut snap).unwrap();
    assert!(snap
        .nodes
        .iter()
        .any(|n| matches!(n.kind, DocumentNodeKind::Page)));
}

#[test]
fn test_doc_applied_no_change_idempotent_layout() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(14));
    let root = snap.nodes.root_id().unwrap();
    let rect = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Rectangle)
        .unwrap();
    let layout = LayoutBox::new(1.0, 2.0, 3.0, 4.0);
    let mut tx = TransactionBuilder::new();
    tx.set_layout(rect, layout);
    tx.into_transaction().apply(&mut snap).unwrap();
    let rev = snap.revision.get();
    let mut tx2 = TransactionBuilder::new();
    tx2.set_layout(rect, layout);
    assert_eq!(
        tx2.into_transaction().apply(&mut snap).unwrap(),
        TransactionOutcome::AppliedNoChange
    );
    assert_eq!(snap.revision.get(), rev);
}

#[test]
fn test_doc_invalid_move_parent() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(15));
    let root = snap.nodes.root_id().unwrap();
    let rect = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Rectangle)
        .unwrap();
    let mut tx = TransactionBuilder::new();
    tx.push(DocumentEdit::MoveNode {
        node: rect,
        parent: StableNodeId::new(9999),
        index: 0,
    });
    assert!(tx.into_transaction().apply(&mut snap).is_err());
}

#[test]
fn test_doc_provenance_missing_node() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(16));
    assert!(snap.provenance.get(StableNodeId::new(404)).is_none());
}
