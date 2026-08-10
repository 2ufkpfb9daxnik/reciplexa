//! TYP-001: language-kernel typecheck via Core infer (not document surface).

use reciplexa_core::{typecheck_language_source, CoreType};

#[test]
fn identity_app_types_as_number() {
    let ty = typecheck_language_source("(val main ((fn (x) x) 1))").unwrap();
    assert_eq!(ty, CoreType::Number);
}

#[test]
fn dynamic_unifies_as_gradual_stub() {
    use reciplexa_core::unify::{unify, Subst};
    let mut s = Subst::new();
    assert!(unify(&CoreType::Dynamic, &CoreType::Number, &mut s).is_ok());
    assert!(unify(&CoreType::String, &CoreType::Dynamic, &mut s).is_ok());
}

#[test]
fn match_non_exhaustive_errors() {
    let err = typecheck_language_source(
        r#"
(data option (none) (some x))
(val main (match (some 1) (some x -> x)))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("non-exhaustive"),
        "unexpected: {}",
        err.message
    );
}

#[test]
fn match_exhaustive_option_ok() {
    let ty = typecheck_language_source(
        r#"
(data option (none) (some x))
(val main (match (some 1) (none -> 0) (some x -> x)))
"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::Number);
}

#[test]
fn match_unreachable_after_wildcard_errors() {
    let err = typecheck_language_source(
        r#"
(data option (none) (some x))
(val main (match (some 1) (_ -> 0) (some x -> x)))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("unreachable"),
        "unexpected: {}",
        err.message
    );
}

#[test]
fn match_unreachable_duplicate_ctor_errors() {
    let err = typecheck_language_source(
        r#"
(data option (none) (some x))
(val main (match (some 1) (none -> 0) (some x -> x) (none -> 1)))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("unreachable"),
        "unexpected: {}",
        err.message
    );
}

#[test]
fn unit_literal_types_as_unit() {
    let ty = typecheck_language_source("(val main unit)").unwrap();
    assert_eq!(ty, CoreType::Unit);
}

#[test]
fn record_field_types() {
    let ty = typecheck_language_source(
        r#"
(val report (record (title "Report") (page-count 10)))
(val main (field report title))
"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::String);
}

#[test]
fn perform_adds_effect_to_fun() {
    use reciplexa_core::unify::Subst;
    use reciplexa_core::{elaborate_source, infer_with_effects, TypeEnv};
    use reciplexa_source::range::TextRange;

    let expr = elaborate_source(r#"(val main (fn () (perform log "hi")))"#).unwrap();
    // peel let to get the fn
    let reciplexa_core::CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    let mut subst = Subst::new();
    let (ty, residual) =
        infer_with_effects(&value, &TypeEnv::new(), &mut subst, TextRange::EMPTY).unwrap();
    assert!(
        residual.ops.is_empty(),
        "lambda suspends effects: {residual:?}"
    );
    match subst.apply(&ty) {
        CoreType::Fun { effects, .. } => {
            assert_eq!(effects.ops, vec!["log".to_string()]);
        }
        other => panic!("expected Fun, got {other:?}"),
    }
}

#[test]
fn handle_removes_effect_from_residual() {
    use reciplexa_core::elaborate_source;
    use reciplexa_core::unify::Subst;
    use reciplexa_core::{infer_with_effects, typecheck_language_source, TypeEnv};
    use reciplexa_source::range::TextRange;

    let src = r#"(val main (handle log (fn (msg) msg) (perform log "ok")))"#;
    let ty = typecheck_language_source(src).unwrap();
    assert_eq!(ty, CoreType::String);

    let expr = elaborate_source(src).unwrap();
    let reciplexa_core::CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    let mut subst = Subst::new();
    let (_ty, residual) =
        infer_with_effects(&value, &TypeEnv::new(), &mut subst, TextRange::EMPTY).unwrap();
    assert!(
        !residual.ops.iter().any(|o| o == "log"),
        "handle should remove log: {residual:?}"
    );
}
