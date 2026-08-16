use reciplexa_package::{
    diagnose_manifest, diagnose_manifest_with_root, BuildGraph, BuildNodeId, BuildTarget,
    DependencySpec, IncrementalCache, InvalidationKind, PackageManifest,
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
            source: None,
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
                source: None,
            },
            DependencySpec {
                name: "a".into(),
                version_req: "1".into(),
                path: None,
                package: None,
                source: None,
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
                source: None,
            },
            DependencySpec {
                name: "lib".into(),
                version_req: "2".into(),
                path: None,
                package: None,
                source: None,
            },
        ],
        entry: "main.rpx".into(),
        targets: vec![],
        ..Default::default()
    };
    let diags = diagnose_manifest(&m);
    assert!(diags.iter().any(|d| d.code == "PKG003"));
}

#[test]
fn diagnose_pkg004_missing_listed_resource() {
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-pkg004-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(dir.join("resources")).unwrap();
    let m = PackageManifest {
        name: "app".into(),
        version: "1".into(),
        dependencies: vec![],
        entry: "main.rpx".into(),
        targets: vec![],
        resources: vec!["images/missing.png".into()],
        ..Default::default()
    };
    // Without root: no PKG004 (filesystem-free path).
    assert!(!diagnose_manifest(&m).iter().any(|d| d.code == "PKG004"));
    let diags = diagnose_manifest_with_root(&m, &dir);
    assert!(
        diags
            .iter()
            .any(|d| d.code == "PKG004" && d.message.contains("missing.png")),
        "expected PKG004, got {diags:?}"
    );
    // Present resource → no PKG004.
    std::fs::create_dir_all(dir.join("resources/images")).unwrap();
    std::fs::write(dir.join("resources/images/logo.png"), b"png").unwrap();
    let ok = PackageManifest {
        resources: vec!["images/logo.png".into()],
        ..m
    };
    assert!(!diagnose_manifest_with_root(&ok, &dir)
        .iter()
        .any(|d| d.code == "PKG004"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn diagnose_pkg005_registry_dependency() {
    let m = PackageManifest {
        name: "app".into(),
        version: "1".into(),
        dependencies: vec![DependencySpec {
            name: "remote-kit".into(),
            version_req: "1".into(),
            path: None,
            package: Some("remote-kit".into()),
            source: Some("registry".into()),
        }],
        entry: "main.rpx".into(),
        targets: vec![],
        ..Default::default()
    };
    let diags = diagnose_manifest(&m);
    let hit = diags.iter().find(|d| d.code == "PKG005").expect("PKG005");
    assert!(hit.message.contains("remote-kit"));
    assert!(hit.message.contains("OPEN-PKG-001"));
    assert_eq!(hit.package.as_deref(), Some("app"));

    let path_dep = PackageManifest {
        dependencies: vec![DependencySpec {
            name: "local".into(),
            version_req: "1".into(),
            path: Some("../local".into()),
            package: None,
            source: Some("workspace".into()),
        }],
        ..m
    };
    assert!(!diagnose_manifest(&path_dep)
        .iter()
        .any(|d| d.code == "PKG005"));
}

#[test]
fn diagnose_pkg007_requires_lock_for_path_deps() {
    use reciplexa_package::{diagnose_required_lockfile_missing, PackageManifest};

    let m = PackageManifest {
        name: "app".into(),
        dependencies: vec![DependencySpec {
            name: "local".into(),
            version_req: "1".into(),
            path: Some("../local".into()),
            package: None,
            source: None,
        }],
        ..Default::default()
    };
    let hit = diagnose_required_lockfile_missing(&m, false)
        .into_iter()
        .find(|d| d.code == "PKG007")
        .expect("PKG007");
    assert!(hit.message.contains("rpx.lock"));
    assert_eq!(hit.package.as_deref(), Some("app"));

    assert!(diagnose_required_lockfile_missing(&m, true).is_empty());
    assert!(diagnose_required_lockfile_missing(
        &PackageManifest {
            dependencies: vec![],
            ..m.clone()
        },
        false
    )
    .is_empty());
}

#[test]
fn diagnose_pkg006_path_checksum_mismatch() {
    use reciplexa_package::{
        content_checksum, diagnose_lockfile_checksums, LockedPackage, Lockfile,
    };

    let dir = std::env::temp_dir().join(format!(
        "reciplexa-pkg006-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let dep = dir.join("util");
    std::fs::create_dir_all(&dep).unwrap();
    let rpxm = dep.join("package.rpxm");
    std::fs::write(&rpxm, "(package util format-version 1 version \"1\")\n").unwrap();
    let actual = content_checksum(&rpxm);

    let matching = Lockfile {
        packages: vec![LockedPackage {
            name: "util".into(),
            version: "1".into(),
            source: "path:util".into(),
            dependencies: vec![],
            checksum: Some(actual.clone()),
        }],
    };
    assert!(
        diagnose_lockfile_checksums(&matching, &dir, None).is_empty(),
        "matching checksum should be quiet"
    );

    let mismatch = Lockfile {
        packages: vec![LockedPackage {
            name: "util".into(),
            version: "1".into(),
            source: "path:util".into(),
            dependencies: vec![],
            checksum: Some(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
            ),
        }],
    };
    let diags = diagnose_lockfile_checksums(&mismatch, &dir, None);
    let hit = diags.iter().find(|d| d.code == "PKG006").expect("PKG006");
    assert!(hit.message.contains("util"));
    assert!(hit.message.contains(&actual));

    // No checksum → no PKG006.
    let bare = Lockfile {
        packages: vec![LockedPackage {
            name: "util".into(),
            version: "1".into(),
            source: "path:util".into(),
            dependencies: vec![],
            checksum: None,
        }],
    };
    assert!(!diagnose_lockfile_checksums(&bare, &dir, None)
        .iter()
        .any(|d| d.code == "PKG006"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn diagnose_lockfile_registry_source_pkg005() {
    use reciplexa_package::{
        diagnose_lockfile_registry_sources, LockedPackage, Lockfile, OPEN_PKG_001_CODE,
    };

    let lock = Lockfile {
        packages: vec![LockedPackage {
            name: "remote".into(),
            version: "1".into(),
            source: "registry:remote-kit@1.0.0".into(),
            dependencies: vec![],
            checksum: None,
        }],
    };
    let diags = diagnose_lockfile_registry_sources(&lock, None);
    let hit = diags.iter().find(|d| d.code == "PKG005").expect("PKG005");
    assert!(hit.message.contains("remote"));
    assert!(hit.message.contains(OPEN_PKG_001_CODE));

    let ok = Lockfile {
        packages: vec![LockedPackage {
            name: "local".into(),
            version: "1".into(),
            source: "path:lib".into(),
            dependencies: vec![],
            checksum: None,
        }],
    };
    assert!(diagnose_lockfile_registry_sources(&ok, None).is_empty());
}

#[test]
fn diagnose_lockfile_registry_allows_local_mirror() {
    use reciplexa_package::{
        content_checksum, diagnose_lockfile_checksums, diagnose_lockfile_registry_sources,
        LocalRegistryMirror, LockedPackage, Lockfile,
    };

    let dir = std::env::temp_dir().join(format!(
        "reciplexa-reg-mirror-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let pkg = dir.join("registry/remote-kit/1.0.0");
    std::fs::create_dir_all(&pkg).unwrap();
    let rpxm = pkg.join("package.rpxm");
    std::fs::write(
        &rpxm,
        "(package remote-kit format-version 1 version \"1.0.0\")\n",
    )
    .unwrap();
    let mirror = LocalRegistryMirror::new(dir.join("registry"));
    let lock = Lockfile {
        packages: vec![LockedPackage {
            name: "remote-kit".into(),
            version: "1.0.0".into(),
            source: LocalRegistryMirror::lock_source("remote-kit", "1.0.0"),
            dependencies: vec![],
            checksum: Some(content_checksum(&rpxm)),
        }],
    };
    assert!(diagnose_lockfile_registry_sources(&lock, Some(&mirror)).is_empty());
    assert!(diagnose_lockfile_checksums(&lock, &dir, Some(&mirror)).is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
