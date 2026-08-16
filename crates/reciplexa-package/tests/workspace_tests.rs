use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use reciplexa_package::{
    check_package_lock_consistency, content_checksum, discover_workspace, find_enclosing_workspace,
    parse_workspace_rpxm, read_lock_for_package, resolve_workspace_dependencies, RpxmError,
    WorkspaceError,
};

fn tmp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.tmp")
        .join(format!("pkg-ws-{label}-{nanos}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_pkg(root: &std::path::Path, name: &str, version: &str) {
    fs::create_dir_all(root).unwrap();
    fs::write(
        root.join("package.rpxm"),
        format!(
            r#"(package {name}
  format-version 1
  version "{version}"
  (public-modules {name}))"#
        ),
    )
    .unwrap();
}

#[test]
fn parses_dd001_workspace_members() {
    let ws = parse_workspace_rpxm(
        r#"(workspace
  format-version 1
  (members
    "document-core"
    "document-render"
    "document-cli"))"#,
    )
    .unwrap();
    assert_eq!(ws.format_version, 1);
    assert_eq!(
        ws.members,
        vec!["document-core", "document-render", "document-cli"]
    );
}

#[test]
fn rejects_empty_members() {
    let err = parse_workspace_rpxm("(workspace format-version 1 (members))").unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));
}

#[test]
fn rejects_wrong_root() {
    let err = parse_workspace_rpxm("(package demo)").unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));
}

#[test]
fn discover_workspace_loads_members() {
    let root = tmp_dir("ok");
    write_pkg(&root.join("document-core"), "document-core", "1.0.0");
    write_pkg(&root.join("document-render"), "document-render", "0.2.0");
    fs::write(
        root.join("workspace.rpxm"),
        r#"(workspace
  format-version 1
  (members
    "document-core"
    "document-render"))"#,
    )
    .unwrap();

    let idx = discover_workspace(&root).unwrap();
    assert_eq!(idx.manifest.members.len(), 2);
    assert_eq!(idx.members.len(), 2);
    assert_eq!(idx.members["document-core"].1.version, "1.0.0");
    assert_eq!(idx.members["document-render"].1.version, "0.2.0");
    assert!(idx.members["document-core"].0.ends_with("document-core"));
}

#[test]
fn discover_rejects_duplicate_formal_names() {
    let root = tmp_dir("dup");
    write_pkg(&root.join("a"), "shared", "1.0.0");
    write_pkg(&root.join("b"), "shared", "2.0.0");
    fs::write(
        root.join("workspace.rpxm"),
        r#"(workspace format-version 1 (members "a" "b"))"#,
    )
    .unwrap();
    let err = discover_workspace(&root).unwrap_err();
    match err {
        WorkspaceError::DuplicateName(msg) => assert!(msg.contains("shared")),
        other => panic!("expected DuplicateName, got {other:?}"),
    }
}

#[test]
fn discover_rejects_nested_workspace() {
    let root = tmp_dir("nest");
    write_pkg(&root.join("core"), "core", "1.0.0");
    fs::write(
        root.join("core").join("workspace.rpxm"),
        "(workspace (members \"x\"))\n",
    )
    .unwrap();
    fs::write(
        root.join("workspace.rpxm"),
        r#"(workspace format-version 1 (members "core"))"#,
    )
    .unwrap();
    let err = discover_workspace(&root).unwrap_err();
    assert!(matches!(err, WorkspaceError::NestedWorkspace(_)));
}

#[test]
fn discover_rejects_missing_member_package() {
    let root = tmp_dir("miss");
    fs::create_dir_all(root.join("ghost")).unwrap();
    fs::write(
        root.join("workspace.rpxm"),
        r#"(workspace format-version 1 (members "ghost"))"#,
    )
    .unwrap();
    let err = discover_workspace(&root).unwrap_err();
    assert!(matches!(err, WorkspaceError::MissingMember(_)));
}

#[test]
fn workspace_shared_lock_roundtrip() {
    let root = tmp_dir("lock");
    write_pkg(&root.join("core"), "core", "1.0.0");
    write_pkg(&root.join("render"), "render", "0.1.0");
    fs::write(
        root.join("workspace.rpxm"),
        r#"(workspace format-version 1 (members "core" "render"))"#,
    )
    .unwrap();
    let idx = discover_workspace(&root).unwrap();
    let written = idx.write_lock().unwrap();
    assert_eq!(written.packages.len(), 2);
    assert!(written.packages.iter().all(|p| p.source == "workspace"));
    for pkg in &written.packages {
        let member_root = root.join(&pkg.name);
        let expected = content_checksum(member_root.join("package.rpxm"));
        assert_eq!(
            pkg.checksum.as_deref(),
            Some(expected.as_str()),
            "workspace lock should fill member checksum for {}",
            pkg.name
        );
        assert!(
            expected.starts_with("sha256:"),
            "sha256 prefix, got {expected}"
        );
    }
    assert!(idx.lock_path().is_file());
    let read = idx.read_lock().unwrap();
    assert_eq!(read, written);
    let written_again = idx.write_lock().unwrap();
    assert_eq!(
        written.to_json().unwrap(),
        written_again.to_json().unwrap(),
        "lock write is reproducible"
    );
}

