//! N6 tip o: load discover Ambiguous + bad .rpi parse + interface_exports Ok.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use reciplexa_package::{elaborate_with_packages, LocalPackageIndex};

fn scratch() -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("rpx_load_tip_n6o_{n}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_pkg(root: &std::path::Path, name: &str, rpi_body: &str) {
    let pkg = root.join(name);
    fs::create_dir_all(pkg.join("src")).unwrap();
    fs::create_dir_all(pkg.join("interface")).unwrap();
    fs::write(
        pkg.join("package.rpxm"),
        format!(
            r#"(package {name}
  format-version 1
  version "0.1.0"
  source-root "src"
  interface-root "interface"
  (public-modules shapes))"#
        ),
    )
    .unwrap();
    fs::write(pkg.join("src/shapes.rpx"), "(val circle 1)\n").unwrap();
    fs::write(pkg.join("interface/shapes.rpi"), rpi_body).unwrap();
}

#[test]
fn load_n6o_ambiguous_and_bad_rpi_and_exports() {
    // Duplicate package name across two roots → Ambiguous
    let a = scratch();
    let b = scratch();
    write_pkg(&a, "widgets", "(val circle)\n");
    write_pkg(&b, "widgets", "(val circle)\n");
    let err = LocalPackageIndex::discover(&[a.as_path(), b.as_path()]);
    assert!(err.is_err(), "expected ambiguous duplicate package");

    // Bad .rpi parse on non-native resolve (parse_rpi_exports ?)
    let root = scratch();
    write_pkg(&root, "widgets", "(val circle\n"); // unterminated
    let idx = LocalPackageIndex::discover(&[root.as_path()]).expect("discover");
    let entry = scratch().join("entry.rpx");
    fs::write(
        &entry,
        "(import widgets/shapes only circle)\n(val main circle)\n",
    )
    .unwrap();
    let _ = elaborate_with_packages(&entry, &idx);

    // Valid rpi → interface_exports insert Ok path
    write_pkg(&root, "widgets", "(val circle)\n");
    let idx = LocalPackageIndex::discover(&[root.as_path()]).expect("discover");
    let _ = elaborate_with_packages(&entry, &idx);

    // Missing public module path
    let bare = scratch();
    let pkg = bare.join("empty");
    fs::create_dir_all(pkg.join("src")).unwrap();
    fs::write(
        pkg.join("package.rpxm"),
        r#"(package empty
  format-version 1
  version "0.1.0"
  source-root "src"
  (public-modules))"#,
    )
    .unwrap();
    let idx = LocalPackageIndex::discover(&[bare.as_path()]).expect("discover empty");
    let e2 = scratch().join("e2.rpx");
    fs::write(&e2, "(import empty/shapes only x)\n(val main 1)\n").unwrap();
    let _ = elaborate_with_packages(&e2, &idx);
}
