//! N0.4 / N1 — domain native registry load wiring.

use std::fs;
use std::path::PathBuf;

use reciplexa_package::{
    document_page_module, elaborate_with_packages, length_units_module, length_units_source,
    DomainNativeModule, DomainNativeRegistry, LocalPackageIndex,
};

fn workspace_packages() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages")
}

fn scratch() -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.tmp/pkg-native-cov");
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

fn register_reference_module(
    idx: &mut LocalPackageIndex,
    module_path: &str,
    exports: Vec<String>,
    source: &str,
) {
    idx.register_native(DomainNativeModule::direct_native(module_path, exports));
    idx.set_hybrid_source_override(module_path, source.to_string());
}

#[test]
fn native_index_with_set_and_mut() {
    let mut reg = DomainNativeRegistry::empty();
    reg.register(DomainNativeModule::direct_native(
        "cov/ping",
        vec!["ping".into()],
    ));
    let mut idx = LocalPackageIndex::default().with_native(reg.clone());
    idx.set_hybrid_source_override("cov/ping", "(val ping 1)\n");
    assert!(idx.native().contains("cov/ping"));

    let mut idx2 = LocalPackageIndex::default();
    idx2.set_native(reg);
    idx2.native_mut()
        .register(DomainNativeModule::direct_native(
            "cov/pong",
            vec!["pong".into()],
        ));
    idx2.set_hybrid_source_override("cov/pong", "(val pong 2)\n");
    assert!(idx2.native().contains("cov/pong"));
    let resolved = idx2.resolve_import_detailed("cov/pong").unwrap();
    assert!(resolved.source.contains("pong"));
}

#[test]
fn native_missing_rpi_uses_registry_exports() {
    let root = scratch();
    let pkg = root.join("widgets");
    fs::create_dir_all(pkg.join("src")).unwrap();
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
    // interface-root declared but no shapes.rpi on disk — native path supplies exports.
    let mut idx = LocalPackageIndex::discover(&[&root]).unwrap();
    register_reference_module(
        &mut idx,
        "widgets/shapes",
        vec!["circle".into(), "rect".into()],
        "(val circle 1)\n(val rect 2)\n",
    );
    let resolved = idx.resolve_import_detailed("widgets/shapes").unwrap();
    assert_eq!(
        resolved.interface_exports.as_ref().unwrap(),
        &vec!["circle".to_string(), "rect".to_string()]
    );
    assert!(resolved.source.contains("rect"));
}

#[test]
fn native_with_rpi_file_reads_exports_from_disk() {
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
    fs::write(
        pkg.join("interface/shapes.rpi"),
        "(val circle)\n(val disk-only)\n",
    )
    .unwrap();
    let mut idx = LocalPackageIndex::discover(&[&root]).unwrap();
    register_reference_module(
        &mut idx,
        "widgets/shapes",
        vec!["circle".into(), "native-only".into()],
        "(val circle 1)\n",
    );
    let resolved = idx.resolve_import_detailed("widgets/shapes").unwrap();
    let exports = resolved.interface_exports.unwrap();
    assert!(exports.iter().any(|e| e == "disk-only"));
    assert!(!exports.iter().any(|e| e == "native-only"));
}

#[test]
fn n0_4_pure_native_test_module_skips_disk_package() {
    let mut idx = LocalPackageIndex::default();
    register_reference_module(
        &mut idx,
        "native/test",
        vec!["ping".into()],
        "(val ping 42)\n",
    );
    let resolved = idx.resolve_import_detailed("native/test").unwrap();
    assert_eq!(resolved.unit_name, "native/test");
    assert!(resolved.source.contains("42"));
    assert_eq!(resolved.interface_exports.as_ref().unwrap(), &vec!["ping"]);
}

#[test]
fn n1_length_units_comes_from_native_not_rpx_file() {
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    assert!(idx.native().contains("length/units"));
    let resolved = idx.resolve_import_detailed("length/units").unwrap();
    assert!(resolved.source.contains("dn2slot-length-units"));
    assert_ne!(resolved.source, length_units_source());
    // Disk .rpx may be absent after N1.2; native path must still work.
    let rpx = workspace_packages().join("length/src/units.rpx");
    if rpx.is_file() {
        let disk = std::fs::read_to_string(&rpx).unwrap();
        assert_ne!(
            disk, resolved.source,
            "native body must not be the raw disk file once synthesized"
        );
    }
}

