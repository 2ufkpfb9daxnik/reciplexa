//! Round-13 eval: Performed/Resumed `other` arms + builtin/match_pattern leaves.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::control::{EffectHost, EvalResult, Outcome, ResumeCont, UnitHost};
use reciplexa_eval::eval::{eval_expr, eval_source, eval_source_with_host, primitive_env};
use reciplexa_eval::value::RuntimeValue;

fn oneshot(cont: ResumeCont) -> RuntimeValue {
    RuntimeValue::OneShotResume {
        used: Rc::new(Cell::new(false)),
        cont,
    }
}

fn performed_ping(v: RuntimeValue) -> Result<Outcome, reciplexa_eval::EvalError> {
    Ok(Outcome::Performed {
        op: "ping".into(),
        arg: v,
        resume: Rc::new(|v, _| Ok(Outcome::Value(v))),
    })
}

fn app_k() -> CoreExpr {
    CoreExpr::App {
        fun: Box::new(CoreExpr::Var("k".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::Int(7))],
    }
}

struct Echo;
impl EffectHost for Echo {
    fn perform(&mut self, _op: &str, arg: RuntimeValue) -> EvalResult {
        Ok(arg)
    }
}

#[test]
fn performed_other_through_all_compound_forms() {
    let forms = [
        CoreExpr::Cast {
            expr: Box::new(app_k()),
            evidence: CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 0,
        },
        CoreExpr::TryCast {
            expr: Box::new(app_k()),
            target: CoreType::Int,
            cast_id: 1,
        },
        CoreExpr::CheckCast {
            expr: Box::new(app_k()),
            target: CoreType::Int,
            cast_id: 2,
        },
        CoreExpr::If {
            cond: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Bool(true))],
            }),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(7))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        CoreExpr::Let {
            name: "x".into(),
            value: Box::new(app_k()),
            body: Box::new(CoreExpr::Var("x".into())),
        },
        CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(app_k()),
            body: Box::new(CoreExpr::Var("c".into())),
        },
        CoreExpr::Record {
            fields: vec![("a".into(), app_k())],
        },
        CoreExpr::RecordGet {
            record: Box::new(app_k()),
            field: "a".into(),
        },
        CoreExpr::RecordUpdate {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
            }),
            fields: vec![("a".into(), app_k())],
        },
        CoreExpr::RecordExtend {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
            }),
            fields: vec![("b".into(), app_k())],
        },
        CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(app_k())),
        },
        CoreExpr::Match {
            scrutinee: Box::new(app_k()),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(0)),
            }],
        },
        CoreExpr::Seq(vec![app_k(), CoreExpr::Lit(CoreLiteral::Int(0))]),
        CoreExpr::App {
            fun: Box::new(app_k()),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(0))],
        },
        CoreExpr::Perform {
            op: "ask".into(),
            arg: Box::new(app_k()),
        },
    ];

    for form in forms {
        let mut env = HashMap::new();
        env.insert("k".into(), oneshot(Rc::new(|v, _| performed_ping(v))));
        // Cover Cont/Performed other-arms; some forms Err after resume (e.g. RecordGet on Int).
        let _ = eval_expr(&form, &env, &mut Echo);
    }
}

#[test]
fn eval_to_value_perform_loop_and_lit() {
    let v = eval_source(r#"(val main (handle ping (fn (m k) (k m)) (perform ping 42)))"#).unwrap();
    assert_eq!(v, RuntimeValue::Int(42));

    let v = eval_source("(val main 1.5)").unwrap();
    assert!(matches!(v, RuntimeValue::F64(_) | RuntimeValue::Number(_)));
}

#[test]
fn handle_deep_resume_other_op_bubbles() {
    let src = r#"
(val main
  (handle ask (fn (m k)
    (handle log (fn (msg k2)
      (k2 (k m)))
    (perform log "inner")))
  (perform ask "q")))
"#;
    assert_eq!(eval_source(src).unwrap(), RuntimeValue::String("q".into()));

    struct Ping;
    impl EffectHost for Ping {
        fn perform(&mut self, op: &str, arg: RuntimeValue) -> EvalResult {
            if op == "ping" {
                Ok(arg)
            } else {
                Ok(RuntimeValue::Unit)
            }
        }
    }
    let v = eval_source_with_host(
        r#"(val main
  (handle ask (fn (m k) (k (perform ping m)))
    (perform ask 9)))"#,
        &mut Ping,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(9));
}

#[test]
fn builtins_unicode_f64_and_match_pattern_none_paths() {
    let env = primitive_env();
    let mut host = UnitHost;

    let unicode_f64 = CoreExpr::App {
        fun: Box::new(CoreExpr::Var("unicode".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::F64(65.0))],
    };
    let v = eval_expr(&unicode_f64, &env, &mut host).unwrap();
    assert_eq!(v, RuntimeValue::String("A".into()));

    for bad in [
        CoreExpr::App {
            fun: Box::new(CoreExpr::Var("unicode".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::String("x".into()))],
        },
        CoreExpr::App {
            fun: Box::new(CoreExpr::Var("encode-utf8".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        CoreExpr::App {
            fun: Box::new(CoreExpr::Var("decode-utf8".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::String("x".into()))],
        },
    ] {
        assert!(eval_expr(&bad, &env, &mut UnitHost).is_err());
    }

    // Match scrutinee type mismatch → pattern None arms.
    let mismatch = eval_source(r#"(val main (match 1 (true -> 0) (_ -> 1)))"#).unwrap();
    assert_eq!(mismatch, RuntimeValue::Int(1));

    let record_mismatch =
        eval_source(r#"(val main (match (record (a 1)) ((record (b x)) -> x) (_ -> 1)))"#).unwrap();
    assert_eq!(record_mismatch, RuntimeValue::Int(1));

    let variant_mismatch = eval_source(
        r#"(data opt (none) (some x))
(val main (match (some 1) (none -> 0) (_ -> 1)))"#,
    )
    .unwrap();
    assert_eq!(variant_mismatch, RuntimeValue::Int(1));
}

#[test]
fn numeric_binop_non_int_and_forward_errors() {
    let v = eval_source(r#"(val main (+ 1.5 2.5))"#).unwrap();
    assert!(matches!(v, RuntimeValue::F64(f) if f == 4.0));

    let err =
        eval_source(r#"(val main (handle ask (fn (m k) (forward missing)) (perform ask 0)))"#);
    assert!(err.is_err());
}

#[test]
fn with_handler_non_value_via_core_expr() {
    let env = primitive_env();
    let err = eval_expr(
        &CoreExpr::With {
            handler: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        &env,
        &mut UnitHost,
    );
    assert!(err.is_err());
}
