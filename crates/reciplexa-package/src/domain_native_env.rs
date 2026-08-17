//! Build eval / typecheck environments for Direct Native v2 exports.

use std::collections::HashMap;

use reciplexa_core::ty::CoreType;
use reciplexa_eval::domain_native::dn2_slot;
use reciplexa_eval::{DomainNativeOp, RuntimeValue};

use crate::domain_native::{DomainNativeExport, DomainNativeModule, DomainNativeRegistry};

/// Hybrid v1 reference body for the differential `native/test` ping module.
pub const TEST_PING_REFERENCE_SOURCE: &str = "(val ping 42)\n";

/// Runtime bindings for DN2 internal slots (`__dn2__…` identifiers).
pub fn build_domain_native_eval_env(
    registry: &DomainNativeRegistry,
) -> HashMap<String, RuntimeValue> {
    let mut env = HashMap::new();
    for module in registry.modules() {
        for export in module.typed_exports.values() {
            let slot = dn2_slot(&module.module_path, &export.name);
            env.insert(slot, RuntimeValue::DomainNative(export.op));
        }
    }
    env
}

/// Declared types for DN2 internal slots.
pub fn build_domain_native_type_env(registry: &DomainNativeRegistry) -> HashMap<String, CoreType> {
    let mut env = HashMap::new();
    for module in registry.modules() {
        for export in module.typed_exports.values() {
            let slot = dn2_slot(&module.module_path, &export.name);
            env.insert(slot, export.ty.clone());
        }
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