#[test]
fn n1_pkg_length_example_elaborates_via_native() {
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_length.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    assert!(units
        .iter()
        .any(|u| u.name == "pkg_length" || u.name.contains("length")));
}

#[test]
fn length_units_module_exports_align_with_rpi() {
    let m = length_units_module();
    let rpi =
        std::fs::read_to_string(workspace_packages().join("length/interface/units.rpi")).unwrap();
    for name in &m.exports {
        assert!(
            rpi.contains(&format!("(val {name})")),
            "export `{name}` missing from .rpi"
        );
    }
}

#[test]
fn n1_color_modules_are_native() {
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    let srgb = idx.resolve_import_detailed("color/srgb").unwrap();
    assert!(srgb.source.contains("dn2slot-color-srgb"));
    let gcolor = idx.resolve_import_detailed("graphics/color").unwrap();
    assert!(gcolor.source.contains("dn2slot-graphics-color"));
    assert!(!workspace_packages().join("color/src/srgb.rpx").is_file());
    assert!(!workspace_packages()
        .join("graphics/src/color.rpx")
        .is_file());
}

#[test]
fn n2_graphics_page_is_native() {
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    let page = idx.resolve_import_detailed("graphics/page").unwrap();
    assert!(page.source.contains("dn2slot-graphics-page"));
    assert!(!workspace_packages().join("graphics/src/page.rpx").is_file());
}

#[test]
fn n2_graphics_shapes_is_native() {
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    let shapes = idx.resolve_import_detailed("graphics/shapes").unwrap();
    assert!(shapes.source.contains("dn2slot-graphics-shapes"));
    let exports = shapes.interface_exports.unwrap();
    assert!(exports.iter().any(|e| e == "circle"));
    assert!(!exports.iter().any(|e| e == "shapes-internal-tag"));
    assert!(!workspace_packages()
        .join("graphics/src/shapes.rpx")
        .is_file());
}

#[test]
fn n3_math_modules_are_native() {
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    let modules = [
        "atoms",
        "scripts",
        "frac",
        "sqrt",
        "delimiters",
        "matrix",
        "accents",
        "bigops",
        "cases",
        "align",
        "stack",
    ];
    for name in modules {
        let path = format!("math/{name}");
        assert!(idx.native().contains(&path), "missing native {path}");
        let resolved = idx.resolve_import_detailed(&path).unwrap();
        assert!(
            resolved.source.contains("dn2slot-math-"),
            "{path} source missing DN2 stub marker"
        );
        assert!(
            !workspace_packages()
                .join(format!("math/src/{name}.rpx"))
                .is_file(),
            "{path} .rpx should be retired"
        );
    }
}

#[test]
fn n4_japanese_modules_are_native() {
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    let modules = ["classes", "linebreak", "kihon", "markup"];
    for name in modules {
        let path = format!("japanese/{name}");
        assert!(idx.native().contains(&path), "missing native {path}");
        let resolved = idx.resolve_import_detailed(&path).unwrap();
        assert!(
            resolved.source.contains("dn2slot-japanese-"),
            "{path} source missing DN2 stub marker"
        );
        assert!(
            !workspace_packages()
                .join(format!("japanese/src/{name}.rpx"))
                .is_file(),
            "{path} .rpx should be retired"
        );
    }
}

#[test]
fn n5_1_document_page_is_native() {
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    assert!(idx.native().contains("document/page"));
    let page = idx.resolve_import_detailed("document/page").unwrap();
    assert!(page.source.contains("dn2slot-document-page"));
    let exports = page.interface_exports.unwrap();
    for name in [
        "page",
        "flow",
        "section",
        "heading",
        "paragraph",
        "paragraph-indented",
        "columns",
        "block-columns",
    ] {
        assert!(exports.iter().any(|e| e == name), "missing export {name}");
    }
    assert!(!workspace_packages().join("document/src/page.rpx").is_file());
}

#[test]
fn n5_1_pkg_document_example_elaborates() {
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_document.rpx");
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    assert!(units.iter().any(|u| u.name == "pkg_document"));
    assert!(units.iter().any(|u| u.name == "document/page"));
}

#[test]
fn document_page_module_exports_align_with_rpi() {
    let m = document_page_module();
    let rpi =
        std::fs::read_to_string(workspace_packages().join("document/interface/page.rpi")).unwrap();
    for name in &m.exports {
        assert!(
            rpi.contains(&format!("(val {name})")),
            "export `{name}` missing from .rpi"
        );
    }
}
