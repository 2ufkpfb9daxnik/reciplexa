//! Domain native package registry (KER-001 / Compiler-native package).
//!
//! Hot std packages (`graphics`, `length`, `color`, `math`, `japanese`, …) keep
//! their public import paths and `.rpi` surfaces, but their bodies are Rust.
//! Loaders consult this registry **before** reading `packages/*/src/*.rpx`.

use std::collections::BTreeMap;

use reciplexa_core::expr::CoreExpr;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::domain_native::{dn2_slot, qualified_export_key, DomainNativeOp};
use reciplexa_identity::binding::{BindingId, BindingIdAllocator};

/// One typed export on a Direct Native module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainNativeExport {
    pub name: String,
    pub ty: CoreType,
    pub op: DomainNativeOp,
}

impl DomainNativeExport {
    pub fn new(name: impl Into<String>, ty: CoreType, op: DomainNativeOp) -> Self {
        Self {
            name: name.into(),
            ty,
            op,
        }
    }

    pub fn qualified_key(&self, module_path: &str) -> String {
        qualified_export_key(module_path, &self.name)
    }
}

/// One native-backed module addressable as `package/module` (e.g. `length/units`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainNativeModule {
    /// Import path without leading slash: `graphics/shapes`.
    pub module_path: String,
    /// Public binding names (mirrors `.rpi` exports).
    pub exports: Vec<String>,
    /// Direct Native v2 typed exports (`export name` → callable + type).
    pub typed_exports: BTreeMap<String, DomainNativeExport>,
}

impl DomainNativeModule {
    /// Direct Native v2 module (stub RPX + typed Rust callables).
    pub fn direct_native(module_path: &str, exports: Vec<String>) -> Self {
        Self {
            module_path: module_path.to_string(),
            exports,
            typed_exports: BTreeMap::new(),
        }
    }

    /// Source text used by loaders for Hybrid overrides and debug dumps.
    ///
    /// Production elaboration uses [`Self::stub_core_expr`] and does not parse this.
    pub fn effective_source(&self) -> String {
        self.dn2_stub_source()
    }

    /// Core Let spine equivalent to [`Self::dn2_stub_source`], without RPX parse.
    pub fn stub_core_expr(&self) -> CoreExpr {
        let bindings: Vec<(String, CoreExpr)> = self
            .typed_exports
            .values()
            .map(|export| {
                let slot = CoreExpr::Var(dn2_slot(&self.module_path, &export.name));
                let value = if export_is_eager_value(&export.ty) {
                    CoreExpr::App {
                        fun: Box::new(slot),
                        args: vec![],
                    }
                } else {
                    slot
                };
                (export.name.clone(), value)
            })
            .collect();
        let body = bindings
            .last()
            .map(|(name, _)| CoreExpr::Var(name.clone()))
            .unwrap_or(CoreExpr::Seq(vec![]));
        bindings
            .into_iter()
            .rev()
            .fold(body, |body, (name, value)| CoreExpr::Let {
                name,
                value: Box::new(value),
                body: Box::new(body),
            })
    }

    /// Minimal RPX stub that aliases public exports to internal DN2 eval slots.
    pub fn dn2_stub_source(&self) -> String {
        let lines: Vec<String> = self
            .typed_exports
            .values()
            .map(|export| {
                let slot = dn2_slot(&self.module_path, &export.name);
                if export_is_eager_value(&export.ty) {
                    format!("(val {} ({slot}))", export.name)
                } else {
                    format!("(val {} {slot})", export.name)
                }
            })
            .collect();
        format!("{}\n", lines.join("\n"))
    }
}

/// Nullary exports are applied eagerly in the stub (constants); others bind the callable.
fn export_is_eager_value(ty: &CoreType) -> bool {
    matches!(ty, CoreType::Fun { args, .. } if args.is_empty())
}

/// Registry of module paths implemented in Rust instead of portable `.rpx`.
///
/// Production load synthesizes [`DomainNativeModule::stub_core_expr`] (no stub
/// RPX elaboration). Hybrid v1 reference RPX is applied only via
/// [`crate::load::LocalPackageIndex`] differential overrides, not this registry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DomainNativeRegistry {
    modules: BTreeMap<String, DomainNativeModule>,
}

