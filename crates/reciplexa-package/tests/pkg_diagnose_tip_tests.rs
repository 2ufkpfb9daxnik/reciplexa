//! Tip coverage: diagnose codes PKG004 (missing resource) + PKG005 (registry dep).

use reciplexa_package::{
    diagnose_manifest, diagnose_manifest_with_root, DependencySpec, PackageManifest,
};

#[test]
fn tip_pkg004_missing_listed_resource() {
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-pkg004-tip-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(dir.join("resources")).unwrap();
    let m = PackageManifest {
        name: "tip-app".into(),
        version: "1".into(),
        resources: vec!["gone.png".into()],
        ..Default::default()
    };
    assert!(
        diagnose_manifest_with_root(&m, &dir)
            .iter()
            .any(|d| d.code == "PKG004"),
        "PKG004 tip"
    );
    assert!(!diagnose_manifest(&m).iter().any(|d| d.code == "PKG004"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tip_pkg005_registry_dependency() {
    let m = PackageManifest {
        name: "tip-app".into(),
        version: "1".into(),
        dependencies: vec![DependencySpec {
            name: "remote".into(),
            version_req: "1".into(),
            path: None,
            package: Some("remote".into()),
            source: Some("registry".into()),
        }],
        ..Default::default()
    };
    let hit = diagnose_manifest(&m)
        .into_iter()
        .find(|d| d.code == "PKG005")
        .expect("PKG005 tip");
    assert!(hit.message.contains("OPEN-PKG-001"));
}