#[test]
fn workspace_rejects_member_local_lock() {
    let root = tmp_dir("memlock");
    write_pkg(&root.join("core"), "core", "1.0.0");
    fs::write(root.join("core").join("rpx.lock"), "{\"packages\":[]}\n").unwrap();
    fs::write(
        root.join("workspace.rpxm"),
        r#"(workspace format-version 1 (members "core"))"#,
    )
    .unwrap();
    let idx = discover_workspace(&root).unwrap();
    let err = idx.write_lock().unwrap_err();
    assert!(matches!(err, WorkspaceError::MemberLocalLock(_)));
    let err = idx.read_lock().unwrap_err();
    assert!(matches!(err, WorkspaceError::MemberLocalLock(_)));
}

#[test]
fn member_uses_workspace_root_lock() {
    let root = tmp_dir("e3");
    write_pkg(&root.join("core"), "core", "1.0.0");
    write_pkg(&root.join("render"), "render", "0.1.0");
    fs::write(
        root.join("workspace.rpxm"),
        r#"(workspace format-version 1 (members "core" "render"))"#,
    )
    .unwrap();
    let idx = discover_workspace(&root).unwrap();
    idx.write_lock().unwrap();

    let found = find_enclosing_workspace(root.join("core"))
        .unwrap()
        .unwrap();
    assert_eq!(found.root, idx.root);

    let (lock_path, lock) = read_lock_for_package(root.join("core")).unwrap();
    assert_eq!(lock_path, root.join("rpx.lock"));
    assert_eq!(lock.packages.len(), 2);

    check_package_lock_consistency(root.join("core")).unwrap();
    check_package_lock_consistency(root.join("render")).unwrap();
}

fn write_pkg_with(root: &std::path::Path, body: &str) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join("package.rpxm"), body).unwrap();
}

#[test]
fn member_lock_consistency_detects_version_mismatch() {
    let root = tmp_dir("e3bad");
    write_pkg(&root.join("core"), "core", "1.0.0");
    fs::write(
        root.join("workspace.rpxm"),
        r#"(workspace format-version 1 (members "core"))"#,
    )
    .unwrap();
    let idx = discover_workspace(&root).unwrap();
    idx.write_lock().unwrap();
    // Mutate package version without refreshing the lock.
    write_pkg(&root.join("core"), "core", "9.9.9");
    let err = check_package_lock_consistency(root.join("core")).unwrap_err();
    match err {
        WorkspaceError::Lock(msg) => assert!(msg.contains("not consistent")),
        other => panic!("expected Lock consistency error, got {other:?}"),
    }
}

#[test]
fn resolve_prefers_workspace_members() {
    let root = tmp_dir("e4ok");
    write_pkg_with(
        &root.join("core"),
        r#"(package core
  format-version 1
  version "1.0.0"
  (public-modules core))"#,
    );
    write_pkg_with(
        &root.join("app"),
        r#"(package app
  format-version 1
  version "0.1.0"
  (public-modules app)
  (dependencies
    (core
      package core
      version "1.0.0")))"#,
    );
    fs::write(
        root.join("workspace.rpxm"),
        r#"(workspace format-version 1 (members "core" "app"))"#,
    )
    .unwrap();
    let idx = discover_workspace(&root).unwrap();
    let lock = resolve_workspace_dependencies(&idx).unwrap();
    let app = lock.packages.iter().find(|p| p.name == "app").unwrap();
    assert_eq!(app.source, "workspace");
    assert_eq!(app.dependencies, vec!["core".to_string()]);
    let core = lock.packages.iter().find(|p| p.name == "core").unwrap();
    assert_eq!(core.source, "workspace");
    for (name, dir) in [("app", "app"), ("core", "core")] {
        let expected = content_checksum(root.join(dir).join("package.rpxm"));
        let locked = lock.packages.iter().find(|p| p.name == name).unwrap();
        assert_eq!(
            locked.checksum.as_deref(),
            Some(expected.as_str()),
            "resolve_workspace_dependencies should fill checksum for {name}"
        );
    }
}

#[test]
fn resolve_detects_member_dependency_cycle() {
    let root = tmp_dir("e4cycle");
    write_pkg_with(
        &root.join("a"),
        r#"(package a
  format-version 1
  version "1.0.0"
  (public-modules a)
  (dependencies
    (b package b version "1.0.0")))"#,
    );
    write_pkg_with(
        &root.join("b"),
        r#"(package b
  format-version 1
  version "1.0.0"
  (public-modules b)
  (dependencies
    (a package a version "1.0.0")))"#,
    );
    fs::write(
        root.join("workspace.rpxm"),
        r#"(workspace format-version 1 (members "a" "b"))"#,
    )
    .unwrap();
    let idx = discover_workspace(&root).unwrap();
    let err = resolve_workspace_dependencies(&idx).unwrap_err();
    assert!(matches!(err, WorkspaceError::DependencyCycle(_)));
}

