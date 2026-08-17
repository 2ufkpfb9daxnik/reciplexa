use std::path::PathBuf;

use reciplexa_core::elaborate::elaborate_source;
use reciplexa_core::expr::CoreExpr;
use reciplexa_package::{
    differential_eval_package_modules, register_test_ping_module, DomainNativeMode,
    DomainNativeRegistry, LocalPackageIndex,
};

#[test]
fn dn2_stub_elaborates_to_var_ref() {
    let mut reg = DomainNativeRegistry::empty();
    register_test_ping_module(&mut reg);
    reg.set_mode("native/test", DomainNativeMode::DirectNative);
    let stub = reg.get("native/test").unwrap().effective_source();
    let expr = elaborate_source(&stub).expect("stub elaborates");
    let ping = collect_bindings(&expr).get("ping").cloned().expect("ping");
    assert!(!matches!(ping, CoreExpr::Error), "ping={ping:?}");
}

#[test]
fn dn2_single_unit_via_elaborate_units() {
    let mut reg = DomainNativeRegistry::empty();
    register_test_ping_module(&mut reg);
    reg.set_mode("native/test", DomainNativeMode::DirectNative);
    let stub = reg.get("native/test").unwrap().effective_source();
    let units = reciplexa_bind::elaborate_units(&[("native/test", stub.as_str())]).unwrap();
    let ping = collect_bindings(&units[0].expr)
        .get("ping")
        .cloned()
        .expect("ping");
    assert!(
        !matches!(ping, CoreExpr::Error),
        "stub={stub:?} ping={ping:?}"
    );
}

#[test]
fn dn2_test_ping_module_hybrid_and_direct_native_agree() {
    let mut reg = DomainNativeRegistry::empty();
    register_test_ping_module(&mut reg);
    let idx = LocalPackageIndex::default().with_native(reg);
    let src = "(import native/test only ping)\n(val main ping)\n";
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-dn2-ping-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let entry = dir.join("main.rpx");
    std::fs::write(&entry, src).unwrap();
    differential_eval_package_modules(&entry, &idx, &["native/test"])
        .expect("hybrid and DN2 agree");
}

#[test]
fn dn2_length_units_hybrid_and_direct_native_agree_on_pkg_length() {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_length.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    differential_eval_package_modules(&entry, &idx, &["length/units"])
        .expect("length/units hybrid and DN2 agree on pkg_length");
}

#[test]
fn dn2_length_units_module_has_typed_exports_for_all_rpi_names() {
    use reciplexa_package::length_units_module;
    let m = length_units_module();
    assert_eq!(m.typed_exports.len(), m.exports.len());
    for name in &m.exports {
        assert!(
            m.typed_exports.contains_key(name),
            "missing typed export `{name}`"
        );
    }
}

#[test]
fn dn2_color_srgb_hybrid_and_direct_native_agree_on_pkg_color() {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_color.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    differential_eval_package_modules(&entry, &idx, &["color/srgb"])
        .expect("color/srgb hybrid and DN2 agree on pkg_color");
}

#[test]
fn dn2_graphics_shapes_hybrid_and_direct_native_agree_on_pkg_circle() {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_circle.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    differential_eval_package_modules(&entry, &idx, &["graphics/shapes"])
        .expect("graphics/shapes hybrid and DN2 agree on pkg_circle");
}

#[test]
fn dn2_graphics_modules_hybrid_and_direct_native_agree_on_pkg_graphics_static() {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_graphics_static.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    differential_eval_package_modules(
        &entry,
        &idx,
        &["graphics/color", "graphics/page", "graphics/shapes"],
    )
    .expect("graphics modules hybrid and DN2 agree on pkg_graphics_static");
}

#[test]
fn dn2_pure_constructor_modules_have_typed_exports_for_all_rpi_names() {
    use reciplexa_package::{
        color_srgb_module, graphics_color_module, graphics_page_module, graphics_shapes_module,
    };
    for m in [
        color_srgb_module(),
        graphics_color_module(),
        graphics_page_module(),
        graphics_shapes_module(),
    ] {
        assert_eq!(
            m.typed_exports.len(),
            m.exports.len(),
            "module {}",
            m.module_path
        );
        for name in &m.exports {
            assert!(
                m.typed_exports.contains_key(name),
                "module {} missing typed export `{name}`",
                m.module_path
            );
        }
    }
}

fn collect_bindings(expr: &CoreExpr) -> std::collections::HashMap<String, CoreExpr> {
    let mut map = std::collections::HashMap::new();
    fn walk(expr: &CoreExpr, map: &mut std::collections::HashMap<String, CoreExpr>) {
        match expr {
            CoreExpr::Let { name, value, body } => {
                map.insert(name.clone(), *value.clone());
                walk(body, map);
            }
            CoreExpr::LetRec { bindings, body } => {
                for (name, value) in bindings {
                    map.insert(name.clone(), value.clone());
                }
                walk(body, map);
            }
            _ => {}
        }
    }
    walk(expr, &mut map);
    map
}
