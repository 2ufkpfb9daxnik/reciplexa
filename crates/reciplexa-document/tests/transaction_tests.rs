use reciplexa_document::*;
use reciplexa_identity::document::{DocumentIdentity, StableNodeId};

#[test]
fn move_orphan_node_with_no_old_parent() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let root = snap.nodes.root_id().unwrap();
    let orphan = snap.nodes.allocate(DocumentNodeKind::Rectangle);
    assert!(snap.nodes.get(orphan).unwrap().parent.is_none());
    let mut tx = TransactionBuilder::new();
    tx.push(DocumentEdit::MoveNode {
        node: orphan,
        parent: root,
        index: 0,
    });
    tx.into_transaction().apply(&mut snap).unwrap();
    assert_eq!(snap.nodes.get(orphan).unwrap().parent, Some(root));
}

#[test]
fn set_text_unknown_node_fails() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let mut tx = TransactionBuilder::new();
    tx.set_text(StableNodeId::new(9999), "nope");
    assert!(matches!(
        tx.into_transaction().apply(&mut snap),
        Err(TransactionError::UnknownNode(_))
    ));
}

#[test]
fn move_with_dangling_old_parent_still_links() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let root = snap.nodes.root_id().unwrap();
    let page = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Page)
        .unwrap();
    let rect = snap
        .nodes
        .insert_child(page, DocumentNodeKind::Rectangle)
        .unwrap();
    snap.nodes.drop_node_keep_links(page);
    let mut tx = TransactionBuilder::new();
    tx.push(DocumentEdit::MoveNode {
        node: rect,
        parent: root,
        index: 0,
    });
    tx.into_transaction().apply(&mut snap).unwrap();
    assert_eq!(snap.nodes.get(rect).unwrap().parent, Some(root));
    assert!(snap.nodes.get(root).unwrap().children.contains(&rect));
}

#[test]
fn failed_transaction_rolls_back() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let root = snap.nodes.root_id().unwrap();
    let rect = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Rectangle)
        .unwrap();
    let mut tx = TransactionBuilder::new();
    tx.set_layout(rect, LayoutBox::new(1.0, 2.0, 10.0, 20.0));
    tx.push(DocumentEdit::SetLayout {
        node: StableNodeId::new(9999),
        layout: LayoutBox::new(0.0, 0.0, 1.0, 1.0),
    });
    let batch = tx.into_transaction();
    assert!(batch.apply(&mut snap).is_err());
    assert!(snap.nodes.get(rect).unwrap().layout().is_none());
}

#[test]
fn move_preserves_stable_id() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
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
    tx.into_transaction().apply(&mut snap).unwrap();
    assert!(snap.nodes.get(rect).is_some());
}

#[test]
fn empty_batch_rejected() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let tx = TransactionBuilder::new().into_transaction();
    assert_eq!(tx.apply(&mut snap), Err(TransactionError::EmptyBatch));
}

#[test]
fn applied_no_change_when_layout_identical() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
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
fn remove_unknown_node_fails() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let mut tx = TransactionBuilder::new();
    tx.push(DocumentEdit::RemoveNode {
        node: StableNodeId::new(9999),
    });
    assert!(matches!(
        tx.into_transaction().apply(&mut snap),
        Err(TransactionError::UnknownNode(_))
    ));
}

#[test]
fn insert_child_unknown_parent_fails() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let mut tx = TransactionBuilder::new();
    tx.push(DocumentEdit::InsertChild {
        parent: StableNodeId::new(9999),
        kind: DocumentNodeKind::Rectangle,
        properties: vec![],
    });
    assert!(matches!(
        tx.into_transaction().apply(&mut snap),
        Err(TransactionError::UnknownNode(_))
    ));
}

#[test]
fn builder_is_empty_initially() {
    let b = TransactionBuilder::new();
    assert!(b.is_empty());
    let mut b2 = TransactionBuilder::new();
    b2.set_layout(StableNodeId::new(1), LayoutBox::new(0.0, 0.0, 1.0, 1.0));
    assert!(!b2.is_empty());
}

#[test]
fn set_text_and_remove_node_succeed() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let root = snap.nodes.root_id().unwrap();
    let text = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Text)
        .unwrap();
    let mut tx = TransactionBuilder::new();
    tx.set_text(text, "hello");
    assert_eq!(
        tx.into_transaction().apply(&mut snap).unwrap(),
        TransactionOutcome::Applied
    );
    assert_eq!(
        snap.nodes
            .get(text)
            .unwrap()
            .text()
            .map(|t| t.text.as_str()),
        Some("hello")
    );

    let mut tx = TransactionBuilder::new();
    tx.push(DocumentEdit::RemoveNode { node: text });
    tx.into_transaction().apply(&mut snap).unwrap();
    assert!(snap.nodes.get(text).is_none());
}

#[test]
fn insert_child_and_set_property_roundtrip() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let root = snap.nodes.root_id().unwrap();
    let mut tx = TransactionBuilder::new();
    tx.push(DocumentEdit::InsertChild {
        parent: root,
        kind: DocumentNodeKind::Rectangle,
        properties: vec![],
    });
    let outcome = tx.into_transaction().apply(&mut snap).unwrap();
    assert!(matches!(
        outcome,
        TransactionOutcome::Applied | TransactionOutcome::AppliedNoChange
    ));
    assert!(snap.nodes.root_id().is_some());
}

#[test]
fn move_unknown_node_fails() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let root = snap.nodes.root_id().unwrap();
    let mut tx = TransactionBuilder::new();
    tx.push(DocumentEdit::MoveNode {
        node: StableNodeId::new(9999),
        parent: root,
        index: 0,
    });
    assert!(tx.into_transaction().apply(&mut snap).is_err());
}

#[test]
fn transaction_error_debug_covers_variants() {
    let _ = format!("{:?}", TransactionError::EmptyBatch);
    let _ = format!("{:?}", TransactionError::UnknownNode(StableNodeId::new(1)));
    let _ = format!("{:?}", TransactionOutcome::Applied);
    let _ = format!("{:?}", TransactionOutcome::AppliedNoChange);
}

#[test]
fn move_to_unknown_parent_is_invalid_parent() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
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
    let err = tx.into_transaction().apply(&mut snap).unwrap_err();
    assert!(matches!(err, TransactionError::InvalidParent(_)));
}

#[test]
fn move_unlinks_from_old_parent() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let root = snap.nodes.root_id().unwrap();
    let page_a = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Page)
        .unwrap();
    let page_b = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Page)
        .unwrap();
    let rect = snap
        .nodes
        .insert_child(page_a, DocumentNodeKind::Rectangle)
        .unwrap();
    let mut tx = TransactionBuilder::new();
    tx.push(DocumentEdit::MoveNode {
        node: rect,
        parent: page_b,
        index: 0,
    });
    tx.into_transaction().apply(&mut snap).unwrap();
    assert!(!snap.nodes.get(page_a).unwrap().children.contains(&rect));
    assert!(snap.nodes.get(page_b).unwrap().children.contains(&rect));
    assert_eq!(snap.nodes.get(rect).unwrap().parent, Some(page_b));
}
