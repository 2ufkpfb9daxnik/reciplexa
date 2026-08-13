use reciplexa_package::{normalize_resource_path, parse_rpxm, RpxmError};

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
    assert_eq!(parse_rpxm("   (// only comment)\n"), Err(RpxmError::Empty));
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
fn ignores_structured_comments() {
    let m = parse_rpxm(
        r#"(// header comment)
(package
  (// inline)
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
  (name "my-package")
  (version "0.2.0")
  (entry "src/main.rpx"))"#,
    )
    .unwrap();
    assert_eq!(m.name, "my-package");
    assert_eq!(m.entry, "src/main.rpx");
}

#[test]
fn rejects_invalid_package_names() {
    let space = parse_rpxm(
        r#"(package
  (name "my package")
  (version "0.2.0")
  (entry "src/main.rpx"))"#,
    )
    .unwrap_err();
    assert!(matches!(space, RpxmError::Syntax(_)));

    let upper = parse_rpxm("(package (name Demo) (version 0.1.0) (entry main.rpx))").unwrap_err();
    assert!(matches!(upper, RpxmError::Syntax(_)));

    let under = parse_rpxm("(package (name my_pkg) (version 0.1.0) (entry main.rpx))").unwrap_err();
    assert!(matches!(under, RpxmError::Syntax(_)));
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
fn rejects_unknown_fields() {
    let err = parse_rpxm(
        r#"(package
  (name demo)
  (version 1)
  (entry main.rpx)
  (unknown ignored)
  bare-token
  (target document))"#,
    )
    .unwrap_err();
    match err {
        RpxmError::UnknownField(msg) => assert!(msg.contains("unknown")),
        other => panic!("expected UnknownField, got {other:?}"),
    }

    let err = parse_rpxm(
        r#"(package demo
  format-version 1
  version "1.0.0"
  soruce-root "src"
  (public-modules main))"#,
    )
    .unwrap_err();
    match err {
        RpxmError::UnknownField(msg) => {
            assert!(msg.contains("soruce-root"));
            assert!(msg.contains("source-root"));
        }
        other => panic!("expected UnknownField, got {other:?}"),
    }
}

#[test]
fn parses_dd001_library_manifest() {
    let m = parse_rpxm(
        r#"(package graphics
  format-version 1
  version "0.1.0"
  source-root "src"
  (public-modules shapes color))"#,
    )
    .unwrap();
    assert_eq!(m.name, "graphics");
    assert_eq!(m.version, "0.1.0");
    assert_eq!(m.format_version, 1);
    assert_eq!(m.source_root, "src");
    assert_eq!(
        m.public_modules,
        vec!["shapes".to_string(), "color".to_string()]
    );
    assert!(m.entry.is_empty());
}

#[test]
fn parses_dd001_dependencies_with_path() {
    let m = parse_rpxm(
        r#"(package app
  format-version 1
  version "1.0.0"
  (public-modules main)
  (dependencies
    (graphics
      package graphics
      version "0.1.0"
      path "../graphics")))"#,
    )
    .unwrap();
    assert_eq!(m.dependencies.len(), 1);
    assert_eq!(m.dependencies[0].name, "graphics");
    assert_eq!(m.dependencies[0].package.as_deref(), Some("graphics"));
    assert_eq!(m.dependencies[0].path.as_deref(), Some("../graphics"));
}

#[test]
fn rejects_unsupported_format_version() {
    let err = parse_rpxm(
        r#"(package demo
  format-version 99
  version "1.0.0"
  (public-modules main))"#,
    )
    .unwrap_err();
    assert_eq!(err, RpxmError::UnsupportedFormatVersion(99));
}

#[test]
fn parses_resources_list_and_normalizes() {
    let m = parse_rpxm(
        r#"(package demo
  format-version 1
  version "1.0.0"
  (public-modules main)
  (resources
    "styles/default.css"
    "images"
    "fonts\body.woff2"
    "locale"))"#,
    )
    .unwrap();
    assert_eq!(
        m.resources,
        vec![
            "styles/default.css".to_string(),
            "images".to_string(),
            "fonts/body.woff2".to_string(),
            "locale".to_string(),
        ]
    );
    assert_eq!(m.resource_root, "resources");
}

#[test]
fn rejects_escaping_resource_paths_pkg10() {
    let err = parse_rpxm(
        r#"(package demo
  format-version 1
  version "1.0.0"
  (public-modules main)
  (resources
    "../secret.txt"))"#,
    )
    .unwrap_err();
    match err {
        RpxmError::Syntax(msg) => {
            assert!(msg.contains("resource path escapes the package resource root"));
        }
        other => panic!("expected Syntax escape error, got {other:?}"),
    }

    for bad in ["/abs", "a//b", "a/../b", "C:/win", r"\rooted", ".", ""] {
        let err = normalize_resource_path(bad).unwrap_err();
        assert!(
            err.contains("resource path escapes the package resource root"),
            "path `{bad}`: {err}"
        );
    }
}
