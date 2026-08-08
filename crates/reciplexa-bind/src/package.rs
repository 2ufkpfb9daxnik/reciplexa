//! Multi-unit package resolution (Phase 1 module skeleton).

use std::collections::HashMap;

use reciplexa_identity::package::{ModuleId, PackageInstanceId};
use reciplexa_source::resource::SourceResourceId;

use crate::module::{ModuleSkeleton, ModuleUnit};
use crate::resolve::{resolve_source, ResolveResult};

/// Resolved package with per-module binding results.
#[derive(Debug, Clone)]
pub struct PackageResolveResult {
    pub skeleton: ModuleSkeleton,
    pub modules: HashMap<ModuleId, ResolveResult>,
}

/// Build a module skeleton from named source units and resolve each.
pub fn resolve_package(
    package_id: PackageInstanceId,
    units: &[(String, &str)],
) -> PackageResolveResult {
    let mut skeleton = ModuleSkeleton::new(package_id);
    let mut modules = HashMap::new();
    for (i, (name, source)) in units.iter().enumerate() {
        let module_id = ModuleId::new((i + 1) as u64);
        let resource_id = SourceResourceId::new((i + 1) as u64);
        skeleton.add_unit(ModuleUnit {
            module_id,
            source_resource_id: resource_id,
            name: name.clone(),
        });
        modules.insert(module_id, resolve_source(source));
    }
    PackageResolveResult { skeleton, modules }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_multiple_units() {
        let units: [(&str, &str); 2] = [
            ("main", "(page a4 (rect 1 2 3 4))"),
            ("lib", "(page a4 (circle 1 2 3))"),
        ];
        let refs: Vec<(String, &str)> = units.iter().map(|(n, s)| (n.to_string(), *s)).collect();
        let pkg = resolve_package(PackageInstanceId::new(1), &refs);
        assert_eq!(pkg.skeleton.units.len(), 2);
        assert!(pkg.modules.values().all(|m| m.is_ok()));
    }
}
