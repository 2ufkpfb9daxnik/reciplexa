//! Phase 10 conformance: package resolution, rpxm, build graph, cache.

use std::collections::BTreeMap;

use reciplexa_package::{
    diagnose_manifest, parse_rpxm, resolve_packages, BuildGraph, BuildNodeId, BuildTarget,
    IncrementalCache, InvalidationKind, Lockfile, PackageManifest, RpxmError, RuntimeProfile,
};

#[test]
fn phase10_deterministic_lockfile() {
    let m = PackageManifest {
        name: "app".into(),
        version: "0.1.0".into(),
        dependencies: vec![],
        entry: "main".into(),
        targets: vec!["document".into()],
        ..Default::default()
    };
    let mut sources = BTreeMap::new();
    sources.insert("app".into(), vec![("main".into(), "(page a4)")]);
    let g = resolve_packages(
        vec![m],
        &sources,
        BuildTarget::Document,
        &RuntimeProfile::document(),
    )
    .unwrap();
    let json = g.lockfile.to_json().unwrap();
    let parsed = Lockfile::from_json(&json).unwrap();
    assert_eq!(parsed, g.lockfile);
}

#[test]
fn phase10_target_validates_entry() {
    let m = PackageManifest {
        name: "native-app".into(),
        version: "1.0".into(),
        dependencies: vec![],
        entry: "main".into(),
        targets: vec!["native".into()],
        ..Default::default()
    };
    let mut sources = BTreeMap::new();
    sources.insert("native-app".into(), vec![("main".into(), "(page a4)")]);
    assert!(resolve_packages(
        vec![m],
        &sources,
        BuildTarget::Native,
        &RuntimeProfile::native_preview()
    )
    .is_err());
}

#[test]
fn phase10_same_lockfile_from_shuffled_input() {
    let ma = PackageManifest {
        name: "a".into(),
        version: "1".into(),
        dependencies: vec![],
        entry: "main".into(),
        targets: vec![],
        ..Default::default()
    };
    let mb = PackageManifest {
        name: "b".into(),
        version: "1".into(),
        dependencies: vec![],
        entry: "main".into(),
        targets: vec![],
        ..Default::default()
    };
    let mut sources = BTreeMap::new();
    sources.insert("a".into(), vec![("main".into(), "(page a4)")]);
    sources.insert("b".into(), vec![("main".into(), "(page a4)")]);
    let g1 = resolve_packages(
        vec![ma.clone(), mb.clone()],
        &sources,
        BuildTarget::Document,
        &RuntimeProfile::document(),
    )
    .unwrap();
    let g2 = resolve_packages(
        vec![mb, ma],
        &sources,
        BuildTarget::Document,
        &RuntimeProfile::document(),
    )
    .unwrap();
    assert_eq!(g1.lockfile, g2.lockfile);
}

#[test]
fn phase10_parses_package_rpxm() {
    let m = parse_rpxm(
        r#"(package
  (name demo)
  (version 0.1.0)
  (entry main.rpx)
  (dep util 1.0 (path ../util))
  (target document))"#,
    )
    .unwrap();
    assert_eq!(m.name, "demo");
    assert_eq!(m.dependencies[0].name, "util");
}

#[test]
fn phase10_build_graph_and_incremental_cache() {
    let m = parse_rpxm(
        r#"(package (name app) (version 1) (entry main.rpx) (dep lib 1) (target document))"#,
    )
    .unwrap();
    let g = BuildGraph::from_manifest(&m, BuildTarget::Document);
    let order = g.topo_order();
    assert!(order.len() >= 2);
    let lib = BuildNodeId("lib:document".into());
    let contract = g.invalidate(&lib, InvalidationKind::Contract);
    let impl_only = g.invalidate(&lib, InvalidationKind::Implementation);
    assert!(contract.len() > impl_only.len());
    let mut cache = IncrementalCache::default();
    cache.put(lib.clone(), "h1");
    assert!(!cache.needs_rebuild(&lib, "h1"));
    cache.invalidate_set(&contract);
    assert!(cache.needs_rebuild(&lib, "h1"));
    assert!(diagnose_manifest(&m).is_empty());
}

