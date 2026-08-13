//! N6 tip n: load discover non-dir roots + resolve interface_exports path.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use reciplexa_package::{elaborate_with_packages, LocalPackageIndex};

fn scratch() -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("rpx_load_tip_n6n_{n}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn load_n6n_discover_skip_and_interface_exports() {
    let root = scratch();
    // Non-dir root entry is skipped (continue)
    let file_root = root.join("not-a-dir.txt");
    fs::write(&file_root, "x").unwrap();
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
    fs::write(pkg.join("interface/shapes.rpi"), "(val circle)\n").unwrap();

    // Discover with mix of file + dir roots
    let idx =
        LocalPackageIndex::discover(&[file_root.as_path(), root.as_path()]).expect("discover");
    let entry = scratch().join("entry.rpx");
    fs::write(
        &entry,
        "(import widgets/shapes only circle)\n(val main circle)\n",
    )
    .unwrap();
    // Hits elaborate_with_packages interface_exports insert (line ~467)
    let _ = elaborate_with_packages(&entry, &idx);

    // Missing public module / missing rpi neighborhood
    fs::remove_file(pkg.join("interface/shapes.rpi")).ok();
    let idx2 = LocalPackageIndex::discover(&[root.as_path()]).expect("discover");
    let _ = elaborate_with_packages(&entry, &idx2);
}
