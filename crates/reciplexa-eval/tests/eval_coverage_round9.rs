//! Round-9 eval: Cont Resumed/Forward bubble through compound forms (no Handle).

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::control::{EffectHost, EvalResult, Outcome, ResumeCont, UnitHost};
use reciplexa_eval::eval::{eval_expr, primitive_env};
use reciplexa_eval::value::RuntimeValue;

fn oneshot(cont: ResumeCont) -> RuntimeValue {
    RuntimeValue::OneShotResume {
        used: Rc::new(Cell::new(false)),
        cont,
    }
}

fn app_k(arg: i128) -> CoreExpr {
    CoreExpr::App {
        fun: Box::new(CoreExpr::Var("k".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::Int(arg))],
    }
}

#[test]
fn resumed_bubbles_through_compound_forms() {
    let forms = [
        CoreExpr::Let {
            name: "x".into(),
            value: Box::new(app_k(1)),
            body: Box::new(CoreExpr::Var("x".into())),
        },
        CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(app_k(2)),
            body: Box::new(CoreExpr::Var("c".into())),
        },
        CoreExpr::Set {
            name: "cell".into(),
            value: Box::new(app_k(3)),
        },
        CoreExpr::If {
            cond: Box::new(app_k(4)),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        CoreExpr::Record {
            fields: vec![("a".into(), app_k(5))],
        },
        CoreExpr::RecordGet {
            record: Box::new(app_k(6)),
            field: "a".into(),
        },
        CoreExpr::RecordUpdate {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
            }),
            fields: vec![("a".into(), app_k(7))],
        },
        CoreExpr::RecordExtend {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
            }),
            fields: vec![("b".into(), app_k(8))],
        },
        CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(app_k(9))),
        },
        CoreExpr::Match {
            scrutinee: Box::new(app_k(10)),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(0)),
            }],
        },
        CoreExpr::Cast {
            expr: Box::new(app_k(11)),
            evidence: CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 0,
        },
        CoreExpr::TryCast {
            expr: Box::new(app_k(12)),
            target: CoreType::Int,
            cast_id: 1,
        },
        CoreExpr::CheckCast {
            expr: Box::new(app_k(13)),
            target: CoreType::Int,
            cast_id: 2,
        },
        CoreExpr::Seq(vec![app_k(14), CoreExpr::Lit(CoreLiteral::Int(0))]),
        CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Var("x".into())),
            }),
            args: vec![app_k(15)],
        },
        CoreExpr::App {
            fun: Box::new(app_k(16)),
            args: vec![],
        },
        CoreExpr::With {
            handler: Box::new(app_k(17)),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
    ];

    for form in forms {
        let mut env = HashMap::new();
        env.insert(
            "cell".into(),
            RuntimeValue::Cell {
                value: Rc::new(RefCell::new(RuntimeValue::Int(0))),
                alive: Rc::new(Cell::new(true)),
            },
        );
        env.insert("k".into(), oneshot(Rc::new(|v, _| Ok(Outcome::Resumed(v)))));
        let v = eval_expr(&form, &env, &mut UnitHost).unwrap();
        assert!(matches!(v, RuntimeValue::Int(_)), "got {v:?}");
    }
}

#[test]
fn forward_bubbles_as_top_level_error() {
    let mut env = HashMap::new();
    env.insert("k".into(), oneshot(Rc::new(|_, _| Ok(Outcome::Forward))));
    let err = eval_expr(
        &CoreExpr::Let {
            name: "x".into(),
            value: Box::new(app_k(1)),
            body: Box::new(CoreExpr::Var("x".into())),
        },
        &env,
        &mut UnitHost,
    );
    assert!(err.is_err());
}

#[test]
fn deep_resume_inner_returns_resumed_performed() {
    // Cont other-arm: after perform resume, nested oneshot returns Resumed/Performed.
    struct Echo;
    impl EffectHost for Echo {
        fn perform(&mut self, _op: &str, arg: RuntimeValue) -> EvalResult {
            Ok(arg)
        }
    }

    let mut env = primitive_env();
    env.insert("k".into(), oneshot(Rc::new(|v, _| Ok(Outcome::Resumed(v)))));
    let form = CoreExpr::Let {
        name: "x".into(),
        value: Box::new(CoreExpr::Seq(vec![
            CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            },
            app_k(42),
        ])),
        body: Box::new(CoreExpr::Var("x".into())),
    };
    // Top-level host resume drives Cont; nested oneshot Resumed should surface.
    let v = eval_expr(&form, &env, &mut Echo).unwrap();
    assert_eq!(v, RuntimeValue::Int(42));

    let mut env = primitive_env();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|v, _host| {
            // Re-perform during Cont resume → hits Performed other arm.
            Ok(Outcome::Performed {
                op: "ping".into(),
                arg: v,
                resume: Rc::new(|v, _| Ok(Outcome::Value(v))),
            })
        })),
    );
    let form = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Seq(vec![
            CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            },
            app_k(7),
        ])),
        arms: vec![MatchArm {
            pattern: CorePattern::Wildcard,
            body: CoreExpr::Lit(CoreLiteral::Int(0)),
        }],
    };
    let v = eval_expr(&form, &env, &mut Echo).unwrap();
    assert_eq!(v, RuntimeValue::Int(7));
}

#[test]
fn oneshot_cont_returns_performed_at_top() {
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|v, _| {
            Ok(Outcome::Performed {
                op: "ask".into(),
                arg: v,
                resume: Rc::new(|v, _| Ok(Outcome::Value(v))),
            })
        })),
    );
    struct Echo;
    impl EffectHost for Echo {
        fn perform(&mut self, _op: &str, arg: RuntimeValue) -> EvalResult {
            Ok(arg)
        }
    }
    let v = eval_expr(&app_k(3), &env, &mut Echo).unwrap();
    assert_eq!(v, RuntimeValue::Int(3));
}
