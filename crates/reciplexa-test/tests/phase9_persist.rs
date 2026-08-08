//! Phase 9 conformance: persistence, undo, migration, recovery.

use reciplexa_codec::{
    atomic_write, compact_after_save, decode_snapshot, encode_snapshot, migrate_snapshot,
    partial_recover, recover_from_journal, write_journal, RevisionUndoLog, TransactionLogSegment,
    RecoveryPaths, ResourceEntry, ResourceManifest, SCHEMA_VERSION,
};
use reciplexa_document::node::DocumentNodeKind;
use reciplexa_document::snapshot::DocumentSnapshot;
use reciplexa_identity::document::DocumentIdentity;

#[test]
fn phase9_stable_id_roundtrip() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(100));
    let root = snap.nodes.root_id().unwrap();
    let rect = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Rectangle)
        .unwrap();
    let bytes = encode_snapshot(&snap).unwrap();
    let decoded = decode_snapshot(&bytes).unwrap();
    assert_eq!(decoded.nodes.get(rect).unwrap().id, rect);
}

#[test]
fn phase9_atomic_write_and_read() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let bytes = encode_snapshot(&snap).unwrap();
    let path = std::env::temp_dir().join(format!(
        "rpx-phase9-{}.rpxsnap",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    atomic_write(&path, &bytes).unwrap();
    let read = std::fs::read(&path).unwrap();
    assert_eq!(read, bytes);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn phase9_undo_redo_revision_log() {
    let mut log = RevisionUndoLog::new(50);
    let v1 = encode_snapshot(&DocumentSnapshot::new(DocumentIdentity::new(1))).unwrap();
    log.push_undo(v1);
    let v2 = encode_snapshot(&DocumentSnapshot::new(DocumentIdentity::new(2))).unwrap();
    let restored = log.undo(v2).unwrap();
    let snap = decode_snapshot(&restored).unwrap();
    assert_eq!(snap.identity.get(), 1);
}

#[test]
fn phase9_migration_preserves_schema() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(7));
    let bytes = encode_snapshot(&snap).unwrap();
    let migrated = migrate_snapshot(&bytes).unwrap();
    assert_eq!(migrated.identity.get(), 7);
    assert_eq!(SCHEMA_VERSION, 1);
}

#[test]
fn phase9_torn_transaction_not_applied() {
    let mut seg = TransactionLogSegment::open(1);
    seg.append(b"partial".to_vec()).unwrap();
    assert!(seg.applyable_records().is_err());
    seg.commit().unwrap();
    assert_eq!(seg.applyable_records().unwrap().len(), 1);
}

#[test]
fn phase9_recovery_does_not_overwrite_primary() {
    let dir = std::env::temp_dir().join(format!(
        "rpx-p9-rec-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let primary = dir.join("doc.rpxsnap");
    let paths = RecoveryPaths::for_primary(&primary);
    let snap = DocumentSnapshot::new(DocumentIdentity::new(9));
    write_journal(&paths, &snap).unwrap();
    recover_from_journal(&paths).unwrap();
    assert!(!paths.primary.exists());
    assert!(paths.recovered.exists());
    compact_after_save(&paths, &snap).unwrap();
    assert!(paths.primary.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn phase9_resource_manifest_no_secrets() {
    let mut m = ResourceManifest::default();
    m.insert(ResourceEntry {
        id: "img1".into(),
        kind: "image".into(),
        path: Some("a.png".into()),
        content_sha256: Some("abc".into()),
    });
    let e = m.get("img1").unwrap();
    assert_eq!(e.kind, "image");
    // Manifest API has no fields for secrets / native pointers / capabilities.
    assert!(e.path.is_some());
}

#[test]
fn phase9_partial_recovery_keeps_snapshot() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let bytes = encode_snapshot(&snap).unwrap();
    let env = partial_recover(&bytes).unwrap();
    assert_eq!(env.snapshot.document_id, 1);
}