impl DomainNativeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Empty registry — all packages still load interim `.rpx` bodies.
    pub fn empty() -> Self {
        Self::new()
    }

    pub fn register(&mut self, module: DomainNativeModule) {
        self.modules.insert(module.module_path.clone(), module);
    }

    pub fn contains(&self, module_path: &str) -> bool {
        self.modules.contains_key(module_path)
    }

    pub fn get(&self, module_path: &str) -> Option<&DomainNativeModule> {
        self.modules.get(module_path)
    }

    pub fn get_mut(&mut self, module_path: &str) -> Option<&mut DomainNativeModule> {
        self.modules.get_mut(module_path)
    }

    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.modules.keys().map(String::as_str)
    }

    pub fn modules(&self) -> impl Iterator<Item = &DomainNativeModule> {
        self.modules.values()
    }

    /// Lookup a typed export by `package/module/export`.
    pub fn export(&self, module_path: &str, export_name: &str) -> Option<&DomainNativeExport> {
        self.modules
            .get(module_path)
            .and_then(|m| m.typed_exports.get(export_name))
    }
}

/// Maps this compilation's [`BindingId`] values to `package/module/export` callables.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DomainNativeBindMap {
    qualified_to_binding: BTreeMap<String, BindingId>,
    binding_to_qualified: BTreeMap<BindingId, String>,
    binding_to_op: BTreeMap<BindingId, DomainNativeOp>,
}

impl DomainNativeBindMap {
    pub fn bind(
        &mut self,
        alloc: &mut BindingIdAllocator,
        module_path: &str,
        export_name: &str,
        op: DomainNativeOp,
    ) -> BindingId {
        let qualified = qualified_export_key(module_path, export_name);
        if let Some(id) = self.qualified_to_binding.get(&qualified) {
            return *id;
        }
        let id = alloc.allocate();
        self.qualified_to_binding.insert(qualified.clone(), id);
        self.binding_to_qualified.insert(id, qualified);
        self.binding_to_op.insert(id, op);
        id
    }

    pub fn binding_for(&self, module_path: &str, export_name: &str) -> Option<BindingId> {
        self.qualified_to_binding
            .get(&qualified_export_key(module_path, export_name))
            .copied()
    }

    pub fn qualified_for(&self, id: BindingId) -> Option<&str> {
        self.binding_to_qualified.get(&id).map(String::as_str)
    }

    /// Typed callable for a compilation BindingId assigned from a DN2 stub.
    pub fn op_for(&self, id: BindingId) -> Option<DomainNativeOp> {
        self.binding_to_op.get(&id).copied()
    }

    /// Bind every typed export to a compilation [`BindingId`].
    ///
    /// Registry key remains `package/module/export`. [`BindingId`] values are
    /// allocated for this compilation from the typed export table; they are not
    /// reused as a persistent ABI identity. Stub RPX is not resolved.
    pub fn bind_stubs(registry: &DomainNativeRegistry) -> Result<Self, String> {
        let mut map = Self::default();
        let mut alloc = BindingIdAllocator::new();
        for module in registry.modules.values() {
            for (name, export) in &module.typed_exports {
                map.bind(&mut alloc, &module.module_path, name, export.op);
            }
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_core::ty::CoreType;

    #[test]
    fn empty_registry_has_no_native_modules() {
        let reg = DomainNativeRegistry::empty();
        assert!(reg.is_empty());
        assert!(!reg.contains("length/units"));
        assert!(reg.get("length/units").is_none());
    }

    #[test]
    fn register_and_lookup_test_module() {
        let mut reg = DomainNativeRegistry::new();
        reg.register(DomainNativeModule::direct_native(
            "native/test",
            vec!["ping".into()],
        ));
        assert_eq!(reg.len(), 1);
        assert!(reg.contains("native/test"));
        let m = reg.get("native/test").expect("registered");
        assert_eq!(m.exports, vec!["ping"]);
        assert!(m.effective_source().is_empty() || m.effective_source() == "\n");
        assert!(!reg.contains("length/units"));
        assert_eq!(reg.paths().collect::<Vec<_>>(), vec!["native/test"]);
    }

    #[test]
    fn dn2_stub_source_aliases_slots() {
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
        let stub = module.dn2_stub_source();
        assert!(stub.contains("(val ping (dn2slot-native-test-ping))"));
    }

    #[test]
    fn bind_map_tracks_qualified_exports() {
        let mut reg = DomainNativeRegistry::new();
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
        reg.register(module);
        let map = DomainNativeBindMap::bind_stubs(&reg).expect("stub bind");
        let id = map.binding_for("native/test", "ping").expect("bound");
        assert!(id.is_valid());
        assert_eq!(map.qualified_for(id), Some("native/test/ping"));
        assert_eq!(map.op_for(id), Some(DomainNativeOp::TestPing));
    }
}
