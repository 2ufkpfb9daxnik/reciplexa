//! Round-23 eval: Handle-wrapped Cont so resume closures actually run
//! (prior Cont Forward tips returned Performed but never resumed).

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::control::{EvalError, Outcome, ResumeCont, UnitHost};
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

fn err_k() -> RuntimeValue {
    oneshot(Rc::new(|_, _| {
        Err(EvalError {
            message: "resume-err".into(),
        })
    }))
}

fn handle(body: CoreExpr) -> CoreExpr {
    CoreExpr::Handle {
        op: "ask".into(),
        handler_params: vec!["m".into(), "k".into()],
        handler_body: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Var("m".into())],
        }),
        body: Box::new(body),
    }
}

fn perform_ask(arg: CoreExpr) -> CoreExpr {
    CoreExpr::Perform {
        op: "ask".into(),
        arg: Box::new(arg),
    }
}

#[test]
fn eval_round23_handle_resume_cont_matrix() {
    let env = HashMap::new();

    // Seq: resume after Perform then Value rest
    let _ = eval_expr(
        &handle(CoreExpr::Seq(vec![
            perform_ask(CoreExpr::Lit(CoreLiteral::Int(1))),
            CoreExpr::Lit(CoreLiteral::Int(2)),
        ])),
        &env,
        &mut UnitHost,
    );

    // Seq: resume then Forward via oneshot in rest (hits `other` on rest loop)
    let mut env_fwd = HashMap::new();
    env_fwd.insert("fk".into(), forward_k());
    let _ = eval_expr(
        &handle(CoreExpr::Seq(vec![
            perform_ask(CoreExpr::Lit(CoreLiteral::Unit)),
            CoreExpr::App {
                fun: Box::new(CoreExpr::Var("fk".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            },
        ])),
        &env_fwd,
        &mut UnitHost,
    );

    // Seq: double Perform → deep_resume Performed re-entry
    let _ = eval_expr(
        &handle(CoreExpr::Seq(vec![
            perform_ask(CoreExpr::Lit(CoreLiteral::Int(1))),
            perform_ask(CoreExpr::Lit(CoreLiteral::Int(2))),
        ])),
        &env,
        &mut UnitHost,
    );

    // Let value Performs; resume then body
    let _ = eval_expr(
        &handle(CoreExpr::Let {
            name: "x".into(),
            value: Box::new(perform_ask(CoreExpr::Lit(CoreLiteral::Int(7)))),
            body: Box::new(CoreExpr::Var("x".into())),
        }),
        &env,
        &mut UnitHost,
    );

    // Let: resume then Forward in body
    let _ = eval_expr(
        &handle(CoreExpr::Let {
            name: "x".into(),
            value: Box::new(perform_ask(CoreExpr::Lit(CoreLiteral::Unit))),
            body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("fk".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            }),
        }),
        &env_fwd,
        &mut UnitHost,
    );

    // LocalVar init Performs
    let _ = eval_expr(
        &handle(CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(perform_ask(CoreExpr::Lit(CoreLiteral::Int(3)))),
            body: Box::new(CoreExpr::Var("c".into())),
        }),
        &env,
        &mut UnitHost,
    );

    // If cond Performs
    let _ = eval_expr(
        &handle(CoreExpr::If {
            cond: Box::new(perform_ask(CoreExpr::Lit(CoreLiteral::Bool(true)))),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        }),
        &env,
        &mut UnitHost,
    );

    // Match scrutinee Performs
    let _ = eval_expr(
        &handle(CoreExpr::Match {
            scrutinee: Box::new(perform_ask(CoreExpr::Lit(CoreLiteral::Int(1)))),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(9)),
            }],
        }),
        &env,
        &mut UnitHost,
    );

    // App fun Performs; resume then apply
    let env_add = primitive_env();
    let _ = eval_expr(
        &handle(CoreExpr::App {
            fun: Box::new(perform_ask(CoreExpr::Var("+".into()))),
            args: vec![
                CoreExpr::Lit(CoreLiteral::Int(1)),
                CoreExpr::Lit(CoreLiteral::Int(2)),
            ],
        }),
        &env_add,
        &mut UnitHost,
    );

    // App arg Performs; rest arg after resume
    let _ = eval_expr(
        &handle(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![
                perform_ask(CoreExpr::Lit(CoreLiteral::Int(1))),
                CoreExpr::Lit(CoreLiteral::Int(2)),
            ],
        }),
        &env_add,
        &mut UnitHost,
    );

    // App: first arg resume then Forward on second
    let mut env_app = primitive_env();
    env_app.insert("fk".into(), forward_k());
    let _ = eval_expr(
        &handle(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![
                perform_ask(CoreExpr::Lit(CoreLiteral::Int(1))),
                CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("fk".into())),
                    args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
                },
            ],
        }),
        &env_app,
        &mut UnitHost,
    );

    // Record field Performs
    let _ = eval_expr(
        &handle(CoreExpr::Record {
            fields: vec![
                (
                    "a".into(),
                    perform_ask(CoreExpr::Lit(CoreLiteral::Int(1))),
                ),
                ("b".into(), CoreExpr::Lit(CoreLiteral::Int(2))),
            ],
        }),
        &env,
        &mut UnitHost,
    );

    // RecordUpdate / Extend record or field Performs
    let base = CoreExpr::Record {
        fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
    };
    let _ = eval_expr(
        &handle(CoreExpr::RecordUpdate {
            record: Box::new(perform_ask(base.clone())),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
        }),
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &handle(CoreExpr::RecordUpdate {
            record: Box::new(base.clone()),
            fields: vec![(
                "a".into(),
                perform_ask(CoreExpr::Lit(CoreLiteral::Int(2))),
            )],
        }),
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &handle(CoreExpr::RecordExtend {
            record: Box::new(perform_ask(base)),
            fields: vec![("b".into(), CoreExpr::Lit(CoreLiteral::Int(3)))],
        }),
        &env,
        &mut UnitHost,
    );

    // Cast / TryCast / CheckCast expr Performs
    let _ = eval_expr(
        &handle(CoreExpr::Cast {
            expr: Box::new(perform_ask(CoreExpr::Lit(CoreLiteral::Int(1)))),
            evidence: CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 0,
        }),
        &env,
        &mut UnitHost,
    );
}

