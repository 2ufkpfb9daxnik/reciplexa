use reciplexa_package::{DependencySpec, LockedPackage, Lockfile, PackageManifest};

#[test]
fn manifest_rejects_garbage_json() {
    assert!(PackageManifest::parse_json("not-json").is_err());
    assert!(PackageManifest::parse_json("{}").is_err());
}

#[test]
fn manifest_parse_json_roundtrip() {
    let m = PackageManifest {
        name: "demo".into(),
        version: "0.1.0".into(),
        dependencies: vec![DependencySpec {
            name: "util".into(),
            version_req: "1.0".into(),
            path: Some("../util".into()),
            package: None,
        }],
        entry: "main.rpx".into(),
        targets: vec!["document".into()],
        ..Default::default()
    };
    let json = serde_json::to_string(&m).unwrap();
    let parsed = PackageManifest::parse_json(&json).unwrap();
    assert_eq!(parsed, m);
}

#[test]
fn lockfile_rejects_garbage_json() {
    assert!(Lockfile::from_json("{not json").is_err());
    assert!(Lockfile::from_json("[]").is_err());
}

#[test]
fn lockfile_json_roundtrip() {
    let lf = Lockfile {
        packages: vec![LockedPackage {
            name: "demo".into(),
            version: "1.0".into(),
            source: "workspace".into(),
            dependencies: vec![],
        }],
    };
    let json = lf.to_json().unwrap();
    let parsed = Lockfile::from_json(&json).unwrap();
    assert_eq!(parsed, lf);
}

#[test]
fn lockfile_from_graph_matches_manifests() {
    let manifests = vec![PackageManifest {
        name: "a".into(),
        version: "0.1".into(),
        dependencies: vec![],
        entry: "main".into(),
        targets: vec![],
        ..Default::default()
    }];
    let lf = Lockfile::from_graph(&manifests);
    assert_eq!(lf.packages[0].name, "a");
    assert_eq!(lf.packages[0].version, "0.1");
    assert_eq!(lf.packages[0].source, "workspace");
}
