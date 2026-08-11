use std::collections::HashMap;
use std::path::PathBuf;

use reciplexa_eval::{eval_expr, UnitHost};
use reciplexa_package::{
    elaborate_with_packages, load_module_tree_with_packages, parse_rpxm, LocalPackageIndex,
    Lockfile,
};

fn workspace_packages() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages")
}

fn index() -> LocalPackageIndex {
    LocalPackageIndex::discover(&[workspace_packages()]).expect("discover packages/")
}

#[test]
fn discovers_std_packages() {
    let idx = index();
    let names: Vec<_> = idx.package_names().collect();
    assert!(names.contains(&"graphics"));
    assert!(names.contains(&"length"));
    assert!(names.contains(&"color"));
    assert!(names.contains(&"math"));
    assert!(names.contains(&"japanese"));
}

#[test]
fn parses_graphics_manifest_from_disk() {
    let src = std::fs::read_to_string(workspace_packages().join("graphics/package.rpxm")).unwrap();
    let m = parse_rpxm(&src).unwrap();
    assert_eq!(m.name, "graphics");
    assert!(m.public_modules.iter().any(|p| p == "shapes"));
}

#[test]
fn resolves_graphics_shapes_module() {
    let (name, src) = index().resolve_import("graphics/shapes").unwrap();
    assert_eq!(name, "graphics/shapes");
    assert!(src.contains("(val circle"));
}

#[test]
fn load_entry_imports_graphics_shapes() {
    let dir = tempfile_dir();
    let entry = dir.join("demo.rpx");
    std::fs::write(
        &entry,
        r#"(import graphics/shapes only circle)
(val main (circle 105 148.5 40))
"#,
    )
    .unwrap();
    let units = load_module_tree_with_packages(&entry, &index()).unwrap();
    assert!(units.iter().any(|(n, _)| n == "graphics/shapes"));
    assert_eq!(units[0].0, "demo");
}

#[test]
fn elaborate_and_eval_circle_from_package() {
    let dir = tempfile_dir();
    let entry = dir.join("demo.rpx");
    std::fs::write(
        &entry,
        r#"(import graphics/shapes only circle)
(val main (circle 105 148.5 40))
"#,
    )
    .unwrap();
    let units = elaborate_with_packages(&entry, &index()).unwrap();
    let demo = units.iter().find(|u| u.name == "demo").unwrap();
    let v = eval_expr(&demo.expr, &HashMap::new(), &mut UnitHost).unwrap();
    let s = format!("{v}");
    assert!(
        s.contains("circle") || s.contains("105"),
        "expected circle record value, got {s}"
    );
}

#[test]
fn path_dep_alias_resolves_import() {
    let consumer = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_consumer");
    let (idx, manifest) =
        LocalPackageIndex::discover_with_consumer(&[workspace_packages()], &consumer).unwrap();
    assert_eq!(manifest.name, "pkg-consumer");
    assert_eq!(idx.resolve_alias("g"), "graphics");
    let (name, src) = idx.resolve_import("g/shapes").unwrap();
    assert_eq!(name, "g/shapes");
    assert!(src.contains("(val circle"));
}

#[test]
fn path_dep_lockfile_roundtrip() {
    let consumer = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_consumer");
    let (idx, manifest) =
        LocalPackageIndex::discover_with_consumer(&[workspace_packages()], &consumer).unwrap();
    let lock = idx.lock_consumer(&manifest).unwrap();
    assert!(lock.packages.iter().any(|p| p.source.starts_with("path:")));
    let dir = tempfile_dir();
    let lock_path = dir.join("rpx.lock");
    lock.write_rpx_lock(&lock_path).unwrap();
    let loaded = Lockfile::read_rpx_lock(&lock_path).unwrap();
    assert_eq!(loaded, lock);
}

#[test]
fn resolve_math_and_japanese_stubs() {
    let idx = index();
    let (_, math) = idx.resolve_import("math/atoms").unwrap();
    assert!(math.contains("math-symbol") || math.contains("symbol"));
    let (_, ja) = idx.resolve_import("japanese/markup").unwrap();
    assert!(ja.contains("ja-heading") || ja.contains("heading"));
}

#[test]
fn elaborate_consumer_via_alias_import() {
    let consumer = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_consumer");
    let (idx, _) =
        LocalPackageIndex::discover_with_consumer(&[workspace_packages()], &consumer).unwrap();
    let entry = consumer.join("src/main.rpx");
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    let main = units.iter().find(|u| u.name == "main").unwrap();
    let v = eval_expr(&main.expr, &HashMap::new(), &mut UnitHost).unwrap();
    let s = format!("{v}");
    assert!(s.contains("circle") || s.contains("10"), "got {s}");
}

#[test]
fn rpi_export_boundary_hides_internal() {
    let resolved = index()
        .resolve_import_detailed("graphics/shapes")
        .expect("shapes");
    let exports = resolved.interface_exports.expect("shapes.rpi");
    assert!(exports.contains(&"circle".into()));
    assert!(!exports.iter().any(|e| e == "shapes-internal-tag"));

    let dir = tempfile_dir();
    let entry = dir.join("demo.rpx");
    std::fs::write(
        &entry,
        r#"(import graphics/shapes only shapes-internal-tag)
(val main (shapes-internal-tag "x"))
"#,
    )
    .unwrap();
    let err = elaborate_with_packages(&entry, &index()).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("not exported") && msg.contains("shapes-internal-tag"),
        "unexpected: {msg}"
    );
}

#[test]
fn interface_path_resolves_under_interface_root() {
    let idx = index();
    let (root, manifest) = idx.get("graphics").expect("graphics");
    let path = manifest
        .module_interface_path(root, "shapes")
        .expect("interface-root");
    assert!(path.ends_with("interface\\shapes.rpi") || path.ends_with("interface/shapes.rpi"));
    assert!(path.is_file());
}

fn tempfile_dir() -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.tmp/pkg-load-tests");
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
