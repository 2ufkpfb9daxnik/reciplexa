//! llvm-cov tip: graphics_bridge adapter + From/error paths.

use std::path::PathBuf;

use reciplexa_eval::{document_from_graphics_value, RuntimeValue};
use reciplexa_package::{
    document_from_package_entry, document_from_package_source, elaborate_with_packages,
    GraphicsBridgeError, LocalPackageIndex, PackageLoadError,
};
use reciplexa_scene::Shape;

fn workspace_packages() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages")
}

fn index() -> LocalPackageIndex {
    LocalPackageIndex::discover(&[workspace_packages()]).expect("discover packages/")
}

fn scratch() -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.tmp/pkg-bridge-cov");
    let _ = std::fs::create_dir_all(&base);
    let dir = base.join(format!(
        "t-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn graphics_bridge_from_impls() {
    let load: GraphicsBridgeError = PackageLoadError::NotFound("nf".into()).into();
    assert!(matches!(load, GraphicsBridgeError::Load(s) if s.contains("nf")));

    let gve = document_from_graphics_value(&RuntimeValue::Int(42)).unwrap_err();
    let bridge: GraphicsBridgeError = gve.into();
    assert!(matches!(bridge, GraphicsBridgeError::Bridge(_)));
}

#[test]
fn document_from_package_source_happy_path() {
    let idx = index();
    let source = r#"(import graphics/shapes only circle fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main
  (page a4
    (fill (circle 105 148.5 40) black)))
"#;
    let doc = document_from_package_source(source, "bridge_src", &idx).expect("happy path");
    match &doc.pages[0].shapes[0] {
        Shape::Circle(c) => assert_eq!(c.radius_mm, 40.0),
        other => panic!("expected circle, got {other:?}"),
    }
}

#[test]
fn document_from_package_entry_load_error_bad_import() {
    let idx = index();
    let dir = scratch();
    let entry = dir.join("bad_import.rpx");
    std::fs::write(
        &entry,
        r#"(import no/such/pkg)
(val main 1)
"#,
    )
    .unwrap();
    let err = document_from_package_entry(&entry, &idx).unwrap_err();
    assert!(matches!(err, GraphicsBridgeError::Load(_)), "{err:?}");
}

#[test]
fn document_from_package_entry_eval_error() {
    let idx = index();
    let dir = scratch();
    let entry = dir.join("eval_err.rpx");
    std::fs::write(
        &entry,
        r#"(import graphics/shapes only circle)
(val main (circle (+ 1 "not-a-number") 2 3))
"#,
    )
    .unwrap();
    let err = document_from_package_entry(&entry, &idx).unwrap_err();
    assert!(matches!(err, GraphicsBridgeError::Eval(_)), "{err:?}");
}

#[test]
fn document_from_package_entry_bridge_error_main_is_int() {
    let idx = index();
    let dir = scratch();
    let entry = dir.join("bridge_err.rpx");
    std::fs::write(&entry, "(val main 42)\n").unwrap();
    let err = document_from_package_entry(&entry, &idx).unwrap_err();
    assert!(matches!(err, GraphicsBridgeError::Bridge(_)), "{err:?}");
}

#[test]
fn document_from_package_entry_missing_elaborated_unit_for_stem() {
    // Directory entry: elaborated units are file stems inside, not the directory name.
    let idx = index();
    let dir = scratch().join("moddir");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.rpx"), "(val x 1)\n").unwrap();
    let err = document_from_package_entry(&dir, &idx).unwrap_err();
    match &err {
        GraphicsBridgeError::Load(s) => {
            assert!(s.contains("missing elaborated unit `moddir`"), "{s}");
        }
        other => panic!("expected Load, got {other:?}"),
    }
}

#[test]
fn document_from_package_source_fs_fail_create_dir() {
    let idx = index();
    reciplexa_package::graphics_bridge::test_set_fail_create_dir(true);
    let err = document_from_package_source("(val main 1)", "fs_fail_dir", &idx);
    reciplexa_package::graphics_bridge::test_set_fail_create_dir(false);
    let err = err.expect_err("create_dir_all fail");
    assert!(matches!(err, GraphicsBridgeError::Load(s) if s.contains("TEST_FAIL_CREATE_DIR")));
}

#[test]
fn document_from_package_source_fs_fail_write() {
    let idx = index();
    reciplexa_package::graphics_bridge::test_set_fail_write(true);
    let err = document_from_package_source("(val main 1)", "fs_fail_write", &idx);
    reciplexa_package::graphics_bridge::test_set_fail_write(false);
    let err = err.expect_err("write fail");
    assert!(matches!(err, GraphicsBridgeError::Load(s) if s.contains("TEST_FAIL_WRITE")));
}

#[test]
fn elaborate_with_packages_populates_interface_exports_for_graphics() {
    // Hits load.rs interface_exports.insert via package import with .rpi boundary.
    let idx = index();
    let dir = scratch();
    let entry = dir.join("iface.rpx");
    std::fs::write(
        &entry,
        r#"(import graphics/shapes only circle)
(val main (circle 1 2 3))
"#,
    )
    .unwrap();
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    assert!(units.iter().any(|u| u.name == "graphics/shapes"));
    assert!(units.iter().any(|u| u.name == "iface"));
}

#[test]
fn document_package_entry_lowers_doc_page() {
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_document.rpx");
    let idx = index();
    let doc = document_from_package_entry(&entry, &idx).expect("pkg_document bridge");
    assert_eq!(doc.pages.len(), 1);
    assert_eq!(doc.pages[0].paper.width_mm, 210.0);
    let texts: Vec<_> = doc.pages[0]
        .shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t.content.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        texts.iter().any(|t| t.contains("Document surface")),
        "expected heading text, got {texts:?}"
    );
    assert!(
        texts.iter().any(|t| t.contains("Native document")),
        "expected paragraph text, got {texts:?}"
    );
}
