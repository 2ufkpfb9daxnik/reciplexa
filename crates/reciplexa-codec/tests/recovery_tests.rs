use reciplexa_codec::recovery::*;
use reciplexa_document::snapshot::DocumentSnapshot;
use reciplexa_identity::document::DocumentIdentity;

#[test]
fn recovery_writes_sidecar_not_primary() {
    let dir = std::env::temp_dir().join(format!(
        "rpx-rec-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let primary = dir.join("doc.rpxsnap");
    let paths = RecoveryPaths::for_primary(&primary);
    let snap = DocumentSnapshot::new(DocumentIdentity::new(3));
    // Primary does not exist yet; journal does.
    write_journal(&paths, &snap).unwrap();
    assert!(!paths.primary.exists());
    let recovered = recover_from_journal(&paths).unwrap();
    assert_eq!(recovered.identity.get(), 3);
    assert!(paths.recovered.exists());
    assert!(!paths.primary.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recover_no_journal_returns_no_candidate() {
    let dir = std::env::temp_dir().join(format!(
        "rpx-rec-none-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let paths = RecoveryPaths::for_primary(dir.join("missing.rpxsnap"));
    let err = recover_from_journal(&paths).unwrap_err();
    assert!(matches!(err, RecoveryError::NoRecoveryCandidate));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn compact_drops_sidecar_files() {
    let dir = std::env::temp_dir().join(format!(
        "rpx-rec-compact-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let primary = dir.join("doc.rpxsnap");
    let paths = RecoveryPaths::for_primary(&primary);
    let snap = DocumentSnapshot::new(DocumentIdentity::new(5));
    write_journal(&paths, &snap).unwrap();
    recover_from_journal(&paths).unwrap();
    assert!(paths.journal.exists());
    assert!(paths.recovered.exists());
    compact_after_save(&paths, &snap).unwrap();
    assert!(paths.primary.exists());
    assert!(!paths.journal.exists());
    assert!(!paths.recovered.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recover_corrupt_journal_returns_codec_error() {
    let dir = std::env::temp_dir().join(format!(
        "rpx-rec-bad-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let paths = RecoveryPaths::for_primary(dir.join("doc.rpxsnap"));
    std::fs::write(&paths.journal, b"not a snapshot").unwrap();
    let err = recover_from_journal(&paths).unwrap_err();
    assert!(matches!(
        err,
        RecoveryError::Sidecar(_) | RecoveryError::Codec(_)
    ));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recovery_paths_for_primary_sets_sidecars() {
    let primary = std::path::PathBuf::from("docs/snap.rpxsnap");
    let paths = RecoveryPaths::for_primary(&primary);
    assert_eq!(paths.primary, primary);
    assert_eq!(
        paths.journal,
        std::path::PathBuf::from("docs/snap.rpxjournal")
    );
    assert_eq!(
        paths.recovered,
        std::path::PathBuf::from("docs/snap.rpxrecovered")
    );
}

#[test]
fn source_journal_path_for_rpx() {
    let rpx = std::path::PathBuf::from("docs/article.rpx");
    assert_eq!(
        source_journal_path(&rpx),
        std::path::PathBuf::from("docs/article.rpjsrc")
    );
    let snap = RecoveryPaths::for_rpx_source(&rpx);
    assert_eq!(snap.primary, std::path::PathBuf::from("docs/article.rpxsnap"));
}

#[test]
fn source_journal_roundtrip_and_clear() {
    let dir = std::env::temp_dir().join(format!(
        "rpx-src-journal-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let rpx = dir.join("doc.rpx");
    write_source_journal(&rpx, b"edited source").unwrap();
    assert_eq!(
        read_source_journal(&rpx).unwrap().as_deref(),
        Some("edited source")
    );
    clear_source_journal(&rpx);
    assert!(read_source_journal(&rpx).unwrap().is_none());
    let _ = std::fs::remove_dir_all(&dir);
}
