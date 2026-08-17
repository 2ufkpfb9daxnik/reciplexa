//! N6 tip o2: native registry + locked .rpi hits load.rs L276 Io path.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use reciplexa_package::{elaborate_with_packages, DomainNativeModule, LocalPackageIndex};

fn scratch() -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("rpx_load_tip_n6o2_{n}"));
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
fn load_n6o2_native_locked_rpi_io() {
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
    let _guard = exclusive_write(&rpi, b"(val circle)\n");

    let mut idx = LocalPackageIndex::discover(&[root.as_path()]).expect("discover");
    idx.register_native(DomainNativeModule::direct_native(
        "widgets/shapes",
        vec!["circle".into()],
    ));
    idx.native_mut()
        .set_reference_body("widgets/shapes", "(val circle 1)\n");
    let entry = scratch().join("entry.rpx");
    fs::write(
        &entry,
        "(import widgets/shapes only circle)\n(val main circle)\n",
    )
    .unwrap();
    let err = elaborate_with_packages(&entry, &idx);
    assert!(err.is_err(), "expected locked native rpi Io");
    drop(_guard);

    // Unlocked: native + existing rpi → read Ok then parse_rpi
    fs::write(&rpi, b"(val circle)\n").unwrap();
    let mut idx = LocalPackageIndex::discover(&[root.as_path()]).expect("discover");
    idx.register_native(DomainNativeModule::direct_native(
        "widgets/shapes",
        vec!["circle".into()],
    ));
    idx.native_mut()
        .set_reference_body("widgets/shapes", "(val circle 1)\n");
    let _ = elaborate_with_packages(&entry, &idx);
}
