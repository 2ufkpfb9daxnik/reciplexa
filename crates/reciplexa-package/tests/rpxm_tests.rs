use reciplexa_package::{parse_rpxm, RpxmError};

#[test]
fn parses_minimal_rpxm() {
    let m = parse_rpxm(
        r#"(package
  (name demo)
  (version 0.1.0)
  (entry main.rpx)
  (dep util 1.0 (path ../util))
  (target lib))"#,
    )
    .unwrap();
    assert_eq!(m.name, "demo");
    assert_eq!(m.dependencies[0].path.as_deref(), Some("../util"));
    assert_eq!(m.targets, vec!["lib".to_string()]);
}

#[test]
fn rejects_empty_input() {
    assert_eq!(parse_rpxm(""), Err(RpxmError::Empty));
    assert_eq!(parse_rpxm("   ; only comment\n"), Err(RpxmError::Empty));
}

#[test]
fn rejects_missing_name() {
    let err = parse_rpxm("(package (version 1) (entry main.rpx))").unwrap_err();
    assert_eq!(err, RpxmError::MissingName);
}

#[test]
fn rejects_missing_version() {
    let err = parse_rpxm("(package (name demo) (entry main.rpx))").unwrap_err();
    assert_eq!(err, RpxmError::MissingVersion);
}

#[test]
fn rejects_missing_entry() {
    let err = parse_rpxm("(package (name demo) (version 1.0))").unwrap_err();
    assert_eq!(err, RpxmError::MissingEntry);
}

#[test]
fn ignores_semicolon_comments() {
    let m = parse_rpxm(
        r#"; header comment
(package
  ; inline
  (name demo)
  (version 0.1.0)
  (entry main.rpx))"#,
    )
    .unwrap();
    assert_eq!(m.name, "demo");
    assert_eq!(m.version, "0.1.0");
}

#[test]
fn parses_quoted_strings() {
    let m = parse_rpxm(
        r#"(package
  (name "my package")
  (version "0.2.0")
  (entry "src/main.rpx"))"#,
    )
    .unwrap();
    assert_eq!(m.name, "my package");
    assert_eq!(m.entry, "src/main.rpx");
}

#[test]
fn parses_dep_without_path() {
    let m = parse_rpxm(
        r#"(package
  (name app)
  (version 1)
  (entry main.rpx)
  (dep util 2.0))"#,
    )
    .unwrap();
    assert_eq!(m.dependencies.len(), 1);
    assert_eq!(m.dependencies[0].name, "util");
    assert_eq!(m.dependencies[0].version_req, "2.0");
    assert_eq!(m.dependencies[0].path, None);
}

#[test]
fn parses_multiple_targets() {
    let m = parse_rpxm(
        r#"(package
  (name multi)
  (version 1)
  (entry main.rpx)
  (target document)
  (target slide)
  (target preview))"#,
    )
    .unwrap();
    assert_eq!(
        m.targets,
        vec![
            "document".to_string(),
            "slide".to_string(),
            "preview".to_string()
        ]
    );
}

#[test]
fn rejects_unclosed_string_syntax() {
    let err = parse_rpxm(r#"(name "broken)"#).unwrap_err();
    match err {
        RpxmError::Syntax(msg) => assert!(msg.contains("unclosed")),
        other => panic!("expected Syntax, got {other:?}"),
    }
}

#[test]
fn skips_unknown_forms_and_bare_tokens() {
    let m = parse_rpxm(
        r#"(package
  (name demo)
  (version 1)
  (entry main.rpx)
  (unknown ignored)
  bare-token
  (target document))"#,
    )
    .unwrap();
    assert_eq!(m.name, "demo");
    assert_eq!(m.targets, vec!["document".to_string()]);
}
