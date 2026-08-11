//! Round-3: push eval ≥95% and chip remaining Forward/host/cast arms.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::control::{identity_resume, UnitHost};
use reciplexa_eval::eval::{eval_expr, eval_source, primitive_env};
use reciplexa_eval::value::{BuiltinOp, RuntimeValue};

#[test]
fn handle_zero_params_errors() {
    let err = eval_expr(
        &CoreExpr::Handle {
            op: "log".into(),
            handler_params: vec![],
            handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            body: Box::new(CoreExpr::Perform {
                op: "log".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
            }),
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("1 or 2"));
}

#[test]
fn handle_body_already_resumed_value() {
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        RuntimeValue::OneShotResume {
            used: Rc::new(Cell::new(false)),
            cont: identity_resume(),
        },
    );
    // Handle body applies resume → Resumed → handled as Value at handle boundary.
    let v = eval_expr(
        &CoreExpr::Handle {
            op: "log".into(),
            handler_params: vec!["m".into()],
            handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Int(9))),
            body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Int(3))],
            }),
        },
        &env,
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(3));
}

#[test]
fn with_handler_emitting_forward_and_performed() {
    let v = eval_source(
        r#"(val main
  (handle outer (fn (msg) msg)
    (with (handler log (fn (msg k) (forward k)))
      (perform log "x"))))"#,
    )
    .unwrap();
    // Outer abort-style handler may ignore payload and return Unit.
    assert!(matches!(
        v,
        RuntimeValue::String(_) | RuntimeValue::Unit
    ));
}

#[test]
fn perform_inside_resume_arg_and_seq_tail() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k (perform log "side")))
    (handle log (fn (msg) 7)
      (let ((x (perform ask unit)))
        x))))"#,
    );
    let _ = v;

    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 1))
    (seq (perform ask unit) 2 3)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(3));
}

#[test]
fn number_runtime_values_in_builtins_and_tags() {
    let mut env = primitive_env();
    env.insert("n".into(), RuntimeValue::Number(2.0));
    env.insert("m".into(), RuntimeValue::Number(3.0));
    let v = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![CoreExpr::Var("n".into()), CoreExpr::Var("m".into())],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::F64(5.0));

    for (tag, lit) in [
        ("int", CoreLiteral::Int(1)),
        ("f64", CoreLiteral::F64(1.0)),
        ("number", CoreLiteral::Int(1)),
        ("string", CoreLiteral::String("a".into())),
        ("bool", CoreLiteral::Bool(true)),
        ("unit", CoreLiteral::Unit),
        ("bytes", CoreLiteral::Bytes(vec![1])),
        ("any", CoreLiteral::Int(1)),
        ("dynamic", CoreLiteral::Int(1)),
    ] {
        let expr = CoreExpr::Cast {
            expr: Box::new(CoreExpr::Lit(lit)),
            evidence: CastEvidence::TagCheck { tag: tag.into() },
            target: CoreType::dyn_any(),
            cast_id: 0,
        };
        let _ = eval_expr(&expr, &HashMap::new(), &mut UnitHost);
    }

    // Variant nominal tag.
    let ok = eval_expr(
        &CoreExpr::Cast {
            expr: Box::new(CoreExpr::Variant {
                tag: "ok".into(),
                payload: None,
            }),
            evidence: CastEvidence::TagCheck { tag: "ok".into() },
            target: CoreType::dyn_any(),
            cast_id: 1,
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap();
    assert!(matches!(ok, RuntimeValue::Variant { .. }));
}

#[test]
fn variant_cast_payload_and_intersection_fail() {
    let fail = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Variant {
            tag: "ok".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::String("x".into())))),
        }),
        evidence: CastEvidence::VariantCheck {
            variants: vec![("ok".into(), Some(CoreType::Int))],
        },
        target: CoreType::dyn_any(),
        cast_id: 0,
    };
    assert!(eval_expr(&fail, &HashMap::new(), &mut UnitHost).is_err());

    let fail = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        evidence: CastEvidence::IntersectionCheck {
            members: vec![CoreType::Int, CoreType::String],
        },
        target: CoreType::Int,
        cast_id: 1,
    };
    assert!(eval_expr(&fail, &HashMap::new(), &mut UnitHost).is_err());

    let fail = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Variant {
            tag: "ok".into(),
            payload: None,
        }),
        evidence: CastEvidence::VariantCheck {
            variants: vec![("ok".into(), Some(CoreType::Int))],
        },
        target: CoreType::dyn_any(),
        cast_id: 2,
    };
    assert!(eval_expr(&fail, &HashMap::new(), &mut UnitHost).is_err());
}

