use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use reciplexa_package::{discover_workspace, parse_workspace_rpxm, RpxmError, WorkspaceError};

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
    fs::write(root.join("core").join("workspace.rpxm"), "(workspace (members \"x\"))\n").unwrap();
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
    assert!(idx.lock_path().is_file());
    let read = idx.read_lock().unwrap();
    assert_eq!(read, written);
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