#[test]
fn resolve_registry_source_is_open_stub() {
    let root = tmp_dir("e4reg");
    write_pkg_with(
        &root.join("app"),
        r#"(package app
  format-version 1
  version "0.1.0"
  (public-modules app)
  (dependencies
    (remote
      package remote-kit
      version "1.0.0"
      source registry)))"#,
    );
    fs::write(
        root.join("workspace.rpxm"),
        r#"(workspace format-version 1 (members "app"))"#,
    )
    .unwrap();
    let idx = discover_workspace(&root).unwrap();
    let err = resolve_workspace_dependencies(&idx).unwrap_err();
    assert_eq!(err.code(), Some(reciplexa_package::OPEN_PKG_001_CODE));
    match &err {
        WorkspaceError::RegistryUnavailable(msg) => {
            assert!(msg.contains("OPEN-PKG-001"));
            assert!(msg.contains("no network"));
        }
        other => panic!("expected RegistryUnavailable, got {other:?}"),
    }
}

/// Checked-in fixture: workspace-level `source registry` refuses with OPEN-PKG-001.
#[test]
fn open_pkg_001_registry_fixture_refuses_with_code() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/open_pkg_001_registry");
    let idx = discover_workspace(&root).expect("fixture workspace");
    assert!(
        idx.manifest
            .dependencies
            .iter()
            .any(|d| d.source.as_deref() == Some("registry")),
        "fixture should declare workspace-level registry dep"
    );
    let err = resolve_workspace_dependencies(&idx).expect_err("registry must refuse");
    assert_eq!(
        err.code(),
        Some(reciplexa_package::OPEN_PKG_001_CODE),
        "expected structured OPEN-PKG-001, got {err}"
    );
    let msg = err.to_string();
    assert!(msg.contains("OPEN-PKG-001"));
    assert!(msg.contains("no network"));
    assert!(msg.contains("remote-kit") || msg.contains("registry"));
    assert!(
        msg.contains("workspace"),
        "refusal should mention workspace-level resolve: {msg}"
    );
}

#[test]
fn verify_lock_skips_without_package_root_or_lock() {
    use reciplexa_package::{
        elaborate_with_packages, verify_package_lock_for_entry, LocalPackageIndex,
    };

    let dir = tmp_dir("nolock");
    let entry = dir.join("main.rpx");
    fs::write(&entry, "(val main 1)\n").unwrap();
    verify_package_lock_for_entry(&entry).unwrap();

    write_pkg(&dir, "solo", "1");
    fs::write(dir.join("main.rpx"), "(val main 1)\n").unwrap();
    verify_package_lock_for_entry(&dir.join("main.rpx")).unwrap();

    let idx = LocalPackageIndex::default();
    elaborate_with_packages(&entry, &idx).unwrap();
}

#[test]
fn verify_lock_pkg006_blocks_elaborate() {
    use reciplexa_package::{
        elaborate_with_packages, parse_rpxm, verify_package_lock_for_entry, LocalPackageIndex,
    };

    let dir = tmp_dir("pkg006load");
    let lib = dir.join("lib");
    write_pkg(&lib, "lib", "1");
    let rpxm = lib.join("package.rpxm");
    fs::write(
        dir.join("package.rpxm"),
        r#"(package app
  format-version 1
  version "1"
  (dependencies
    (lib
      package lib
      version "1"
      path "lib")))"#,
    )
    .unwrap();
    fs::write(dir.join("main.rpx"), "(import lib)\n(val main 1)\n").unwrap();

    let consumer = parse_rpxm(&fs::read_to_string(dir.join("package.rpxm")).unwrap()).unwrap();
    let idx = LocalPackageIndex::discover(&[&dir]).unwrap();
    let mut lock = idx.lock_consumer(&consumer).unwrap();
    let lib_entry = lock
        .packages
        .iter_mut()
        .find(|p| p.name == "lib")
        .expect("locked lib");
    lib_entry.checksum =
        Some("sha256:deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into());
    lock.write_rpx_lock(dir.join("rpx.lock")).unwrap();

    let entry = dir.join("main.rpx");
    let err = verify_package_lock_for_entry(&entry).unwrap_err();
    assert!(
        err.to_string().contains("PKG006"),
        "expected PKG006, got {err}"
    );

    let err = elaborate_with_packages(&entry, &idx).unwrap_err();
    assert!(err.to_string().contains("lock verify"));
    assert!(err.to_string().contains("PKG006"));

    let good = idx.lock_consumer(&consumer).unwrap();
    assert_eq!(
        good.packages
            .iter()
            .find(|p| p.name == "lib")
            .and_then(|p| p.checksum.as_deref()),
        Some(content_checksum(&rpxm).as_str())
    );
    good.write_rpx_lock(dir.join("rpx.lock")).unwrap();
    verify_package_lock_for_entry(&entry).unwrap();
}
