//! Domain native package registry (KER-001 / Compiler-native package).
//!
//! Hot std packages (`graphics`, `length`, `color`, `math`, `japanese`, …) keep
//! their public import paths and `.rpi` surfaces, but their bodies are Rust.
//! Loaders consult this registry **before** reading `packages/*/src/*.rpx`.

use std::collections::BTreeMap;

use reciplexa_core::ty::CoreType;
use reciplexa_eval::domain_native::{dn2_slot, qualified_export_key, DomainNativeOp};
use reciplexa_identity::binding::{BindingId, BindingIdAllocator};

/// Hybrid v1 uses synthesized RPX; Direct Native v2 uses typed Rust callables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DomainNativeMode {
    #[default]
    Hybrid,
    DirectNative,
}

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
    /// Hybrid v1: RPX-shaped body synthesized by Rust.
    pub synthetic_source: String,
    /// Per-module path selection between Hybrid and Direct Native v2.
    pub mode: DomainNativeMode,
    /// Direct Native v2 typed exports (`export name` → callable + type).
    pub typed_exports: BTreeMap<String, DomainNativeExport>,
}

impl DomainNativeModule {
    /// Hybrid Native v1 module (synthetic RPX body).
    pub fn hybrid(module_path: &str, exports: Vec<String>, synthetic_source: &str) -> Self {
        Self {
            module_path: module_path.to_string(),
            exports,
            synthetic_source: synthetic_source.to_string(),
            mode: DomainNativeMode::Hybrid,
            typed_exports: BTreeMap::new(),
        }
    }

    /// Source text used by loaders and elaboration for this module.
    pub fn effective_source(&self) -> String {
        match self.mode {
            DomainNativeMode::Hybrid => self.synthetic_source.clone(),
            DomainNativeMode::DirectNative => self.dn2_stub_source(),
        }
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

    pub fn is_direct_native(&self) -> bool {
        self.mode == DomainNativeMode::DirectNative
    }
}

/// Nullary exports are applied eagerly in the stub (constants); others bind the callable.
fn export_is_eager_value(ty: &CoreType) -> bool {
    matches!(ty, CoreType::Fun { args, .. } if args.is_empty())
}

/// Registry of module paths implemented in Rust instead of portable `.rpx`.
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
            .get(module_path)?
            .typed_exports
            .get(export_name)
    }

    /// Set one module to Direct Native v2 (others unchanged).
    pub fn set_mode(&mut self, module_path: &str, mode: DomainNativeMode) -> bool {
        if let Some(module) = self.modules.get_mut(module_path) {
            module.mode = mode;
            true
        } else {
            false
        }
    }

    /// Clone registry with selected modules switched to Direct Native v2.
    pub fn with_direct_native_modules(&self, module_paths: &[&str]) -> Self {
        let mut out = self.clone();
        for path in module_paths {
            out.set_mode(path, DomainNativeMode::DirectNative);
        }
        out
    }
}

/// Maps `package/module/export` qualified keys to compilation [`BindingId`] values.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DomainNativeBindMap {
    qualified_to_binding: BTreeMap<String, BindingId>,
    binding_to_qualified: BTreeMap<BindingId, String>,
}

impl DomainNativeBindMap {
    pub fn bind(
        &mut self,
        alloc: &mut BindingIdAllocator,
        module_path: &str,
        export_name: &str,
    ) -> BindingId {
        let qualified = qualified_export_key(module_path, export_name);
        if let Some(id) = self.qualified_to_binding.get(&qualified) {
            return *id;
        }
        let id = alloc.allocate();
        self.qualified_to_binding.insert(qualified.clone(), id);
        self.binding_to_qualified.insert(id, qualified);
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

    pub fn populate_from_registry(
        &mut self,
        alloc: &mut BindingIdAllocator,
        registry: &DomainNativeRegistry,
    ) {
        for module in registry.modules.values() {
            if module.mode != DomainNativeMode::DirectNative {
                continue;
            }
            for export_name in module.typed_exports.keys() {
                self.bind(alloc, &module.module_path, export_name);
            }
        }
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
        reg.register(DomainNativeModule::hybrid(
            "native/test",
            vec!["ping".into()],
            "(val ping 1)\n",
        ));
        assert_eq!(reg.len(), 1);
        assert!(reg.contains("native/test"));
        let m = reg.get("native/test").expect("registered");
        assert_eq!(m.exports, vec!["ping"]);
        assert!(m.synthetic_source.contains("ping"));
        assert!(!reg.contains("length/units"));
        assert_eq!(reg.paths().collect::<Vec<_>>(), vec!["native/test"]);
    }

    #[test]
    fn dn2_stub_source_aliases_slots() {
        let mut module = DomainNativeModule::hybrid("native/test", vec!["ping".into()], "");
        module.mode = DomainNativeMode::DirectNative;
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
        let mut module = DomainNativeModule::hybrid("native/test", vec!["ping".into()], "");
        module.mode = DomainNativeMode::DirectNative;
        module.typed_exports.insert(
            "ping".into(),
            DomainNativeExport::new("ping", CoreType::Int, DomainNativeOp::TestPing),
        );
        reg.register(module);
        let mut alloc = BindingIdAllocator::new();
        let mut map = DomainNativeBindMap::default();
        map.populate_from_registry(&mut alloc, &reg);
        let id = map.binding_for("native/test", "ping").expect("bound");
        assert!(id.is_valid());
        assert_eq!(map.qualified_for(id), Some("native/test/ping"));
    }
}
