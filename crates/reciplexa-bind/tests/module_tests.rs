//! Integration tests moved from src/module.rs for region coverage.

use reciplexa_bind::module::*;
use reciplexa_core::expr::CoreExpr;
use reciplexa_eval::{eval_expr, RuntimeValue, UnitHost};
use reciplexa_identity::package::{ModuleId, PackageInstanceId};
use reciplexa_source::resource::SourceResourceId;
use std::collections::HashMap;

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

#[test]
fn elaborate_units_links_import_only() {
    let units = elaborate_units(&[
        ("lib", r#"(val id (fn (x) x)) (val unused 0)"#),
        ("main", r#"(import lib only (id)) (val main (id 7))"#),
    ])
    .unwrap();
    let main = units.iter().find(|u| u.name == "main").unwrap();
    assert_eq!(
        main.imports,
        vec![ImportDecl {
            module: "lib".into(),
            only: Some(vec!["id".into()]),
        }]
    );
    assert!(matches!(main.expr, CoreExpr::Let { ref name, .. } if name == "id"));
    let v = eval_expr(&main.expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(7.0));
}

#[test]
fn elaborate_units_rejects_unknown_import() {
    let err = elaborate_units(&[("main", "(import missing) (val main 1)")]).unwrap_err();
    assert!(err.message.contains("unknown"));
}

#[test]
fn load_module_tree_reads_sibling_imports() {
    let dir = std::env::temp_dir().join(format!("reciplexa-mod-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("lib.rpx"), "(val id (fn (x) x))\n").unwrap();
    std::fs::write(
        dir.join("main.rpx"),
        "(import lib only (id))\n(val main (id 7))\n",
    )
    .unwrap();

    let units = load_module_tree(dir.join("main.rpx")).unwrap();
    assert!(units.iter().any(|(n, _)| n == "main"));
    assert!(units.iter().any(|(n, _)| n == "lib"));

    let elaborated = elaborate_module_tree(dir.join("main.rpx")).unwrap();
    let main = elaborated.iter().find(|u| u.name == "main").unwrap();
    let v = eval_expr(&main.expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(7.0));

    let dir_units = load_module_tree(&dir).unwrap();
    assert_eq!(dir_units.len(), 2);
    let _ = std::fs::remove_dir_all(&dir);
}
