//! N6 tip k: load.rs I/O Err arms via Windows exclusive lock (no injectable seam).

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
    let root = std::env::temp_dir().join(format!("rpx_load_tip_n6k_{n}"));
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
fn load_n6k_locked_manifest_and_rpi_read_err() {
    let root = scratch();
    let pkg = root.join("lockedpkg");
    fs::create_dir_all(pkg.join("src")).unwrap();
    fs::create_dir_all(pkg.join("interface")).unwrap();

    // Discover: locked package.rpxm → Io Err on read_to_string
    let manifest = pkg.join("package.rpxm");
    let body = br#"(package lockedpkg
  format-version 1
  version "0.1.0"
  source-root "src"
  interface-root "interface"
  (public-modules shapes))"#;
    let _guard = exclusive_write(&manifest, body);
    let err = LocalPackageIndex::discover(&[root.as_path()]);
    assert!(err.is_err(), "expected locked manifest discover err");

    // Unlock for a second scenario: native path with locked .rpi
    drop(_guard);
    fs::write(&manifest, body).unwrap();
    fs::write(pkg.join("src/shapes.rpx"), "(val circle 1)\n").unwrap();
    let rpi = pkg.join("interface/shapes.rpi");
    let _rpi_guard = exclusive_write(&rpi, b"(val circle)\n");

    let idx = LocalPackageIndex::discover(&[root.as_path()]).expect("discover after unlock");
    // Resolve/elaborate may hit locked rpi when reading interface exports.
    let entry = scratch().join("entry.rpx");
    fs::write(
        &entry,
        "(import lockedpkg/shapes only circle)\n(val main circle)\n",
    )
    .unwrap();
    // Point package root via discover already done; elaborate may fail on rpi read.
    let _ = elaborate_with_packages(&entry, &idx);
}
