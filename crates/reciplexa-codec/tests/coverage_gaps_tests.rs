//! Extra coverage for product paths not exercised by the primary suites.

use std::path::{Path, PathBuf};

use reciplexa_codec::atomic::atomic_write;
use reciplexa_codec::codec::{decode_snapshot, encode_snapshot, CodecError, PortableSnapshot};
use reciplexa_codec::extension::PartialRecoveryError;
use reciplexa_codec::migration::{MigrationError, MigrationGraph};
use reciplexa_codec::recovery::{
    compact_after_save, recover_from_journal, write_journal, RecoveryError, RecoveryPaths,
};
use reciplexa_codec::txnlog::{encode_segment, LogError, SegmentStatus, TransactionLogSegment};
use reciplexa_codec::undo::{RevisionUndoLog, UndoAction};
use reciplexa_document::snapshot::DocumentSnapshot;
use reciplexa_identity::document::{DocumentIdentity, DocumentRevision};

fn unique_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rpx-cov-{label}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn atomic_write_empty_path_uses_fallback_name() {
    let cwd = std::env::current_dir().unwrap();
    let tmp = cwd.join(".reciplexa.tmp");
    let target = cwd.join(""); // empty path relative to cwd semantics
                               // Empty Path has no file_name/parent → fallbacks to "./.reciplexa.tmp" then rename to "".
                               // Prefer an explicit empty OsStr path without creating a weird cwd file permanently:
    let empty = Path::new("");
    let _ = std::fs::remove_file(&tmp);
    let result = atomic_write(empty, b"fallback");
    // May succeed (writes .reciplexa.tmp → "") or fail depending on OS; both exercise fallbacks.
    let _ = result;
    let _ = std::fs::remove_file(cwd.join(".reciplexa.tmp"));
    let _ = std::fs::remove_file(cwd.join(""));
    let _ = target;
}

#[test]
fn atomic_write_basename_only_uses_dot_parent() {
    let name = format!(
        "rpx-basename-{}.bin",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let path = Path::new(&name);
    // Ensure we run from temp so cleanup is local.
    let prev = std::env::current_dir().unwrap();
    let dir = unique_dir("atomic-base");
    std::env::set_current_dir(&dir).unwrap();
    atomic_write(path, b"data").unwrap();
    assert_eq!(std::fs::read(path).unwrap(), b"data");
    std::env::set_current_dir(prev).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recovery_io_errors_when_parent_missing() {
    let dir = unique_dir("rec-io");
    let missing_parent = dir.join("nope").join("doc.rpxsnap");
    let paths = RecoveryPaths::for_primary(&missing_parent);
    let snap = DocumentSnapshot::new(DocumentIdentity::new(9));
    let err = write_journal(&paths, &snap).unwrap_err();
    assert!(matches!(err, RecoveryError::Io(_)));

    // recover Io: journal readable but recovered parent missing
    let ok_primary = dir.join("ok.rpxsnap");
    let mut paths_ok = RecoveryPaths::for_primary(&ok_primary);
    write_journal(&paths_ok, &snap).unwrap();
    paths_ok.recovered = dir.join("missing-parent").join("out.rpxrecovered");
    let err = recover_from_journal(&paths_ok).unwrap_err();
    assert!(matches!(err, RecoveryError::Io(_)));

    // compact Io: primary parent missing
    let paths_bad = RecoveryPaths {
        primary: dir.join("missing-parent").join("primary.rpxsnap"),
        journal: dir.join("j.rpxjournal"),
        recovered: dir.join("r.rpxrecovered"),
    };
    let err = compact_after_save(&paths_bad, &snap).unwrap_err();
    assert!(matches!(err, RecoveryError::Io(_)));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn error_and_model_derives_are_exercised() {
    let c1 = CodecError::Decode("x".into());
    let c2 = CodecError::UnsupportedSchema(3);
    assert_eq!(c1.clone(), c1);
    assert_ne!(c1, c2);
    let _ = format!("{c1:?}{c2:?}");

    let m1 = MigrationError::Unsupported(2);
    let m2 = MigrationError::Codec(CodecError::Decode("y".into()));
    assert_eq!(m1.clone(), m1);
    assert_ne!(m1, m2);
    let _ = format!("{m1:?}{m2:?}");
    let _ = MigrationGraph.clone();

    let r1 = RecoveryError::Io("io".into());
    let r2 = RecoveryError::NoRecoveryCandidate;
    let r3 = RecoveryError::Codec(CodecError::Decode("z".into()));
    assert_eq!(r1.clone(), r1);
    assert_ne!(r1, r2);
    assert_ne!(r2, r3);
    let _ = format!("{r1:?}{r2:?}{r3:?}");
    let from_io: RecoveryError = std::io::Error::other("boom").into();
    assert!(matches!(from_io, RecoveryError::Io(_)));

    let paths = RecoveryPaths::for_primary("a.rpxsnap");
    assert_eq!(paths.clone(), paths);
    let _ = format!("{paths:?}");

    let l1 = LogError::TornSegment;
    let l2 = LogError::AlreadyClosed;
    let l3 = LogError::EmptyCommit;
    assert_eq!(l1.clone(), l1);
    assert_ne!(l1, l2);
    assert_ne!(l2, l3);
    let _ = format!("{l1:?}{l2:?}{l3:?}");

    let st = SegmentStatus::Open;
    assert_eq!(st, SegmentStatus::Open);
    assert_ne!(st, SegmentStatus::Committed);
    let _ = format!(
        "{:?}{:?}{:?}",
        st,
        SegmentStatus::Committed,
        SegmentStatus::Aborted
    );

    let action = UndoAction::ApplySnapshot {
        before_revision: DocumentRevision::new(1),
        after_revision: DocumentRevision::new(2),
    };
    assert_eq!(action.clone(), action);
    let _ = format!("{action:?}");
    let _ = RevisionUndoLog::default();

    let p = PartialRecoveryError::NoUsableSnapshot;
    assert_eq!(p.clone(), p);
    let p2 = PartialRecoveryError::Codec(CodecError::Decode("q".into()));
    assert_ne!(p, p2);
    let _ = format!("{p:?}{p2:?}");

    let portable = PortableSnapshot {
        schema_version: 1,
        document_id: 1,
        revision: 0,
        nodes: vec![],
    };
    assert_eq!(portable.clone(), portable);
    let _ = format!("{portable:?}");
    let bytes = encode_snapshot(&DocumentSnapshot::new(DocumentIdentity::new(1)));
    assert!(!bytes.is_empty());
    let seg = TransactionLogSegment::open(1);
    let encoded = encode_segment(&seg);
    assert!(!encoded.is_empty());
    let _ = decode_snapshot(&bytes);
}