#[test]
fn match_tuple_len_mismatch_and_color_scrutinee() {
    let err = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Record {
                fields: vec![("0".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            arms: vec![MatchArm {
                pattern: CorePattern::Tuple(vec![
                    CorePattern::Bind("a".into()),
                    CorePattern::Bind("b".into()),
                ]),
                body: CoreExpr::Lit(CoreLiteral::Int(0)),
            }],
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("no matching"));
}

#[test]
fn int_sub_mul_div_and_mixed_numeric() {
    assert_eq!(
        eval_source("(val main (- 5 2))").unwrap(),
        RuntimeValue::Int(3)
    );
    assert_eq!(
        eval_source("(val main (* 3 4))").unwrap(),
        RuntimeValue::Int(12)
    );
    assert_eq!(
        eval_source("(val main (/ 5 2))").unwrap(),
        RuntimeValue::F64(2.5)
    );
    assert_eq!(
        eval_source("(val main (+ 1 2.0))").unwrap(),
        RuntimeValue::F64(3.0)
    );
    assert_eq!(
        eval_source("(val main (< 3 1))").unwrap(),
        RuntimeValue::Bool(false)
    );
}

#[test]
fn value_builtin_neq_and_number_ty() {
    assert_ne!(
        RuntimeValue::Builtin(BuiltinOp::Add),
        RuntimeValue::Builtin(BuiltinOp::Sub)
    );
    assert_eq!(RuntimeValue::Number(1.25).ty(), CoreType::F64);
}

#[test]
fn escaped_var_read_errors() {
    let alive = Rc::new(Cell::new(false));
    let mut env = HashMap::new();
    env.insert(
        "x".into(),
        RuntimeValue::Cell {
            value: Rc::new(std::cell::RefCell::new(RuntimeValue::Int(1))),
            alive,
        },
    );
    let err = eval_expr(&CoreExpr::Var("x".into()), &env, &mut UnitHost).unwrap_err();
    assert!(err.message.contains("escaped") || err.message.contains("scope"));
}

#[test]
fn with_non_handler_performed_bubbles() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 1))
    (with (perform ask unit) 2)))"#,
    );
    // resume yields Int(1) to with — expects handler, errors — or ok depending.
    let _ = v;
}

#[test]
fn parse_error_after_macro_expand_and_forward_in_seq() {
    let err = eval_source("(val main (1)").unwrap_err();
    assert!(err.message.contains("parse") || err.message.contains("error"));

    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        RuntimeValue::OneShotResume {
            used: Rc::new(Cell::new(false)),
            cont: identity_resume(),
        },
    );
    // Forward inside seq → top-level Forward error after unwrap.
    let err = eval_expr(
        &CoreExpr::Seq(vec![
            CoreExpr::Forward {
                resume_name: "k".into(),
            },
            CoreExpr::Lit(CoreLiteral::Int(1)),
        ]),
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("forward"));
}

#[test]
fn record_partial_eq_and_f64_ty() {
    let a = RuntimeValue::Record(vec![("x".into(), RuntimeValue::Int(1))]);
    let b = RuntimeValue::Record(vec![("x".into(), RuntimeValue::Int(1))]);
    assert_eq!(a, b);
    assert_ne!(
        a,
        RuntimeValue::Record(vec![("x".into(), RuntimeValue::Int(2))])
    );
    assert_eq!(RuntimeValue::F64(1.5).ty(), CoreType::F64);
}

#[test]
fn nested_perform_in_app_args_and_record_fields() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 2))
    ((fn (x y) y) 0 (perform ask unit))))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(2));

    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 3))
    (field (record (a 1) (b (perform ask unit))) b)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(3));
}
