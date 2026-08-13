//! Round-26 eval: Cont other via nested Perform inside resume chain.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
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
fn eval_round26_cont_other_and_deep_nests() {
    let mut env_fwd = HashMap::new();
    env_fwd.insert("fk".into(), forward_k());

    // Let Cont other: value resume returns Forward
    let _ = eval_expr(
        &handle(CoreExpr::Let {
            name: "x".into(),
            value: Box::new(perform_ask(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("fk".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            })),
            body: Box::new(CoreExpr::Var("x".into())),
        }),
        &env_fwd,
        &mut UnitHost,
    );

    // LocalVar Cont other
    let _ = eval_expr(
        &handle(CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(perform_ask(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("fk".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            })),
            body: Box::new(CoreExpr::Var("c".into())),
        }),
        &env_fwd,
        &mut UnitHost,
    );

    // Set Cont other
    let _ = eval_expr(
        &handle(CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
            body: Box::new(CoreExpr::Set {
                name: "c".into(),
                value: Box::new(perform_ask(CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("fk".into())),
                    args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
                })),
            }),
        }),
        &env_fwd,
        &mut UnitHost,
    );

    // If Cont other
    let _ = eval_expr(
        &handle(CoreExpr::If {
            cond: Box::new(perform_ask(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("fk".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            })),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        }),
        &env_fwd,
        &mut UnitHost,
    );

    // RecordGet Cont other
    let _ = eval_expr(
        &handle(CoreExpr::RecordGet {
            record: Box::new(perform_ask(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("fk".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            })),
            field: "a".into(),
        }),
        &env_fwd,
        &mut UnitHost,
    );

    // Match Cont other
    let _ = eval_expr(
        &handle(CoreExpr::Match {
            scrutinee: Box::new(perform_ask(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("fk".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            })),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(1)),
            }],
        }),
        &env_fwd,
        &mut UnitHost,
    );

    // App fun Cont other
    let mut env_app = primitive_env();
    env_app.insert("fk".into(), forward_k());
    let _ = eval_expr(
        &handle(CoreExpr::App {
            fun: Box::new(perform_ask(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("fk".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            })),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        }),
        &env_app,
        &mut UnitHost,
    );

    // App arg Cont other (first arg)
    let _ = eval_expr(
        &handle(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![
                perform_ask(CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("fk".into())),
                    args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
                }),
                CoreExpr::Lit(CoreLiteral::Int(2)),
            ],
        }),
        &env_app,
        &mut UnitHost,
    );

    // Variant Cont other
    let _ = eval_expr(
        &handle(CoreExpr::Variant {
            tag: "ok".into(),
            payload: Some(Box::new(perform_ask(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("fk".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            }))),
        }),
        &env_fwd,
        &mut UnitHost,
    );

    // Record field Cont other
    let _ = eval_expr(
        &handle(CoreExpr::Record {
            fields: vec![(
                "a".into(),
                perform_ask(CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("fk".into())),
                    args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
                }),
            )],
        }),
        &env_fwd,
        &mut UnitHost,
    );

    // Double-perform deep_resume then Forward in second
    let _ = eval_expr(
        &handle(CoreExpr::Seq(vec![
            perform_ask(CoreExpr::Lit(CoreLiteral::Int(1))),
            perform_ask(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("fk".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            }),
        ])),
        &env_fwd,
        &mut UnitHost,
    );

    // Tuple match missing key with 3-elem pattern / 2-field record
    let env = HashMap::new();
    let _ = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Record {
                fields: vec![
                    ("0".into(), CoreExpr::Lit(CoreLiteral::Int(1))),
                    ("1".into(), CoreExpr::Lit(CoreLiteral::Int(2))),
                ],
            }),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Tuple(vec![
                        CorePattern::Wildcard,
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
        &env,
        &mut UnitHost,
    );
}
