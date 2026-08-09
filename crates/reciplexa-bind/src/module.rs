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
