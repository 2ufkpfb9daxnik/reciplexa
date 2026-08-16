//! Tip coverage: diagnose codes PKG004 (missing resource) + PKG005 (registry dep)
//! + PKG006 (path-dep lock checksum mismatch) + `content_checksum` stub edges.

use reciplexa_package::{
    content_checksum, diagnose_lockfile_checksums, diagnose_manifest, diagnose_manifest_with_root,
    DependencySpec, LockedPackage, Lockfile, PackageManifest,
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

#[test]
fn tip_pkg006_lock_checksum_mismatch() {
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-pkg006-tip-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let dep = dir.join("lib");
    std::fs::create_dir_all(&dep).unwrap();
    let rpxm = dep.join("package.rpxm");
    std::fs::write(&rpxm, "(package lib format-version 1 version \"1\")\n").unwrap();
    let actual = content_checksum(&rpxm);

    let ok = Lockfile {
        packages: vec![LockedPackage {
            name: "lib".into(),
            version: "1".into(),
            source: "path:lib".into(),
            dependencies: vec![],
            checksum: Some(actual),
        }],
    };
    assert!(
        diagnose_lockfile_checksums(&ok, &dir).is_empty(),
        "matching stub quiet"
    );

    let bad = Lockfile {
        packages: vec![LockedPackage {
            name: "lib".into(),
            version: "1".into(),
            source: "path:lib".into(),
            dependencies: vec![],
            checksum: Some(
                "sha256:deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(),
            ),
        }],
    };
    assert!(
        diagnose_lockfile_checksums(&bad, &dir)
            .iter()
            .any(|d| d.code == "PKG006"),
        "PKG006 tip"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tip_cs0_from_consumer_fills_rpxm_checksum() {
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-cs0-tip-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let dep_root = dir.join("util");
    std::fs::create_dir_all(&dep_root).unwrap();
    let rpxm = dep_root.join("package.rpxm");
    std::fs::write(
        &rpxm,
        "(package util format-version 1 version \"1\" (public-modules core))\n",
    )
    .unwrap();
    let consumer = PackageManifest {
        name: "app".into(),
        version: "1".into(),
        dependencies: vec![DependencySpec {
            name: "u".into(),
            version_req: "1".into(),
            path: Some("util".into()),
            package: Some("util".into()),
            source: None,
        }],
        entry: "main.rpx".into(),
        ..Default::default()
    };
    let dep = PackageManifest {
        name: "util".into(),
        version: "1".into(),
        public_modules: vec!["core".into()],
        ..Default::default()
    };
    let spec = &consumer.dependencies[0];
    let lf =
        Lockfile::from_consumer_with_roots(&consumer, &[(spec, &dep, Some(dep_root.as_path()))]);
    let locked = lf.packages.iter().find(|p| p.name == "util").unwrap();
    assert_eq!(
        locked.checksum.as_deref(),
        Some(content_checksum(&rpxm).as_str())
    );
    assert!(diagnose_lockfile_checksums(&lf, &dir).is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tip_content_checksum_empty_file_and_missing_path() {
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-checksum-empty-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let empty = dir.join("empty.bin");
    std::fs::write(&empty, b"").unwrap();
    let hash = content_checksum(&empty);
    assert!(
        hash.starts_with("sha256:"),
        "empty file uses sha256 prefix, got {hash}"
    );
    assert_eq!(hash.len(), "sha256:".len() + 64);
    assert_eq!(
        hash,
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(hash, content_checksum(&empty), "empty-file hash is stable");

    let missing = content_checksum(dir.join("no-such.bin"));
    assert!(
        missing.starts_with("stub-error:"),
        "missing path → stub-error, got {missing}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tip_pkg006_skips_workspace_source_even_with_checksum() {
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-pkg006-ws-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let lock = Lockfile {
        packages: vec![LockedPackage {
            name: "core".into(),
            version: "1".into(),
            source: "workspace".into(),
            dependencies: vec![],
            checksum: Some(
                "sha256:deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(),
            ),
        }],
    };
    assert!(
        diagnose_lockfile_checksums(&lock, &dir).is_empty(),
        "PKG006 is path-dep only; workspace checksums are recorded, not compared"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tip_pkg006_missing_rpxm_mismatches_stored_stub() {
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-pkg006-miss-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    // No package.rpxm → content_checksum is stub-error:…
    let lock = Lockfile {
        packages: vec![LockedPackage {
            name: "lib".into(),
            version: "1".into(),
            source: "path:lib".into(),
            dependencies: vec![],
            checksum: Some(
                "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into(),
            ),
        }],
    };
    let diags = diagnose_lockfile_checksums(&lock, &dir);
    let hit = diags.iter().find(|d| d.code == "PKG006").expect("PKG006");
    assert!(hit.message.contains("lib"), "{}", hit.message);
    assert!(
        hit.message.contains("stub-error:") || hit.message.contains("sha256:"),
        "mismatch should cite stub hashes: {}",
        hit.message
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tip_pkg008_resource_content_hash_replay() {
    use reciplexa_eval::RuntimeValue;
    use reciplexa_package::{
        diagnose_package_resource_replay, resource_file_content_hash, PACKAGE_RESOURCE_TAG,
    };

    let dir = std::env::temp_dir().join(format!(
        "reciplexa-pkg008-tip-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("logo.png");
    std::fs::write(&path, b"png").unwrap();
    let actual = resource_file_content_hash(&path);
    let v = RuntimeValue::Record(vec![
        (
            "tag".into(),
            RuntimeValue::String(PACKAGE_RESOURCE_TAG.into()),
        ),
        ("path".into(), RuntimeValue::String("logo.png".into())),
        (
            "resolved-path".into(),
            RuntimeValue::String(path.to_string_lossy().into_owned()),
        ),
        (
            "content-hash".into(),
            RuntimeValue::String(
                "sha256:deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(),
            ),
        ),
    ]);
    let hit = diagnose_package_resource_replay(&v)
        .into_iter()
        .find(|d| d.code == "PKG008")
        .expect("PKG008 tip");
    assert!(hit.message.contains("logo.png"));
    assert_ne!(
        actual,
        "sha256:deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
