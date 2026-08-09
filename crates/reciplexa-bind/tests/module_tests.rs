//! Integration tests moved from src/module.rs for region coverage.

use reciplexa_bind::module::*;
use reciplexa_identity::package::{ModuleId, PackageInstanceId};
use reciplexa_source::resource::SourceResourceId;

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
