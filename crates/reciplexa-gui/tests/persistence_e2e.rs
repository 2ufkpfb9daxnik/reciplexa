//! Step 10: editor persistence wiring (atomic save, recovery).

use reciplexa_codec::{atomic_write, write_journal, RecoveryPaths};
use reciplexa_document::snapshot::DocumentSnapshot;
use reciplexa_gui::persistence::recover_snapshot_sidecar;
use reciplexa_identity::document::DocumentIdentity;
use std::path::PathBuf;

const TEXT_LINE: &str = include_str!("../../../examples/text_line.rpx");

#[test]
fn rpx_atomic_save_roundtrip_and_no_temp_sidecar() {
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-atomic-save-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("text_line.rpx");
    atomic_write(&path, TEXT_LINE.as_bytes()).expect("atomic save");
    let tmp = dir.join(".text_line.rpx.tmp");
    assert!(!tmp.exists(), "temp sidecar must be removed after rename");
    let reloaded = std::fs::read_to_string(&path).expect("reload");
    assert_eq!(reloaded, TEXT_LINE);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rpx_atomic_save_overwrites_prior_content() {
    let dir = PathBuf::from(std::env::temp_dir()).join(format!(
        "reciplexa-atomic-overwrite-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("doc.rpx");
    atomic_write(&path, b"v1").expect("first save");
    atomic_write(&path, b"v2-longer").expect("second save");
    let bytes = std::fs::read(&path).expect("read");
    assert_eq!(bytes, b"v2-longer");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn snapshot_sidecar_recovery_writes_recovered_without_touching_rpx() {
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-snap-recovery-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let rpx = dir.join("doc.rpx");
    atomic_write(&rpx, TEXT_LINE.as_bytes()).expect("write rpx");
    let paths = RecoveryPaths::for_rpx_source(&rpx);
    let snap = DocumentSnapshot::new(DocumentIdentity::new(42));
    write_journal(&paths, &snap).expect("journal");
    assert!(!paths.primary.exists());

    let note = recover_snapshot_sidecar(&rpx).expect("recover");
    assert!(note.is_some());
    assert!(paths.primary.exists());
    assert!(!paths.journal.exists());
    assert!(!paths.recovered.exists());
    assert_eq!(std::fs::read_to_string(&rpx).expect("rpx unchanged"), TEXT_LINE);
    let _ = std::fs::remove_dir_all(&dir);
}
