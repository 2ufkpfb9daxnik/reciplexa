//! Phase 9 conformance: persistence, undo, migration, recovery.

use reciplexa_codec::{
    atomic_write, compact_after_save, decode_envelope, decode_segment, decode_snapshot,
    encode_segment, encode_snapshot, migrate_snapshot, partial_recover, recover_from_journal,
    write_journal, CodecError, LogError, MigrationError, PartialRecoveryError, RecoveryError,
    RecoveryPaths, ResourceEntry, ResourceManifest, RevisionUndoLog, TransactionLogSegment,
    SCHEMA_VERSION,
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
    let bytes = encode_snapshot(&snap);
    let decoded = decode_snapshot(&bytes).unwrap();
    assert_eq!(decoded.nodes.get(rect).unwrap().id, rect);
}

#[test]
fn phase9_atomic_write_and_read() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let bytes = encode_snapshot(&snap);
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
    let v1 = encode_snapshot(&DocumentSnapshot::new(DocumentIdentity::new(1)));
    log.push_undo(v1);
    let v2 = encode_snapshot(&DocumentSnapshot::new(DocumentIdentity::new(2)));
    let restored = log.undo(v2).unwrap();
    let snap = decode_snapshot(&restored).unwrap();
    assert_eq!(snap.identity.get(), 1);
}

#[test]
fn phase9_migration_preserves_schema() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(7));
    let bytes = encode_snapshot(&snap);
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
    assert!(e.path.is_some());
}

#[test]
fn phase9_partial_recovery_keeps_snapshot() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let bytes = encode_snapshot(&snap);
    let env = partial_recover(&bytes).unwrap();
    assert_eq!(env.snapshot.document_id, 1);
}

#[test]
fn phase9_unsupported_schema_rejected() {
    let json = r#"{"schema_version":999,"document_id":1,"revision":0,"nodes":[]}"#;
    let err = decode_snapshot(json.as_bytes()).unwrap_err();
    assert!(matches!(err, CodecError::UnsupportedSchema(999)));
}

#[test]
fn phase9_garbage_decode_fails() {
    let err = decode_snapshot(b"{{{not json").unwrap_err();
    assert!(matches!(err, CodecError::Decode(_)));
}

#[test]
fn phase9_nested_parent_child_encoding() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let root = snap.nodes.root_id().unwrap();
    let group = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Group)
        .unwrap();
    let rect = snap
        .nodes
        .insert_child(group, DocumentNodeKind::Rectangle)
        .unwrap();
    let decoded = decode_snapshot(&encode_snapshot(&snap)).unwrap();
    assert_eq!(decoded.nodes.get(rect).unwrap().parent, Some(group));
}

#[test]
fn phase9_envelope_with_extensions() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(4));
    let bytes = encode_snapshot(&snap);
    let bare = decode_envelope(&bytes).unwrap();
    assert_eq!(bare.snapshot.document_id, 4);
}

#[test]
fn phase9_partial_recovery_drops_unknown_extensions() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let base = encode_snapshot(&snap);
    let bare = partial_recover(&base).unwrap();
    let env_json = format!(
        r#"{{"snapshot":{},"extensions":[{{"name":"vendor.x","version":50,"payload":null}},{{"name":"rpx.safe","version":50,"payload":{{}}}}]}}"#,
        String::from_utf8_lossy(&base)
    );
    let recovered = partial_recover(env_json.as_bytes()).unwrap();
    assert_eq!(recovered.extensions.len(), 1);
    assert_eq!(recovered.extensions[0].name, "rpx.safe");
    assert_eq!(recovered.snapshot.document_id, bare.snapshot.document_id);
}

#[test]
fn phase9_partial_recovery_unusable_bytes() {
    let err = partial_recover(b"bad").unwrap_err();
    assert!(matches!(err, PartialRecoveryError::NoUsableSnapshot));
}

#[test]
fn phase9_txnlog_already_closed_and_empty_commit() {
    let mut seg = TransactionLogSegment::open(1);
    assert!(matches!(seg.commit().unwrap_err(), LogError::EmptyCommit));
    seg.append(b"a".to_vec()).unwrap();
    seg.commit().unwrap();
    assert!(matches!(
        seg.append(b"b".to_vec()).unwrap_err(),
        LogError::AlreadyClosed
    ));
}