#[test]
fn phase10_rpxm_error_paths() {
    assert!(matches!(parse_rpxm(""), Err(RpxmError::Empty)));
    assert!(matches!(
        parse_rpxm("(package (version 1) (entry m))"),
        Err(RpxmError::MissingName)
    ));
    assert!(matches!(
        parse_rpxm("(package (name x) (entry m))"),
        Err(RpxmError::MissingVersion)
    ));
    assert!(matches!(
        parse_rpxm("(package (name x) (version 1))"),
        Err(RpxmError::MissingEntry)
    ));
}

#[test]
fn phase10_rpxm_comments_and_quotes() {
    let m = parse_rpxm(
        r#"(// top)
(package
  (name "quoted-name")
  (version 1)
  (entry main.rpx)
  (// dep comment)
  (dep util 1.0)
  (target document)
  (target slide))"#,
    )
    .unwrap();
    assert_eq!(m.name, "quoted-name");
    assert_eq!(m.dependencies[0].path, None);
    assert_eq!(m.targets.len(), 2);
}

#[test]
fn phase10_resolve_duplicate_and_missing() {
    use reciplexa_package::ResolveError;
    let m = PackageManifest {
        name: "dup".into(),
        version: "1".into(),
        dependencies: vec![],
        entry: "main".into(),
        targets: vec![],
        ..Default::default()
    };
    let mut sources = BTreeMap::new();
    sources.insert("dup".into(), vec![("main".into(), "(page a4)")]);
    assert!(matches!(
        resolve_packages(
            vec![m.clone(), m],
            &sources,
            BuildTarget::Document,
            &RuntimeProfile::document()
        ),
        Err(ResolveError::DuplicateName(_))
    ));
    let orphan = PackageManifest {
        name: "solo".into(),
        version: "1".into(),
        dependencies: vec![],
        entry: "main".into(),
        targets: vec![],
        ..Default::default()
    };
    assert!(matches!(
        resolve_packages(
            vec![orphan],
            &BTreeMap::new(),
            BuildTarget::Document,
            &RuntimeProfile::document()
        ),
        Err(ResolveError::MissingEntry(_))
    ));
}

#[test]
fn phase10_target_as_str_and_native_entry() {
    assert_eq!(BuildTarget::Slide.as_str(), "slide");
    assert_eq!(BuildTarget::Preview.as_str(), "preview");
    assert!(BuildTarget::Native.validate_entry("native_main").is_ok());
    assert!(BuildTarget::Native.validate_entry("main").is_err());
}

#[test]
fn phase10_diagnose_manifest_codes() {
    let bad = PackageManifest {
        name: "".into(),
        version: "1".into(),
        dependencies: vec![
            reciplexa_package::DependencySpec {
                name: "a".into(),
                version_req: "1".into(),
                path: None,
            package: None,
            },
            reciplexa_package::DependencySpec {
                name: "a".into(),
                version_req: "2".into(),
                path: None,
            package: None,
            },
        ],
        entry: "".into(),
        targets: vec![],
        ..Default::default()
    };
    let codes: Vec<_> = diagnose_manifest(&bad)
        .into_iter()
        .map(|d| d.code)
        .collect();
    assert!(codes.contains(&"PKG001".to_string()));
    assert!(codes.contains(&"PKG002".to_string()));
    assert!(codes.contains(&"PKG003".to_string()));
}

#[test]
fn phase10_topo_linear_order() {
    let m = parse_rpxm(
        r#"(package (name root) (version 1) (entry main.rpx) (dep left 1) (dep right 1) (target document))"#,
    )
    .unwrap();
    let g = BuildGraph::from_manifest(&m, BuildTarget::Document);
    let order = g.topo_order();
    let root_pos = order.iter().position(|n| n.0 == "root:document").unwrap();
    let left_pos = order.iter().position(|n| n.0 == "left:document").unwrap();
    let right_pos = order.iter().position(|n| n.0 == "right:document").unwrap();
    assert!(left_pos < root_pos);
    assert!(right_pos < root_pos);
}

#[test]
fn phase10_manifest_json_roundtrip() {
    let json = r#"{"name":"json","version":"0.1","dependencies":[],"entry":"main","targets":["document"]}"#;
    let parsed = PackageManifest::parse_json(json).unwrap();
    assert_eq!(parsed.name, "json");
    assert_eq!(parsed.entry, "main");
    assert!(Lockfile::from_json("not-json").is_err());
}
