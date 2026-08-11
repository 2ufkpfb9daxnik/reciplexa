//! Round-8 eval: denser Cont other-arm cascades through app/record/seq/match.

use std::cell::{Cell, RefCell};
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

struct CaptureHost {
    last_op: Option<String>,
}

impl EffectHost for CaptureHost {
    fn perform(&mut self, op: &str, arg: RuntimeValue) -> EvalResult {
        self.last_op = Some(op.to_string());
        Ok(arg)
    }
}

#[test]
fn deep_compound_forward_and_resumed_in_args() {
    let src = r#"
(val main
  (handle ask (fn (m k)
    (handle log (fn (msg k2)
      (k2 (k m)))
    (perform log "inner")))
  (perform ask "q")))
"#;
    let v = eval_source(src).unwrap();
    assert_eq!(v, RuntimeValue::String("q".into()));

    let mut env = HashMap::new();
    env.insert("k".into(), oneshot(Rc::new(|v, _| Ok(Outcome::Resumed(v)))));
    env.insert(
        "f".into(),
        RuntimeValue::Closure {
            params: vec!["x".into(), "y".into()],
            body: CoreExpr::Var("x".into()),
            env: Rc::new(RefCell::new(HashMap::new())),
        },
    );
    let v = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("f".into())),
            args: vec![
                CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("k".into())),
                    args: vec![CoreExpr::Lit(CoreLiteral::Int(7))],
                },
                CoreExpr::Lit(CoreLiteral::Int(9)),
            ],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(7));
}

#[test]
fn record_seq_match_cast_cont_other_arms() {
    let mut env = HashMap::new();
    env.insert("k".into(), oneshot(Rc::new(|v, _| Ok(Outcome::Resumed(v)))));
    let v = eval_expr(
        &CoreExpr::Record {
            fields: vec![(
                "a".into(),
                CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("k".into())),
                    args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
                },
            )],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(1));

    let mut env = HashMap::new();
    env.insert("k".into(), oneshot(Rc::new(|_, _| Ok(Outcome::Forward))));
    let err = eval_expr(
        &CoreExpr::Seq(vec![CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
        }]),
        &env,
        &mut UnitHost,
    );
    assert!(err.is_err());

    let env = primitive_env();
    let mut host = CaptureHost { last_op: None };
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Perform {
            op: "ask".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
        }),
        arms: vec![MatchArm {
            pattern: CorePattern::Wildcard,
            body: CoreExpr::Lit(CoreLiteral::Int(42)),
        }],
    };
    let v = eval_expr(&expr, &env, &mut host).unwrap();
    assert_eq!(v, RuntimeValue::Int(42));
    assert_eq!(host.last_op.as_deref(), Some("ask"));

    let mut env = HashMap::new();
    env.insert("k".into(), oneshot(Rc::new(|v, _| Ok(Outcome::Resumed(v)))));
    let v = eval_expr(
        &CoreExpr::Cast {
            expr: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Int(3))],
            }),
            evidence: CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 0,
        },
        &env,
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(3));

    let mut env = HashMap::new();
    env.insert("k".into(), oneshot(Rc::new(|v, _| Ok(Outcome::Resumed(v)))));
    let _ = eval_expr(
        &CoreExpr::TryCast {
            expr: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Int(4))],
            }),
            target: CoreType::Int,
            cast_id: 1,
        },
        &env,
        &mut UnitHost,
    );
}

#[test]
fn handle_failure_and_forward_compound() {
    let src = r#"
(val main
  (handle failure (fn (e) e)
    (raise "boom")))
"#;
    let v = eval_source(src).unwrap();
    assert_eq!(v, RuntimeValue::String("boom".into()));

    let src = r#"
(val main
  (handle ask (fn (m k)
    (forward k))
  (perform ask "x")))
"#;
    let mut host = CaptureHost { last_op: None };
    let v = eval_source_with_host(src, &mut host).unwrap();
    assert_eq!(v, RuntimeValue::String("x".into()));
    assert_eq!(host.last_op.as_deref(), Some("ask"));

    let src = r#"
(val main
  (handle ask (fn (m k) (k m))
    (let ((a (perform ask "a")))
      (if true a "b"))))
"#;
    let v = eval_source(src).unwrap();
    assert_eq!(v, RuntimeValue::String("a".into()));
}
