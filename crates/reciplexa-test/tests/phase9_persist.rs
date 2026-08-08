//! Phase 9 conformance: persistence, undo, migration.

use reciplexa_codec::{
    atomic_write, decode_snapshot, encode_snapshot, migrate_snapshot, RevisionUndoLog, SCHEMA_VERSION,
};
use reciplexa_document::node::DocumentNodeKind;
use reciplexa_document::snapshot::DocumentSnapshot;
use reciplexa_identity::document::DocumentIdentity;

#[test]
fn phase9_stable_id_roundtrip() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(100));
    let root = snap.nodes.root_id().unwrap();
    let rect = snap.nodes.insert_child(root, DocumentNodeKind::Rectangle).unwrap();
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
