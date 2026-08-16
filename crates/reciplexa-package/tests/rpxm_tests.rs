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

#[test]
fn listed_resources_must_exist_under_resource_root() {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use reciplexa_package::ResourceCheckError;

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.tmp")
        .join(format!("pkg-res-{nanos}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("resources/images")).unwrap();
    fs::write(root.join("resources/images/logo.png"), b"png").unwrap();

    let m = parse_rpxm(
        r#"(package demo
  format-version 1
  version "1.0.0"
  (public-modules main)
  (resources
    "images/logo.png"
    "missing.txt"))"#,
    )
    .unwrap();
    let err = m.check_resources_exist(&root).unwrap_err();
    match err {
        ResourceCheckError::Missing { resource, .. } => {
            assert_eq!(resource, "missing.txt");
        }
        other => panic!("expected Missing, got {other:?}"),
    }

    let ok = parse_rpxm(
        r#"(package demo
  format-version 1
  version "1.0.0"
  (public-modules main)
  (resources "images/logo.png"))"#,
    )
    .unwrap();
    ok.check_resources_exist(&root).unwrap();
}

#[test]
fn resolve_package_resource_under_resource_root() {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use reciplexa_package::{resolve_package_resource, ResourceResolveError};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.tmp")
        .join(format!("pkg-resolve-res-{nanos}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("resources/images")).unwrap();
    fs::write(root.join("resources/images/logo.png"), b"png").unwrap();

    let m = parse_rpxm(
        r#"(package demo
  format-version 1
  version "1.0.0"
  (public-modules main)
  (resources "images/logo.png"))"#,
    )
    .unwrap();

    let path = resolve_package_resource(&root, &m, "images/logo.png").unwrap();
    assert_eq!(path, root.join("resources/images/logo.png"));
    assert_eq!(
        m.resolve_package_resource(&root, r"images\logo.png")
            .unwrap(),
        path
    );

    let err = resolve_package_resource(&root, &m, "images/other.png").unwrap_err();
    match err {
        ResourceResolveError::NotListed { resource } => {
            assert_eq!(resource, "images/other.png");
        }
        other => panic!("expected NotListed, got {other:?}"),
    }

    let err = resolve_package_resource(&root, &m, "../escape.txt").unwrap_err();
    match err {
        ResourceResolveError::InvalidPath(msg) => {
            assert!(msg.contains("escapes the package resource root"));
        }
        other => panic!("expected InvalidPath, got {other:?}"),
    }
}

#[test]
fn language_resource_path_resolves_when_package_root_available() {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use reciplexa_eval::{eval_source, RuntimeValue};
    use reciplexa_package::{
        materialize_package_resource, resolve_resource_value, ResourceValueError,
    };

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.tmp")
        .join(format!("pkg-lang-resource-{nanos}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("resources/images")).unwrap();
    fs::write(root.join("resources/images/logo.png"), b"png").unwrap();

    let m = parse_rpxm(
        r#"(package demo
  format-version 1
  version "1.0.0"
  (public-modules main)
  (resources "images/logo.png"))"#,
    )
    .unwrap();

    let v = eval_source(r#"(val main (resource "images/logo.png"))"#).unwrap();
    let RuntimeValue::Record(fields) = &v else {
        panic!("expected package-resource record");
    };
    let note = fields
        .iter()
        .find(|(k, _)| k == "note")
        .and_then(|(_, v)| match v {
            RuntimeValue::String(s) => Some(s.as_str()),
            _ => None,
        });
    assert_eq!(note, Some("resolve at package load"));

    let abs = materialize_package_resource(&v, &root, &m).unwrap();
    assert_eq!(abs, root.join("resources/images/logo.png"));
    assert_eq!(resolve_resource_value(&v, &root, &m).unwrap(), abs);

    let err = materialize_package_resource(&RuntimeValue::Int(1), &root, &m).unwrap_err();
    assert!(matches!(err, ResourceValueError::NotPackageResource(_)));

    let bad_tag = RuntimeValue::Record(vec![
        ("tag".into(), RuntimeValue::String("other".into())),
        (
            "path".into(),
            RuntimeValue::String("images/logo.png".into()),
        ),
    ]);
    let err = materialize_package_resource(&bad_tag, &root, &m).unwrap_err();
    assert!(matches!(err, ResourceValueError::NotPackageResource(_)));

    let missing_path = RuntimeValue::Record(vec![(
        "tag".into(),
        RuntimeValue::String("package-resource".into()),
    )]);
    let err = materialize_package_resource(&missing_path, &root, &m).unwrap_err();
    assert!(matches!(err, ResourceValueError::NotPackageResource(_)));

    let unlisted = eval_source(r#"(val main (resource "images/missing.png"))"#).unwrap();
    let err = materialize_package_resource(&unlisted, &root, &m).unwrap_err();
    assert!(matches!(err, ResourceValueError::Resolve(_)));
    assert!(!err.to_string().is_empty());
}

#[test]
fn auto_materialize_package_resource_tree_and_entry_helper() {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use reciplexa_eval::{eval_source, RuntimeValue};
    use reciplexa_package::{
        find_enclosing_package_root, materialize_package_resources_in_tree,
        maybe_materialize_package_resources_for_entry, parse_rpxm,
    };

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.tmp")
        .join(format!("pkg-auto-mat-{nanos}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("resources/images")).unwrap();
    fs::write(root.join("resources/images/logo.png"), b"png").unwrap();
    fs::write(
        root.join("package.rpxm"),
        r#"(package demo
  format-version 1
  version "1.0.0"
  (public-modules main)
  (resources "images/logo.png"))"#,
    )
    .unwrap();
    let entry = root.join("main.rpx");
    fs::write(&entry, "(val main 1)").unwrap();

    assert_eq!(
        find_enclosing_package_root(&entry).as_deref(),
        Some(root.as_path())
    );

    let m = parse_rpxm(&fs::read_to_string(root.join("package.rpxm")).unwrap()).unwrap();
    let listed = eval_source(r#"(val main (resource "images/logo.png"))"#).unwrap();
    let unlisted = eval_source(r#"(val main (resource "images/missing.png"))"#).unwrap();
    let nested = RuntimeValue::Record(vec![
        ("ok".into(), listed.clone()),
        ("bad".into(), unlisted.clone()),
    ]);
    let out = materialize_package_resources_in_tree(&nested, &root, &m);
    let RuntimeValue::Record(fields) = &out else {
        panic!("record");
    };
    let ok = fields.iter().find(|(k, _)| k == "ok").unwrap().1.clone();
    let RuntimeValue::Record(okf) = ok else {
        panic!("ok");
    };
    let resolved = okf
        .iter()
        .find(|(k, _)| k == "resolved-path")
        .and_then(|(_, v)| match v {
            RuntimeValue::String(s) => Some(s.as_str()),
            _ => None,
        })
        .expect("resolved-path");
    assert!(resolved
        .replace('\\', "/")
        .ends_with("resources/images/logo.png"));
    assert!(okf.iter().any(|(k, v)| {
        k == "resource-id"
            && matches!(v, RuntimeValue::String(s) if s == "demo@1.0.0/images/logo.png")
    }));
    assert!(okf.iter().any(|(k, v)| {
        k == "content-hash" && matches!(v, RuntimeValue::String(s) if s.starts_with("sha256:"))
    }));
    assert!(okf.iter().any(|(k, v)| {
        k == "effect" && matches!(v, RuntimeValue::String(s) if s == "Resource")
    }));

    let bad = fields.iter().find(|(k, _)| k == "bad").unwrap().1.clone();
    let RuntimeValue::Record(badf) = bad else {
        panic!("bad");
    };
    assert!(!badf.iter().any(|(k, _)| k == "resolved-path"));

    // Entry helper finds package root and materializes.
    let via_entry = maybe_materialize_package_resources_for_entry(&listed, &entry);
    let RuntimeValue::Record(ef) = via_entry else {
        panic!("entry");
    };
    assert!(ef.iter().any(|(k, _)| k == "resolved-path"));

    // Outside any package → unchanged.
    let orphan = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.tmp")
        .join(format!("pkg-orphan-{nanos}/x.rpx"));
    fs::create_dir_all(orphan.parent().unwrap()).unwrap();
    fs::write(&orphan, "x").unwrap();
    let same = maybe_materialize_package_resources_for_entry(&listed, &orphan);
    let RuntimeValue::Record(sf) = same else {
        panic!("orphan");
    };
    assert!(!sf.iter().any(|(k, _)| k == "resolved-path"));
}
