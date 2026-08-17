//! DN2-7 — portable fallback contract (`OPEN-NATIVE-PKG-001`) and load routing.

use std::fs;
use std::path::PathBuf;

use reciplexa_package::{
    DomainNativeRegistry, LocalPackageIndex, OPEN_NATIVE_PKG_001_CODE, OPEN_NATIVE_PKG_001_FALLBACK,
};

fn scratch() -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.tmp/pkg-dn2-fallback");
    let _ = fs::create_dir_all(&base);
    let dir = base.join(format!(
        "t-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn open_native_pkg_001_contract_is_public() {
    assert_eq!(OPEN_NATIVE_PKG_001_CODE, "OPEN-NATIVE-PKG-001");
    assert!(OPEN_NATIVE_PKG_001_FALLBACK.contains(OPEN_NATIVE_PKG_001_CODE));
    assert!(OPEN_NATIVE_PKG_001_FALLBACK.contains("not implemented"));
}

#[test]
fn non_native_package_still_loads_portable_rpx_from_disk() {
    let root = scratch();
    let pkg = root.join("widgets");
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
    fs::write(
        pkg.join("src/shapes.rpx"),
        "(val circle (fn (_) (record (tag \"portable-circle\"))))\n",
    )
    .unwrap();

    let idx = LocalPackageIndex::discover(&[&root]).unwrap();
    assert!(!idx.native().contains("widgets/shapes"));
    let resolved = idx.resolve_import_detailed("widgets/shapes").unwrap();
    assert!(resolved.source.contains("portable-circle"));
    assert!(!resolved.source.contains("dn2slot-"));
}

#[test]
fn std_native_registry_does_not_implicitly_fallback_to_disk_rpx() {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    let resolved = idx.resolve_import_detailed("length/units").unwrap();
    assert!(resolved.source.contains("dn2slot-length-units"));
    let rpx = packages.join("length/src/units.rpx");
    if rpx.is_file() {
        let disk = fs::read_to_string(&rpx).unwrap();
        assert_ne!(
            resolved.source, disk,
            "native registry must not silently fall back to disk .rpx"
        );
    }
}

#[test]
fn empty_native_registry_keeps_portable_load_path() {
    let root = scratch();
    let pkg = root.join("portable");
    fs::create_dir_all(pkg.join("src")).unwrap();
    fs::write(
        pkg.join("package.rpxm"),
        r#"(package portable
  format-version 1
  version "0.1.0"
  source-root "src"
  (public-modules main))"#,
    )
    .unwrap();
    fs::write(pkg.join("src/main.rpx"), "(val answer 42)\n").unwrap();

    let idx = LocalPackageIndex::discover(&[&root])
        .unwrap()
        .with_native(DomainNativeRegistry::empty());
    let resolved = idx.resolve_import_detailed("portable/main").unwrap();
    assert!(resolved.source.contains("answer"));
    assert!(!resolved.source.contains("dn2slot-"));
}
