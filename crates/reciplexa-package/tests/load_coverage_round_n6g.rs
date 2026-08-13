//! Tip load.rs residuals: non-native module with on-disk `.rpi`, and
//! `elaborate_with_packages` interface overlay when resolve succeeds.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use reciplexa_package::{elaborate_with_packages, LocalPackageIndex};

fn scratch() -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("rpx_load_tip_n6g_{n}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn portable_module_with_rpi_reads_exports() {
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
    fs::write(pkg.join("src/shapes.rpx"), "(val circle 1)\n(val rect 2)\n").unwrap();
    fs::write(
        pkg.join("interface/shapes.rpi"),
        "(val circle)\n(val rect)\n",
    )
    .unwrap();

    let idx = LocalPackageIndex::discover(&[&root]).unwrap();
    let resolved = idx.resolve_import_detailed("widgets/shapes").unwrap();
    let exports = resolved.interface_exports.expect("rpi exports");
    assert!(exports.iter().any(|e| e == "circle"));
    assert!(exports.iter().any(|e| e == "rect"));
    assert!(resolved.source.contains("circle"));

    // Entry that imports the portable module → elaborate_with_packages overlay.
    let entry = root.join("entry.rpx");
    fs::write(
        &entry,
        "(import widgets/shapes only circle)\n(val main circle)\n",
    )
    .unwrap();
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    assert!(units
        .iter()
        .any(|u| u.name.contains("shapes") || u.name == "entry"));
}
