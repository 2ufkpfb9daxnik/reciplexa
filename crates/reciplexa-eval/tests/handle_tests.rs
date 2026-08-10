//! EFF-001: shallow handle / one-shot resume v0.

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
fn oneshot_resume_becomes_handle_result() {
    let v =
        eval_source(r#"(val main (handle log (fn (msg k) (k 42)) (perform log "hi")))"#).unwrap();
    assert_eq!(v, RuntimeValue::Number(42.0));
}

#[test]
fn unmatched_perform_falls_through_to_host() {
    let expr = elaborate_source(r#"(val main (perform log "x"))"#).unwrap();
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Unit);
}

#[test]
fn oneshot_resume_aborts_rest_of_handler() {
    // Shallow one-shot: applying resume unwinds the handler; trailing exprs are skipped.
    let v =
        eval_source(r#"(val main (handle log (fn (msg k) (seq (k 1) 99)) (perform log "hi")))"#)
            .unwrap();
    assert_eq!(v, RuntimeValue::Number(1.0));
}