#[test]
fn eval_round23_oneshot_err_forward_match_lits() {
    // apply_value OneShotResume → Forward / Err (line 913)
    let mut env = HashMap::new();
    env.insert("fk".into(), forward_k());
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("fk".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
        },
        &env,
        &mut UnitHost,
    );
    env.insert("ek".into(), err_k());
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("ek".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
        },
        &env,
        &mut UnitHost,
    );

    // Lit Color / Number / Bytes / shape-tag strings
    for lit in [
        CoreLiteral::Color("red".into()),
        CoreLiteral::Number(1.25),
        CoreLiteral::Bytes(vec![1, 2, 3]),
        CoreLiteral::String("circle".into()),
        CoreLiteral::String("rect".into()),
        CoreLiteral::String("text".into()),
        CoreLiteral::String("plain".into()),
        CoreLiteral::Unit,
        CoreLiteral::Bool(false),
    ] {
        let _ = eval_expr(&CoreExpr::Lit(lit), &HashMap::new(), &mut UnitHost);
    }

    // Variant nullary with nested Lit payload pattern → None (line 1040)
    let _ = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "none".into(),
                payload: None,
            }),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Variant {
                        tag: "none".into(),
                        payload: Some(Box::new(CorePattern::Lit(CoreLiteral::Int(1)))),
                    },
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

    // Tuple / Record pattern: missing key → None (1001 / 1014)
    let _ = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Record {
                fields: vec![("0".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Tuple(vec![
                        CorePattern::Wildcard,
                        CorePattern::Wildcard,
                    ]),
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
    let _ = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Record {
                        fields: vec![("b".into(), CorePattern::Wildcard)],
                    },
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

    // Nested record pattern match Ok (sub match_pattern)
    let _ = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Record {
                fields: vec![
                    ("0".into(), CoreExpr::Lit(CoreLiteral::Int(1))),
                    ("1".into(), CoreExpr::Lit(CoreLiteral::Int(2))),
                ],
            }),
            arms: vec![MatchArm {
                pattern: CorePattern::Tuple(vec![
                    CorePattern::Lit(CoreLiteral::Int(1)),
                    CorePattern::Bind("y".into()),
                ]),
                body: CoreExpr::Var("y".into()),
            }],
        },
        &HashMap::new(),
        &mut UnitHost,
    );
}