#[test]
fn phase9_txnlog_abort_is_torn() {
    let mut seg = TransactionLogSegment::open(2);
    seg.append(b"x".to_vec()).unwrap();
    seg.abort();
    assert!(matches!(
        seg.applyable_records().unwrap_err(),
        LogError::TornSegment
    ));
}

#[test]
fn phase9_txnlog_encode_decode_roundtrip() {
    let mut seg = TransactionLogSegment::open(3);
    seg.append(b"rec".to_vec()).unwrap();
    seg.commit().unwrap();
    let bytes = encode_segment(&seg);
    let decoded = decode_segment(&bytes).unwrap();
    assert_eq!(decoded.records.len(), 1);
}

#[test]
fn phase9_undo_redo_and_depth() {
    let mut log = RevisionUndoLog::new(2);
    log.push_undo(b"a".to_vec());
    log.push_undo(b"b".to_vec());
    log.push_undo(b"c".to_vec());
    assert_eq!(log.undo(b"cur".to_vec()).unwrap(), b"c");
    assert!(log.redo(b"c".to_vec()).is_some());
    log.push_undo(b"d".to_vec());
    assert!(!log.can_redo());
}

#[test]
fn phase9_undo_empty_returns_none() {
    let mut log = RevisionUndoLog::new(5);
    assert!(log.undo(b"x".to_vec()).is_none());
}

#[test]
fn phase9_migration_corrupt_bytes() {
    let err = migrate_snapshot(b"corrupt").unwrap_err();
    assert!(matches!(err, MigrationError::Codec(CodecError::Decode(_))));
}

#[test]
fn phase9_recovery_no_journal() {
    let dir = std::env::temp_dir().join(format!(
        "rpx-p9-none-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let paths = RecoveryPaths::for_primary(dir.join("nope.rpxsnap"));
    let err = recover_from_journal(&paths).unwrap_err();
    assert!(matches!(err, RecoveryError::NoRecoveryCandidate));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn phase9_compact_removes_sidecars() {
    let dir = std::env::temp_dir().join(format!(
        "rpx-p9-compact-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let paths = RecoveryPaths::for_primary(dir.join("doc.rpxsnap"));
    let snap = DocumentSnapshot::new(DocumentIdentity::new(3));
    write_journal(&paths, &snap).unwrap();
    recover_from_journal(&paths).unwrap();
    compact_after_save(&paths, &snap).unwrap();
    assert!(paths.primary.exists());
    assert!(!paths.journal.exists());
    assert!(!paths.recovered.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn phase9_manifest_upsert_and_sort() {
    let mut m = ResourceManifest::default();
    m.insert(ResourceEntry {
        id: "z".into(),
        kind: "a".into(),
        path: None,
        content_sha256: None,
    });
    m.insert(ResourceEntry {
        id: "a".into(),
        kind: "b".into(),
        path: None,
        content_sha256: None,
    });
    assert!(m.get("missing").is_none());
    m.insert(ResourceEntry {
        id: "a".into(),
        kind: "updated".into(),
        path: Some("p".into()),
        content_sha256: None,
    });
    assert_eq!(m.get("a").unwrap().kind, "updated");
    assert_eq!(m.entries[0].id, "a");
}

#[test]
fn phase9_atomic_write_overwrite() {
    let path = std::env::temp_dir().join(format!(
        "rpx-p9-ow-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    atomic_write(&path, b"v1").unwrap();
    atomic_write(&path, b"v2").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"v2");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn phase9_parse_unknown_kind_as_document() {
    let json = r#"{"schema_version":1,"document_id":1,"revision":0,"nodes":[{"id":1,"kind":"Weird","parent":null,"children":[]}]}"#;
    let snap = decode_snapshot(json.as_bytes()).unwrap();
    let node = snap
        .nodes
        .get(reciplexa_identity::document::StableNodeId::new(1))
        .unwrap();
    assert_eq!(node.kind, DocumentNodeKind::Document);
}
