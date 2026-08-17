//! Build eval / typecheck environments for Direct Native v2 exports.

use std::collections::HashMap;

use reciplexa_core::ty::CoreType;
use reciplexa_eval::{DomainNativeOp, RuntimeValue};

use crate::domain_native::{
    DomainNativeBindMap, DomainNativeExport, DomainNativeModule, DomainNativeRegistry,
};

/// Hybrid v1 reference body for the differential `native/test` ping module.
pub const TEST_PING_REFERENCE_SOURCE: &str = "(val ping 42)\n";

/// Runtime bindings for compilation [`reciplexa_identity::binding::BindingId`] slots.
///
/// Keys are [`DomainNativeBindMap::slot_key`]; values come from
/// [`DomainNativeBindMap::op_for`]. Debug stub RPX still names `dn2slot-*`.
pub fn build_domain_native_eval_env(map: &DomainNativeBindMap) -> HashMap<String, RuntimeValue> {
    let mut env = HashMap::new();
    for (id, _) in map.iter_ops() {
        let Some(op) = map.op_for(id) else {
            continue;
        };
        env.insert(
            DomainNativeBindMap::slot_key(id),
            RuntimeValue::DomainNative(op),
        );
    }
    env
}

/// Declared types for compilation BindingId slots.
pub fn build_domain_native_type_env(
    map: &DomainNativeBindMap,
    registry: &DomainNativeRegistry,
) -> HashMap<String, CoreType> {
    let mut env = HashMap::new();
    for (id, _) in map.iter_ops() {
        let Some((module_path, export_name)) = map.site_for(id) else {
            continue;
        };
        let Some(export) = registry.export(module_path, export_name) else {
            continue;
        };
        env.insert(DomainNativeBindMap::slot_key(id), export.ty.clone());
    }
    env
}

/// Test helper: register a minimal DN2 module for differential harness smoke tests.
pub fn register_test_ping_module(registry: &mut DomainNativeRegistry) {
    let mut module = DomainNativeModule::direct_native("native/test", vec!["ping".into()]);
    module.typed_exports.insert(
        "ping".into(),
        DomainNativeExport::new(
            "ping",
            CoreType::Fun {
                args: vec![],
                ret: Box::new(CoreType::Int),
                effects: reciplexa_core::ty::EffectRow::default(),
            },
            DomainNativeOp::TestPing,
        ),
    );
    registry.register(module);
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_eval::RuntimeValue;

    #[test]
    fn eval_env_keys_binding_ids_via_op_for() {
        let mut registry = DomainNativeRegistry::empty();
        register_test_ping_module(&mut registry);
        let map = DomainNativeBindMap::bind_stubs(&registry).expect("stub bind");
        let env = build_domain_native_eval_env(&map);
        let id = map.binding_for("native/test", "ping").expect("bound");
        let op = map.op_for(id).expect("op_for");
        assert_eq!(op, DomainNativeOp::TestPing);
        assert_eq!(
            env.get(&DomainNativeBindMap::slot_key(id)),
            Some(&RuntimeValue::DomainNative(op))
        );
        assert!(env.keys().all(|k| k.starts_with("dn2bid-")));
        assert!(!env.keys().any(|k| k.contains("dn2slot-")));

        let types = build_domain_native_type_env(&map, &registry);
        assert!(types.contains_key(&DomainNativeBindMap::slot_key(id)));
    }
}
