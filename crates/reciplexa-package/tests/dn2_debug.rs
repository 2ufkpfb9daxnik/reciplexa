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
