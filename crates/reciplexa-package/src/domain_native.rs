//! Domain native package registry (KER-001 / Compiler-native package).
//!
//! Hot std packages (`graphics`, `length`, `color`, `math`, `japanese`, …) keep
//! their public import paths and `.rpi` surfaces, but their bodies are Rust.
//! Loaders consult this registry **before** reading `packages/*/src/*.rpx`.

use std::collections::BTreeMap;

/// One native-backed module addressable as `package/module` (e.g. `length/units`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainNativeModule {
    /// Import path without leading slash: `graphics/shapes`.
    pub module_path: String,
    /// Public binding names (mirrors `.rpi` exports).
    pub exports: Vec<String>,
    /// RPX-shaped body synthesized by Rust (portable text until eval binds constructors).
    pub synthetic_source: String,
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

    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.modules.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        reg.register(DomainNativeModule {
            module_path: "native/test".into(),
            exports: vec!["ping".into()],
            synthetic_source: "(val ping 1)\n".into(),
        });
        assert_eq!(reg.len(), 1);
        assert!(reg.contains("native/test"));
        let m = reg.get("native/test").expect("registered");
        assert_eq!(m.exports, vec!["ping"]);
        assert!(m.synthetic_source.contains("ping"));
        assert!(!reg.contains("length/units"));
        assert_eq!(reg.paths().collect::<Vec<_>>(), vec!["native/test"]);
    }
}
