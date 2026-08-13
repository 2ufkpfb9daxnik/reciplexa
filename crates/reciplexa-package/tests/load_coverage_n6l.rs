//! N6 tip l: load.rs residual Io — locked .rpx source body + locked sibling.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use reciplexa_package::{elaborate_with_packages, LocalPackageIndex, PackageLoadError};

fn scratch() -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("rpx_load_tip_n6l_{n}"));
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
fn load_n6l_locked_source_body_and_sibling() {
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
    fs::write(pkg.join("interface/shapes.rpi"), "(val circle)\n").unwrap();

    // Locked .rpx body → Io on read during resolve/elaborate
    let src = pkg.join("src/shapes.rpx");
    let _guard = exclusive_write(&src, b"(val circle 1)\n");
    let idx = LocalPackageIndex::discover(&[root.as_path()]).expect("discover");
    let entry = scratch().join("entry.rpx");
    fs::write(
        &entry,
        "(import widgets/shapes only circle)\n(val main circle)\n",
    )
    .unwrap();
    let err = elaborate_with_packages(&entry, &idx);
    assert!(err.is_err(), "expected locked src body err");
    drop(_guard);

    // Locked sibling via bind load_module_tree (no package index)
    let tree = scratch();
    fs::write(tree.join("main.rpx"), "(import sib)\n(val main 1)\n").unwrap();
    let sib = tree.join("sib.rpx");
    let _sib = exclusive_write(&sib, b"(val x 1)\n");
    let err = reciplexa_bind::module::load_module_tree(tree.join("main.rpx"));
    assert!(err.is_err(), "expected locked sibling read err");
}

#[cfg(windows)]
#[test]
fn load_n6l_locked_rpi_via_resolve_and_interface_exports() {
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
    let rpi = pkg.join("interface/shapes.rpi");
    let _rpi = exclusive_write(&rpi, b"(val circle)\n");

    let idx = LocalPackageIndex::discover(&[root.as_path()]).expect("discover");
    let err = idx.resolve_import("widgets/shapes");
    assert!(err.is_err(), "locked rpi resolve");

    // Unlock rpi; tip elaborate_with_packages interface_exports insert path
    drop(_rpi);
    fs::write(&rpi, b"(val circle)\n").unwrap();
    let entry = scratch().join("entry.rpx");
    fs::write(
        &entry,
        "(import widgets/shapes only circle)\n(val main circle)\n",
    )
    .unwrap();
    let _ = elaborate_with_packages(&entry, &idx);
}

#[test]
fn load_n6l_display_and_bad_entry_stem_edges() {
    let err = PackageLoadError::Io("x".into());
    assert!(!err.to_string().is_empty());
    let _ = format!("{err:?}");

    let err2 = PackageLoadError::NotFound("missing".into());
    assert!(err2.to_string().contains("missing"));
    let err3 = PackageLoadError::Ambiguous("a".into());
    assert!(!err3.to_string().is_empty());
    let err4 = PackageLoadError::Interface("i".into());
    assert!(!err4.to_string().is_empty());
}
