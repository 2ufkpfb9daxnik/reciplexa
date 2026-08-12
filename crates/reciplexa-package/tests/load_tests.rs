use std::collections::HashMap;
use std::path::PathBuf;

use reciplexa_eval::{eval_expr, primitive_env, UnitHost};
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
fn resolve_japanese_jlreq_modules() {
    let idx = index();
    for mod_path in [
        "japanese/classes",
        "japanese/linebreak",
        "japanese/kihon",
        "japanese/markup",
    ] {
        let (name, src) = idx.resolve_import(mod_path).unwrap();
        assert_eq!(name, mod_path);
        assert!(!src.is_empty(), "{mod_path} empty");
    }
    let (_, classes) = idx.resolve_import("japanese/classes").unwrap();
    assert!(classes.contains("cl-01") && classes.contains("all-class-ids"));
    assert!(classes.contains("cl-19") && classes.contains("class-name"));
    assert!(classes.contains("cl-30") && classes.contains("is-punctuation-class?"));
    let (_, lb) = idx.resolve_import("japanese/linebreak").unwrap();
    assert!(lb.contains("break-between") && lb.contains("kinsoku-profile"));
    assert!(lb.contains("sample-pair-rules"));
    let (_, kihon) = idx.resolve_import("japanese/kihon").unwrap();
    assert!(kihon.contains("kihon-hanmen") && kihon.contains("line-rate-default"));
    assert!(kihon.contains("a5-trim") && kihon.contains("place-hanmen"));
    let exports = idx
        .resolve_import_detailed("japanese/classes")
        .unwrap()
        .interface_exports
        .unwrap();
    assert!(exports.contains(&"cl-19".into()));
    assert!(exports.contains(&"cl-13".into()));
    assert!(exports.contains(&"cl-20".into()));
    assert!(exports.contains(&"cl-30".into()));
    assert!(exports.contains(&"advance-em".into()));
    assert!(exports.contains(&"all-class-ids".into()));
    assert!(exports.contains(&"is-kana-class?".into()));
}

#[test]
fn resolve_math_domain_modules() {
    let idx = index();
    for mod_path in [
        "math/atoms",
        "math/scripts",
        "math/frac",
        "math/sqrt",
        "math/delimiters",
        "math/matrix",
        "math/accents",
        "math/bigops",
        "math/cases",
    ] {
        let (name, src) = idx.resolve_import(mod_path).unwrap();
        assert_eq!(name, mod_path);
        assert!(!src.is_empty(), "{mod_path} empty");
    }
    let (_, atoms) = idx.resolve_import("math/atoms").unwrap();
    assert!(atoms.contains("class-ord") && atoms.contains("(val bin"));
    let (_, sqrt) = idx.resolve_import("math/sqrt").unwrap();
    assert!(sqrt.contains("radical-indexed"));
    let (_, matrix) = idx.resolve_import("math/matrix").unwrap();
    assert!(matrix.contains("bmatrix") && matrix.contains("math-matrix"));
    let (_, accents) = idx.resolve_import("math/accents").unwrap();
    assert!(accents.contains("hat") && accents.contains("math-accent"));
    let (_, bigops) = idx.resolve_import("math/bigops").unwrap();
    assert!(bigops.contains("sum") && bigops.contains("integral") && bigops.contains("withlimits"));
    let (_, cases) = idx.resolve_import("math/cases").unwrap();
    assert!(cases.contains("piecewise") && cases.contains("math-cases"));
    let (_, scripts) = idx.resolve_import("math/scripts").unwrap();
    assert!(scripts.contains("underbrace") && scripts.contains("overset"));
    let exports = idx
        .resolve_import_detailed("math/delimiters")
        .unwrap()
        .interface_exports
        .unwrap();
    assert!(exports.contains(&"paren".into()));
    assert!(exports.contains(&"delimiter".into()));
}

#[test]
fn elaborate_and_eval_japanese_jlreq_example() {
    let idx = index();
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_japanese_jlreq.rpx");
    let units = load_module_tree_with_packages(&entry, &idx).unwrap();
    assert!(units.iter().any(|(n, _)| n == "japanese/classes"));
    assert!(units.iter().any(|(n, _)| n == "japanese/linebreak"));
    assert!(units.iter().any(|(n, _)| n == "japanese/kihon"));
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    let demo = units
        .iter()
        .find(|u| u.name == "pkg_japanese_jlreq")
        .unwrap();
    let v = eval_expr(&demo.expr, &primitive_env(), &mut UnitHost).unwrap();
    let s = format!("{v}");
    assert!(
        s.contains("ja-jlreq-demo")
            || s.contains("ja-doc")
            || s.contains("kihon-hanmen")
            || s.contains("jlreq"),
        "expected japanese jlreq tree, got {s}"
    );
}

