use reciplexa_codec::atomic::atomic_write;
use std::fs;

use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn atomic_write_creates_file() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("rpx-atomic-{ts}.bin"));
    atomic_write(&path, b"hello").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"hello");
    let _ = fs::remove_file(&path);
}

#[test]
fn atomic_write_overwrites_existing() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("rpx-atomic-ow-{ts}.bin"));
    atomic_write(&path, b"first").unwrap();
    atomic_write(&path, b"second").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"second");
    let _ = fs::remove_file(&path);
}
