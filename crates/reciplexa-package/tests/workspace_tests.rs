use reciplexa_package::{parse_workspace_rpxm, RpxmError};

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