#[test]
fn elaborate_and_eval_math_package_example() {
    // Linking many math modules nests Lets deeply; bump thread stack on Windows debug.
    std::thread::Builder::new()
        .name("math-pkg-eval".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let idx = index();
            let entry =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_math.rpx");
            let units = load_module_tree_with_packages(&entry, &idx).unwrap();
            assert!(units.iter().any(|(n, _)| n == "math/atoms"));
            assert!(units.iter().any(|(n, _)| n == "math/frac"));
            assert!(units.iter().any(|(n, _)| n == "math/sqrt"));
            assert!(units.iter().any(|(n, _)| n == "math/matrix"));
            assert!(units.iter().any(|(n, _)| n == "math/accents"));
            assert!(units.iter().any(|(n, _)| n == "math/bigops"));
            assert!(units.iter().any(|(n, _)| n == "math/cases"));
            assert!(units.iter().any(|(n, _)| n == "math/scripts"));
            let units = elaborate_with_packages(&entry, &idx).unwrap();
            let demo = units.iter().find(|u| u.name == "pkg_math").unwrap();
            let v = eval_expr(&demo.expr, &primitive_env(), &mut UnitHost).unwrap();
            let s = format!("{v}");
            assert!(
                s.contains("math-demo")
                    || s.contains("math-fraction")
                    || s.contains("math-radical")
                    || s.contains("math-delimiter")
                    || s.contains("math-matrix")
                    || s.contains("math-accent")
                    || s.contains("math-bigop"),
                "expected math package tree, got {s}"
            );
        })
        .expect("spawn math-pkg-eval")
        .join()
        .expect("math-pkg-eval thread");
}

#[test]
fn elaborate_and_eval_pkg_markup_ja_example() {
    let idx = index();
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_markup_ja.rpx");
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    let demo = units.iter().find(|u| u.name == "pkg_markup_ja").unwrap();
    let v = eval_expr(&demo.expr, &primitive_env(), &mut UnitHost).unwrap();
    let s = format!("{v}");
    assert!(
        s.contains("markup-ja-package-demo")
            || s.contains("ja-doc")
            || s.contains("jlreq")
            || s.contains("ideographic"),
        "expected markup_ja package tree, got {s}"
    );
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
fn graphics_value_bridge_matches_interim_lower_source() {
    // Slice D strangler: package-eval page tree → same scene as interim CST lower.
    use reciplexa_eval::document_from_graphics_value;
    use reciplexa_lower::lower_source;
    use reciplexa_scene::Shape;

    let idx = index();
    let dir = tempfile_dir();
    let entry = dir.join("bridge.rpx");
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
    let demo = units.iter().find(|u| u.name == "bridge").unwrap();
    let v = eval_expr(&demo.expr, &HashMap::new(), &mut UnitHost).unwrap();
    let from_pkg = document_from_graphics_value(&v).expect("package bridge");
    let from_cst = lower_source("(page a4 (circle 105 148.5 40))").expect("interim lower");
    assert_eq!(from_pkg.pages.len(), 1);
    assert_eq!(from_cst.pages.len(), 1);
    assert_eq!(from_pkg.pages[0].paper, from_cst.pages[0].paper);
    match (&from_pkg.pages[0].shapes[0], &from_cst.pages[0].shapes[0]) {
        (Shape::Circle(a), Shape::Circle(b)) => {
            assert_eq!(a.x_mm, b.x_mm);
            assert_eq!(a.y_mm, b.y_mm);
            assert_eq!(a.radius_mm, b.radius_mm);
            assert_eq!(a.fill, b.fill);
        }
        other => panic!("expected matching circles, got {other:?}"),
    }
}

#[test]
fn graphics_value_bridge_grows_ellipse_text_transform_opacity() {
    // Slice D step 4/6: more package tags lower like interim CST forms.
    use reciplexa_eval::document_from_graphics_value;
    use reciplexa_lower::lower_source;
    use reciplexa_scene::Shape;

    let idx = index();
    let dir = tempfile_dir();
    let entry = dir.join("bridge_more.rpx");
    std::fs::write(
        &entry,
        r#"(import graphics/shapes only
  circle ellipse text translate rotate scale opacity fill group)
(import graphics/page only a4 page)
(import graphics/color only red black)
(val main
  (page a4
    (group
      (list
        (fill (ellipse 105 220 55 28) red)
        (opacity 0.45
          (translate 105 148.5
            (rotate 30
              (scale 1.5 1.5
                (fill (circle 0 0 20) black)))))
        (text 30 270 6 "shapes")))))
"#,
    )
    .unwrap();
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    let demo = units.iter().find(|u| u.name == "bridge_more").unwrap();
    let v = eval_expr(&demo.expr, &HashMap::new(), &mut UnitHost).unwrap();
    let from_pkg = document_from_graphics_value(&v).expect("package bridge");
    assert_eq!(from_pkg.pages.len(), 1);
    match &from_pkg.pages[0].shapes[0] {
        Shape::Group { children, .. } => {
            assert!(children.iter().any(|s| matches!(s, Shape::Ellipse(_))));
            assert!(children.iter().any(|s| matches!(s, Shape::Opacity { .. })));
            assert!(children.iter().any(|s| matches!(s, Shape::Text(_))));
        }
        other => panic!("expected top-level group, got {other:?}"),
    }

    let from_cst = lower_source(
        r#"(page a4
  (ellipse 105 220 55 28 red)
  (opacity 0.45
    (translate 105 148.5
      (rotate 30
        (scale 1.5
          (circle 0 0 20)))))
  (text 30 270 6 "shapes"))"#,
    )
    .expect("interim lower");
    assert_eq!(from_cst.pages[0].shapes.len(), 3);
}

#[test]
fn document_from_package_entry_adapter_matches_bridge() {
    // Thin opt-in adapter; GUI interim ingest stays default.
    use reciplexa_package::document_from_package_entry;
    use reciplexa_scene::Shape;

    let idx = index();
    let dir = tempfile_dir();
    let entry = dir.join("adapter.rpx");
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
    let doc = document_from_package_entry(&entry, &idx).expect("adapter");
    match &doc.pages[0].shapes[0] {
        Shape::Circle(c) => assert_eq!(c.radius_mm, 40.0),
        other => panic!("expected circle, got {other:?}"),
    }
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
