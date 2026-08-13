//! N6 tip m: load.rs locked .rpi under native resolve path (line ~277 Io).

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use reciplexa_package::{elaborate_with_packages, LocalPackageIndex};

fn scratch() -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("rpx_load_tip_n6m_{n}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[cfg(windows)]
fn exclusive_write(path: &std::path::Path, bytes: &[u8]) -> File {
    use std::os::windows::fs::OpenOptionsExt;
    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .share_mode(0)
        .open(path)
        .expect("exclusive");
    f.write_all(bytes).unwrap();
    f.flush().unwrap();
    f
}

#[cfg(windows)]
#[test]
fn load_n6m_locked_rpi_on_native_and_source_paths() {
    let root = scratch();
    let pkg = root.join("widgets");
    fs::create_dir_all(pkg.join("src")).unwrap();
    fs::create_dir_all(pkg.join("interface")).unwrap();
    fs::write(
        pkg.join("package.rpxm"),
        r#"(package widgets
  format-version 1
  version "0.1.0"
  source-root "src"
  interface-root "interface"
  (public-modules shapes))"#,
    )
    .unwrap();
    fs::write(pkg.join("src/shapes.rpx"), "(val circle 1)\n").unwrap();

    // Locked .rpi while resolving public module with interface-root
    let rpi = pkg.join("interface/shapes.rpi");
    let _guard = exclusive_write(&rpi, b"(val circle)\n");
    let idx = LocalPackageIndex::discover(&[root.as_path()]).expect("discover");
    let entry = scratch().join("entry.rpx");
    fs::write(
        &entry,
        "(import widgets/shapes only circle)\n(val main circle)\n",
    )
    .unwrap();
    let err = elaborate_with_packages(&entry, &idx);
    assert!(err.is_err(), "expected locked rpi Io err");
    drop(_guard);

    // Re-tip unlocked path for Ok coverage nearby
    fs::write(&rpi, b"(val circle)\n").unwrap();
    let idx = LocalPackageIndex::discover(&[root.as_path()]).expect("discover");
    let _ = elaborate_with_packages(&entry, &idx);
}
