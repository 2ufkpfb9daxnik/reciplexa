use std::collections::HashMap;
use std::path::PathBuf;

use reciplexa_eval::{eval_expr, UnitHost};
use reciplexa_package::{
    elaborate_with_packages, load_module_tree_with_packages, parse_rpxm, LocalPackageIndex,
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
