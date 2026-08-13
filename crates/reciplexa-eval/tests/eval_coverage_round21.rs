//! Round-21 eval: Cont Forward deep resume rest-loops + match/cast leftovers.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::control::{Outcome, ResumeCont, UnitHost};
use reciplexa_eval::eval::{eval_expr, primitive_env};
use reciplexa_eval::value::RuntimeValue;

fn oneshot(cont: ResumeCont) -> RuntimeValue {
    RuntimeValue::OneShotResume {
        used: Rc::new(Cell::new(false)),
        cont,
    }
}

fn forward_k() -> RuntimeValue {
    oneshot(Rc::new(|_, _| Ok(Outcome::Forward)))
}

fn perform_then_forward(op: &str) -> CoreExpr {
    CoreExpr::Seq(vec![
        CoreExpr::Perform {
            op: op.into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
        },
    ])
}

#[test]
fn eval_round21_cont_forward_rest_loops() {
    let mut env = primitive_env();
    env.insert("k".into(), forward_k());

    // App: first arg Performs; resume then hits Forward on later arg via k
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![
                CoreExpr::Perform {
                    op: "ask".into(),
                    arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
                },
                CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("k".into())),
                    args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
                },
            ],
        },
        &env,
        &mut UnitHost,
    );

    // Record: field0 Performs; resume evaluates field1 → Forward
    let _ = eval_expr(
        &CoreExpr::Record {
            fields: vec![
                (
                    "a".into(),
                    CoreExpr::Perform {
                        op: "ask".into(),
                        arg: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
                    },
                ),
                (
                    "b".into(),
                    CoreExpr::App {
                        fun: Box::new(CoreExpr::Var("k".into())),
                        args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
                    },
                ),
            ],
        },
        &env,
        &mut UnitHost,
    );

    // RecordUpdate / Extend: Cont Forward on field values after perform
    let _ = eval_expr(
        &CoreExpr::RecordUpdate {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
            }),
            fields: vec![
                (
                    "a".into(),
                    CoreExpr::Perform {
                        op: "ask".into(),
                        arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
                    },
                ),
                (
                    "a".into(),
                    CoreExpr::App {
                        fun: Box::new(CoreExpr::Var("k".into())),
                        args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
                    },
                ),
            ],
        },
        &env,
        &mut UnitHost,
    );

    // Seq Cont Forward mid-stream
    let _ = eval_expr(&perform_then_forward("ask"), &env, &mut UnitHost);

    // Match Cont Forward on scrutinee
    let _ = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            }),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(1)),
            }],
        },
        &env,
        &mut UnitHost,
    );

    // Cast Cont Forward on expr
    let _ = eval_expr(
        &CoreExpr::Cast {
            expr: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            }),
            evidence: CastEvidence::Identity,
            target: CoreType::Unit,
            cast_id: 1,
        },
        &env,
        &mut UnitHost,
    );

    // Forward already-used / missing binder
    let used = oneshot(Rc::new(|_, _| Ok(Outcome::Value(RuntimeValue::Unit))));
    if let RuntimeValue::OneShotResume { used: flag, .. } = &used {
        flag.set(true);
    }
    env.insert("used".into(), used);
    let _ = eval_expr(
        &CoreExpr::Forward {
            resume_name: "used".into(),
        },
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &CoreExpr::Forward {
            resume_name: "missing".into(),
        },
        &env,
        &mut UnitHost,
    );

    // Cell escaped
    env.insert(
        "c".into(),
        RuntimeValue::Cell {
            value: Rc::new(std::cell::RefCell::new(RuntimeValue::Int(1))),
            alive: Rc::new(Cell::new(false)),
        },
    );
    let _ = eval_expr(&CoreExpr::Var("c".into()), &env, &mut UnitHost);

    // match_pattern residual: record missing field / variant tag miss / lit miss
    for (scr, pat) in [
        (
            CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            },
            CorePattern::Record {
                fields: vec![("b".into(), CorePattern::Bind("x".into()))],
            },
        ),
        (
            CoreExpr::Variant {
                tag: "a".into(),
                payload: None,
            },
            CorePattern::Variant {
                tag: "b".into(),
                payload: None,
            },
        ),
        (
            CoreExpr::Lit(CoreLiteral::Int(1)),
            CorePattern::Lit(CoreLiteral::Int(2)),
        ),
        (
            CoreExpr::Lit(CoreLiteral::Bool(true)),
            CorePattern::Lit(CoreLiteral::Bool(false)),
        ),
    ] {
        let _ = eval_expr(
            &CoreExpr::Match {
                scrutinee: Box::new(scr),
                arms: vec![
                    MatchArm {
                        pattern: pat,
                        body: CoreExpr::Lit(CoreLiteral::Int(0)),
                    },
                    MatchArm {
                        pattern: CorePattern::Wildcard,
                        body: CoreExpr::Lit(CoreLiteral::Int(1)),
                    },
                ],
            },
            &HashMap::new(),
            &mut UnitHost,
        );
    }

    // TagCheck / NominalCheck / IntersectionCheck via Cast
    for (ev, lit) in [
        (
            CastEvidence::TagCheck { tag: "int".into() },
            CoreLiteral::Int(1),
        ),
        (
            CastEvidence::TagCheck {
                tag: "string".into(),
            },
            CoreLiteral::Int(1),
        ),
        (
            CastEvidence::NominalCheck { name: "ok".into() },
            CoreLiteral::Unit,
        ),
        (
            CastEvidence::IntersectionCheck {
                members: vec![CoreType::Int],
            },
            CoreLiteral::Int(3),
        ),
        (CastEvidence::NumericPromote, CoreLiteral::Int(1)),
        (
            CastEvidence::Compose(vec![
                CastEvidence::TagCheck { tag: "int".into() },
                CastEvidence::NumericPromote,
            ]),
            CoreLiteral::Int(1),
        ),
    ] {
        let _ = eval_expr(
            &CoreExpr::Cast {
                expr: Box::new(CoreExpr::Lit(lit)),
                evidence: ev,
                target: CoreType::dyn_any(),
                cast_id: 9,
            },
            &HashMap::new(),
            &mut UnitHost,
        );
    }
}
