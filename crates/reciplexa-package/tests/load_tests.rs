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
    assert!(lock
        .packages
        .iter()
        .any(|p| p.name == "pkg-consumer" && !p.dependencies.is_empty()));
    lock.is_consistent_with_consumer(&manifest).unwrap();
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

#[test]
fn elaborate_and_eval_static_graphics_surface() {
    let idx = index();
    let (shapes_name, shapes_src) = idx.resolve_import("graphics/shapes").unwrap();
    assert_eq!(shapes_name, "graphics/shapes");
    assert!(shapes_src.contains("(val line"));
    assert!(shapes_src.contains("(val path"));
    assert!(shapes_src.contains("(val fill"));
    assert!(shapes_src.contains("(val stroke"));
    assert!(shapes_src.contains("(val text"));
    assert!(shapes_src.contains("(val image"));
    assert!(shapes_src.contains("(val translate"));
    assert!(shapes_src.contains("(val opacity"));

    let (_, page_src) = idx.resolve_import("graphics/page").unwrap();
    assert!(page_src.contains("(val a3"));
    assert!(page_src.contains("(val page-size"));

    let (_, color_src) = idx.resolve_import("graphics/color").unwrap();
    assert!(color_src.contains("(val rgba"));
    assert!(color_src.contains("(val transparent"));

    let detailed = idx
        .resolve_import_detailed("graphics/shapes")
        .expect("shapes");
    let exports = detailed.interface_exports.expect("shapes.rpi");
    assert!(exports.contains(&"line".into()));
    assert!(exports.contains(&"path".into()));
    assert!(exports.contains(&"paint".into()));
    assert!(exports.contains(&"text".into()));
    assert!(exports.contains(&"image".into()));
    assert!(exports.contains(&"opacity".into()));
    assert!(!exports.iter().any(|e| e == "shapes-internal-tag"));

    let dir = tempfile_dir();
    let entry = dir.join("demo.rpx");
    std::fs::write(
        &entry,
        r#"(import graphics/shapes only circle fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main
  (page a4
    (fill (circle 105 148.5 40) black)))
"#,
    )
    .unwrap();
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    let demo = units.iter().find(|u| u.name == "demo").unwrap();
    let v = eval_expr(&demo.expr, &HashMap::new(), &mut UnitHost).unwrap();
    let s = format!("{v}");
    assert!(
        s.contains("page") || s.contains("circle") || s.contains("fill") || s.contains("210"),
        "expected static graphics page tree, got {s}"
    );
}

#[test]
fn load_full_graphics_package_tree_and_eval_transforms() {
    let idx = index();
    for mod_path in ["graphics/shapes", "graphics/page", "graphics/color"] {
        let (name, _) = idx.resolve_import(mod_path).unwrap();
        assert_eq!(name, mod_path);
    }
    let rpi = idx
        .resolve_import_detailed("graphics/shapes")
        .unwrap()
        .interface_exports
        .unwrap();
    for export in [
        "circle",
        "rect",
        "ellipse",
        "line",
        "path",
        "polyline",
        "polygon",
        "ring",
        "frame",
        "group",
        "text",
        "text-box",
        "image",
        "translate",
        "rotate",
        "scale",
        "opacity",
        "fill",
        "stroke",
        "paint",
    ] {
        assert!(rpi.iter().any(|e| e == export), "missing export {export}");
    }

    let dir = tempfile_dir();
    let entry = dir.join("xf.rpx");
    std::fs::write(
        &entry,
        r#"(import graphics/shapes only
  circle text image translate rotate scale opacity fill group)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main
  (page a4
    (opacity 0.7
      (translate 10 20
        (rotate 5
          (scale 1 1
            (group
              (list
                (fill (circle 0 0 10) black)
                (text 15 0 12 "hi")
                (image "x.png" 0 20 30 20)))))))))
"#,
    )
    .unwrap();
    let units = load_module_tree_with_packages(&entry, &idx).unwrap();
    assert!(units.iter().any(|(n, _)| n == "graphics/shapes"));
    assert!(units.iter().any(|(n, _)| n == "graphics/page"));
    assert!(units.iter().any(|(n, _)| n == "graphics/color"));
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    let demo = units.iter().find(|u| u.name == "xf").unwrap();
    let v = eval_expr(&demo.expr, &HashMap::new(), &mut UnitHost).unwrap();
    let s = format!("{v}");
    assert!(
        s.contains("opacity")
            || s.contains("translate")
            || s.contains("text")
            || s.contains("page"),
        "expected transform/text/image package tree, got {s}"
    );
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
