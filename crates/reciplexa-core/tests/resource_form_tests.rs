//! PKG §21.14 light `(resource "rel")` → package-resource record.

use reciplexa_core::elaborate::elaborate_source;
use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_core::typecheck_language_source;

fn record_field<'a>(expr: &'a CoreExpr, label: &str) -> Option<&'a CoreExpr> {
    let CoreExpr::Record { fields } = expr else {
        return None;
    };
    fields
        .iter()
        .find(|(k, _)| k == label)
        .map(|(_, v)| v)
}

fn string_lit(expr: &CoreExpr) -> Option<&str> {
    match expr {
        CoreExpr::Lit(CoreLiteral::String(s)) => Some(s.as_str()),
        _ => None,
    }
}

#[test]
fn elaborates_resource_to_package_resource_record() {
    let expr = elaborate_source(r#"(val main (resource "styles/default.css"))"#).unwrap();
    // Top-level val → Let; value is the resource record.
    let CoreExpr::Let { value, .. } = expr else {
        panic!("expected let: {expr:?}");
    };
    assert_eq!(
        string_lit(record_field(&value, "tag").unwrap()),
        Some("package-resource")
    );
    assert_eq!(
        string_lit(record_field(&value, "path").unwrap()),
        Some("styles/default.css")
    );
    assert_eq!(
        string_lit(record_field(&value, "note").unwrap()),
        Some("resolve at package load")
    );
}

#[test]
fn typechecks_resource_as_record() {
    let ty = typecheck_language_source(r#"(val main (resource "images/logo.png"))"#).unwrap();
    assert!(
        matches!(ty, reciplexa_core::CoreType::Record { .. }),
        "expected record type, got {ty:?}"
    );
}

#[test]
fn rejects_resource_arity_and_non_string() {
    let err = elaborate_source(r#"(val main (resource))"#).unwrap_err();
    assert!(err.message.contains("exactly one string"));

    let err = elaborate_source(r#"(val main (resource "a" "b"))"#).unwrap_err();
    assert!(err.message.contains("exactly one string"));

    let err = elaborate_source(r#"(val main (resource 1))"#).unwrap_err();
    assert!(err.message.contains("string literal"));

    let err = elaborate_source(r#"(val main (resource ""))"#).unwrap_err();
    assert!(err.message.contains("empty"));
}
