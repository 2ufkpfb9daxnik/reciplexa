//! Step 10: editor persistence wiring (atomic save, recovery).

use reciplexa_codec::atomic_write;
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
