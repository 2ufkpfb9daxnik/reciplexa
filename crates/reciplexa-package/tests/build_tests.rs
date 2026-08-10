use reciplexa_package::{
    diagnose_manifest, BuildGraph, BuildNodeId, BuildTarget, DependencySpec, IncrementalCache,
    InvalidationKind, PackageManifest,
};

#[test]
fn contract_invalidation_wider_than_impl() {
    let m = PackageManifest {
        name: "app".into(),
        version: "1".into(),
        dependencies: vec![DependencySpec {
            name: "lib".into(),
            version_req: "1".into(),
            path: None,
            package: None,
        }],
        entry: "main.rpx".into(),
        targets: vec!["document".into()],
        ..Default::default()
    };
    let g = BuildGraph::from_manifest(&m, BuildTarget::Document);
    let lib = BuildNodeId("lib:document".into());
    let impl_only = g.invalidate(&lib, InvalidationKind::Implementation);
    let contract = g.invalidate(&lib, InvalidationKind::Contract);
    assert_eq!(impl_only.len(), 1);
    assert!(contract.len() > impl_only.len());
}

#[test]
fn topo_order_is_linear_for_chain() {
    let m = PackageManifest {
        name: "app".into(),
        version: "1".into(),
        dependencies: vec![
            DependencySpec {
                name: "b".into(),
                version_req: "1".into(),
                path: None,
                package: None,
            },
            DependencySpec {
                name: "a".into(),
                version_req: "1".into(),
                path: None,
                package: None,
            },
        ],
        entry: "main.rpx".into(),
        targets: vec![],
        ..Default::default()
    };
    let g = BuildGraph::from_manifest(&m, BuildTarget::Document);
    let order = g.topo_order();
    assert_eq!(order.len(), 3);
    let a_pos = order.iter().position(|n| n.0 == "a:document").unwrap();
    let b_pos = order.iter().position(|n| n.0 == "b:document").unwrap();
    let app_pos = order.iter().position(|n| n.0 == "app:document").unwrap();
    assert!(a_pos < app_pos);
    assert!(b_pos < app_pos);
}

#[test]
fn needs_rebuild_tracks_hash_changes() {
    let id = BuildNodeId("node:document".into());
    let mut cache = IncrementalCache::default();
    assert!(cache.needs_rebuild(&id, "abc"));
    cache.put(id.clone(), "abc");
    assert!(!cache.needs_rebuild(&id, "abc"));
    assert!(cache.needs_rebuild(&id, "def"));
}

#[test]
fn invalidate_set_removes_cached_hashes() {
    let id = BuildNodeId("node:document".into());
    let other = BuildNodeId("other:document".into());
    let mut cache = IncrementalCache::default();
    cache.put(id.clone(), "h1");
    cache.put(other.clone(), "h2");
    let mut ids = std::collections::BTreeSet::new();
    ids.insert(id.clone());
    cache.invalidate_set(&ids);
    assert!(cache.needs_rebuild(&id, "h1"));
    assert!(!cache.needs_rebuild(&other, "h2"));
}

#[test]
fn diagnose_pkg001_empty_name() {
    let m = PackageManifest {
        name: "".into(),
        version: "1".into(),
        dependencies: vec![],
        entry: "main.rpx".into(),
        targets: vec![],
        ..Default::default()
    };
    let diags = diagnose_manifest(&m);
    assert!(diags.iter().any(|d| d.code == "PKG001"));
}

#[test]
fn diagnose_pkg002_empty_entry() {
    let m = PackageManifest {
        name: "app".into(),
        version: "1".into(),
        dependencies: vec![],
        entry: "".into(),
        targets: vec![],
        ..Default::default()
    };
    let diags = diagnose_manifest(&m);
    assert!(diags
        .iter()
        .any(|d| d.code == "PKG002" && d.package == Some("app".into())));
}

#[test]
fn diagnose_pkg003_duplicate_dependency() {
    let m = PackageManifest {
        name: "app".into(),
        version: "1".into(),
        dependencies: vec![
            DependencySpec {
                name: "lib".into(),
                version_req: "1".into(),
                path: None,
                package: None,
            },
            DependencySpec {
                name: "lib".into(),
                version_req: "2".into(),
                path: None,
                package: None,
            },
        ],
        entry: "main.rpx".into(),
        targets: vec![],
        ..Default::default()
    };
    let diags = diagnose_manifest(&m);
    assert!(diags.iter().any(|d| d.code == "PKG003"));
}
