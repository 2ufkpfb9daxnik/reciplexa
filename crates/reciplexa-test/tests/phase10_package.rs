//! Phase 10 conformance: package resolution and lockfile.

use std::collections::BTreeMap;

use reciplexa_package::{
    resolve_packages, BuildTarget, Lockfile, PackageManifest, RuntimeProfile,
};

#[test]
fn phase10_deterministic_lockfile() {
    let m = PackageManifest {
        name: "app".into(),
        version: "0.1.0".into(),
        dependencies: vec![],
        entry: "main".into(),
        targets: vec!["document".into()],
    };
    let mut sources = BTreeMap::new();
    sources.insert("app".into(), vec![("main".into(), "(page a4)")]);
    let g = resolve_packages(vec![m], &sources, BuildTarget::Document, &RuntimeProfile::document()).unwrap();
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
    };
    let mut sources = BTreeMap::new();
    sources.insert("native-app".into(), vec![("main".into(), "(page a4)")]);
    assert!(resolve_packages(vec![m], &sources, BuildTarget::Native, &RuntimeProfile::native_preview()).is_err());
}

#[test]
fn phase10_same_lockfile_from_shuffled_input() {
    let ma = PackageManifest {
        name: "a".into(),
        version: "1".into(),
        dependencies: vec![],
        entry: "main".into(),
        targets: vec![],
    };
    let mb = PackageManifest {
        name: "b".into(),
        version: "1".into(),
        dependencies: vec![],
        entry: "main".into(),
        targets: vec![],
    };
    let mut sources = BTreeMap::new();
    sources.insert("a".into(), vec![("main".into(), "(page a4)")]);
    sources.insert("b".into(), vec![("main".into(), "(page a4)")]);
    let g1 = resolve_packages(vec![ma.clone(), mb.clone()], &sources, BuildTarget::Document, &RuntimeProfile::document()).unwrap();
    let g2 = resolve_packages(vec![mb, ma], &sources, BuildTarget::Document, &RuntimeProfile::document()).unwrap();
    assert_eq!(g1.lockfile, g2.lockfile);
}
