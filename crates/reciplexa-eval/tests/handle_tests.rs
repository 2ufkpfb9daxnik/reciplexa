//! EFF-001: deep one-shot resume for handle.

use reciplexa_core::elaborate_source;
use reciplexa_core::expr::CoreExpr;
use reciplexa_eval::{eval_expr, eval_source, RuntimeValue, UnitHost};
use std::collections::HashMap;

#[test]
fn elaborate_handle_and_perform() {
    let expr =
        elaborate_source(r#"(val main (handle log (fn (msg) msg) (perform log "hi")))"#).unwrap();
    assert!(matches!(
        expr,
        CoreExpr::Let {
            ref value,
            ..
        } if matches!(**value, CoreExpr::Handle { ref op, .. } if op == "log")
    ));
}

#[test]
fn shallow_handle_abort_returns_handler_result() {
    let v =
        eval_source(r#"(val main (handle log (fn (msg) msg) (perform log "caught")))"#).unwrap();
    assert_eq!(v, RuntimeValue::String("caught".into()));
}

#[test]
fn oneshot_resume_becomes_perform_result() {
    let v =
        eval_source(r#"(val main (handle log (fn (msg k) (k 42)) (perform log "hi")))"#).unwrap();
    assert_eq!(v, RuntimeValue::Int(42));
}

#[test]
fn unmatched_perform_falls_through_to_host() {
    let expr = elaborate_source(r#"(val main (perform log "x"))"#).unwrap();
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Unit);
}

#[test]
fn oneshot_resume_aborts_rest_of_handler() {
    // One-shot: applying resume transfers to the body continuation; trailing
    // handler exprs are skipped.
    let v =
        eval_source(r#"(val main (handle log (fn (msg k) (seq (k 1) 99)) (perform log "hi")))"#)
            .unwrap();
    assert_eq!(v, RuntimeValue::Int(1));
}

#[test]
fn with_installs_handler_value() {
    let v = eval_source(
        r#"
(val h (handler log (fn (msg) msg)))
(val main (with h (perform log "ok")))
"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::String("ok".into()));
}

#[test]
fn with_inline_handler_and_resume() {
    let v =
        eval_source(r#"(val main (with (handler ask (fn (_ k) (k 3))) (seq (perform ask 0) 10)))"#)
            .unwrap();
    assert_eq!(v, RuntimeValue::Int(10));
}

#[test]
fn deep_resume_continues_body_after_perform() {
    // Deep: resume plugs the value into perform, then seq continues to 99.
    let v =
        eval_source(r#"(val main (handle log (fn (msg k) (k 42)) (seq (perform log "hi") 99)))"#)
            .unwrap();
    assert_eq!(v, RuntimeValue::Int(99));
}

#[test]
fn deep_resume_with_modified_value_in_let() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 7))
    (let ((x (perform ask 0)))
      x)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(7));
}

#[test]
fn raise_lowers_to_failure_perform() {
    let expr = elaborate_source(r#"(val main (raise "boom"))"#).unwrap();
    assert!(matches!(
        expr,
        CoreExpr::Let {
            ref value,
            ..
        } if matches!(**value, CoreExpr::Perform { ref op, .. } if op == "failure")
    ));
}

#[test]
fn handle_failure_catches_raise() {
    let v = eval_source(r#"(val main (handle failure (fn (err) err) (raise "caught")))"#).unwrap();
    assert_eq!(v, RuntimeValue::String("caught".into()));
}

#[test]
fn handle_failure_rejects_resume_param() {
    let err =
        eval_source(r#"(val main (handle failure (fn (err k) err) (raise "x")))"#).unwrap_err();
    assert!(
        err.message.contains("Failure") || err.message.contains("resume"),
        "unexpected: {}",
        err.message
    );
}

#[test]
fn or_raise_ok_and_err_paths() {
    let src = r#"
(data result (ok int) (err str))
(val main
  (seq
    (or-raise (ok 3))
    (handle failure (fn (e) e)
      (or-raise (err "boom")))))
"#;
    let v = eval_source(src).unwrap();
    assert_eq!(v, RuntimeValue::String("boom".into()));
}

#[test]
fn as_result_wraps_success_and_failure() {
    let ok = eval_source(r#"(val main (as-result (fn () 42)))"#).unwrap();
    assert!(matches!(
        ok,
        RuntimeValue::Variant {
            tag,
            payload: Some(_),
        } if tag == "ok"
    ));

    let err = eval_source(r#"(val main (as-result (fn () (raise "nope"))))"#).unwrap();
    assert!(matches!(
        err,
        RuntimeValue::Variant {
            tag,
            payload: Some(_),
        } if tag == "err"
    ));
}

#[test]
fn raise_without_handler_surfaces_unhandled_failure() {
    let err = eval_source(r#"(val main (raise "boom"))"#).unwrap_err();
    assert!(err.message.contains("unhandled failure"));
}

#[test]
fn never_type_allows_raise_in_if() {
    let ty =
        reciplexa_core::typecheck_language_source(r#"(val main (if true 1 (raise "x")))"#).unwrap();
    assert!(matches!(
        ty,
        reciplexa_core::ty::CoreType::Int
            | reciplexa_core::ty::CoreType::F64
            | reciplexa_core::ty::CoreType::Number
    ));
}

#[test]
fn forward_delegates_to_outer_handler() {
    let v = eval_source(
        r#"(val main
  (handle log (fn (msg) (seq (perform log "inner") msg))
    (handle log (fn (msg k) (forward k))
      (perform log "outer"))))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::String("outer".into()));
}
