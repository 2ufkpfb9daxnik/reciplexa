//! Module skeleton for multi-unit programs.

use reciplexa_identity::package::{ModuleId, PackageInstanceId};
use reciplexa_source::resource::SourceResourceId;

/// A single compilable unit within a package instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleUnit {
    pub module_id: ModuleId,
    pub source_resource_id: SourceResourceId,
    pub name: String,
}

/// Minimal module graph container.
#[derive(Debug, Clone)]
pub struct ModuleSkeleton {
    pub package_instance_id: PackageInstanceId,
    pub units: Vec<ModuleUnit>,
    pub entry: Option<ModuleId>,
}

impl ModuleSkeleton {
    pub fn new(package_instance_id: PackageInstanceId) -> Self {
        Self {
            package_instance_id,
            units: Vec::new(),
            entry: None,
        }
    }

    pub fn add_unit(&mut self, unit: ModuleUnit) {
        if self.entry.is_none() {
            self.entry = Some(unit.module_id);
        }
        self.units.push(unit);
    }

    pub fn find(&self, module_id: ModuleId) -> Option<&ModuleUnit> {
        self.units.iter().find(|u| u.module_id == module_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_unit_becomes_entry() {
        let mut sk = ModuleSkeleton::new(PackageInstanceId::new(1));
        sk.add_unit(ModuleUnit {
            module_id: ModuleId::new(7),
            source_resource_id: SourceResourceId::new(1),
            name: "main".into(),
        });
        assert_eq!(sk.entry, Some(ModuleId::new(7)));
    }

    #[test]
    fn find_returns_unit_or_none() {
        let mut sk = ModuleSkeleton::new(PackageInstanceId::new(1));
        sk.add_unit(ModuleUnit {
            module_id: ModuleId::new(7),
            source_resource_id: SourceResourceId::new(1),
            name: "main".into(),
        });
        sk.add_unit(ModuleUnit {
            module_id: ModuleId::new(8),
            source_resource_id: SourceResourceId::new(2),
            name: "lib".into(),
        });
        assert_eq!(sk.entry, Some(ModuleId::new(7)));
        assert_eq!(sk.find(ModuleId::new(8)).unwrap().name, "lib");
        assert!(sk.find(ModuleId::new(99)).is_none());
    }
}
