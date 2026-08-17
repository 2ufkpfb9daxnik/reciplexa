//! N6 tip p: load discover Io / duplicate / interface resolve residuals.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use reciplexa_package::{
    elaborate_with_packages, load_module_tree_with_packages, DomainNativeModule, LocalPackageIndex,
};

fn scratch(tag: &str) -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("rpx_load_n6p_{tag}_{n}"));
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

#[test]
fn load_n6p_discover_dup_and_interface() {
    let root = scratch("disc");
    // Non-dir search root skipped
    let file_root = root.join("nota_dir");
    fs::write(&file_root, b"x").unwrap();
    let _ = LocalPackageIndex::discover(&[file_root.as_path(), root.as_path()]);

    // Two packages with same name → Ambiguous
    for name in ["a", "b"] {
        let pkg = root.join(name);
        fs::create_dir_all(pkg.join("src")).unwrap();
        fs::write(
            pkg.join("package.rpxm"),
            r#"(package widgets
  format-version 1
  version "0.1.0"
  source-root "src"
  (public-modules shapes))"#,
        )
        .unwrap();
        fs::write(pkg.join("src/shapes.rpx"), "(val circle 1)\n").unwrap();
    }
    let err = LocalPackageIndex::discover(&[root.as_path()]);
    assert!(err.is_err(), "expected duplicate package");

    // Clean single package + interface-root elaborate
    let root2 = scratch("iface");
    let pkg = root2.join("widgets");
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
    fs::write(
        pkg.join("src/shapes.rpx"),
        "(val circle 1)\n(val hidden 2)\n",
    )
    .unwrap();
    fs::write(pkg.join("interface/shapes.rpi"), "(val circle)\n").unwrap();

    let idx = LocalPackageIndex::discover(&[root2.as_path()]).expect("discover");
    let entry = root2.join("entry.rpx");
    fs::write(
        &entry,
        "(import widgets/shapes only circle)\n(val main circle)\n",
    )
    .unwrap();
    let _ = load_module_tree_with_packages(&entry, &idx);
    let _ = elaborate_with_packages(&entry, &idx);

    // Native without .rpi file → exports from native
    let root3 = scratch("native");
    let pkg = root3.join("widgets");
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
    // no shapes.rpi
    let mut idx = LocalPackageIndex::discover(&[root3.as_path()]).expect("discover");
    idx.register_native(DomainNativeModule::hybrid(
        "widgets/shapes".into(),
        vec!["circle".into()],
        "(val circle 1)\n".into(),
    ));
    let entry = root3.join("entry.rpx");
    fs::write(
        &entry,
        "(import widgets/shapes only circle)\n(val main circle)\n",
    )
    .unwrap();
    let _ = elaborate_with_packages(&entry, &idx);

    // Directory-mode entry
    let dir_entry = scratch("dirmode");
    fs::write(dir_entry.join("main.rpx"), "(val main 1)\n").unwrap();
    let idx = LocalPackageIndex::default();
    let _ = load_module_tree_with_packages(&dir_entry, &idx);

    // Missing path
    let _ = load_module_tree_with_packages(root3.join("nope.rpx"), &idx);
}

#[cfg(windows)]
#[test]
fn load_n6p_locked_manifest_and_rpi() {
    let root = scratch("lock");
    let pkg = root.join("widgets");
    fs::create_dir_all(pkg.join("src")).unwrap();
    fs::create_dir_all(pkg.join("interface")).unwrap();
    let manifest = pkg.join("package.rpxm");
    let _g = exclusive_write(
        &manifest,
        br#"(package widgets
  format-version 1
  version "0.1.0"
  source-root "src"
  interface-root "interface"
  (public-modules shapes))"#,
    );
    // discover may fail reading locked manifest
    let _ = LocalPackageIndex::discover(&[root.as_path()]);
    drop(_g);

    fs::write(
        &manifest,
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
    let _lock = exclusive_write(&rpi, b"(val circle)\n");
    let mut idx = LocalPackageIndex::discover(&[root.as_path()]).expect("discover");
    idx.register_native(DomainNativeModule::hybrid(
        "widgets/shapes".into(),
        vec!["circle".into()],
        "(val circle 1)\n".into(),
    ));
    let entry = root.join("entry.rpx");
    fs::write(
        &entry,
        "(import widgets/shapes only circle)\n(val main circle)\n",
    )
    .unwrap();
    let _ = elaborate_with_packages(&entry, &idx);
}
