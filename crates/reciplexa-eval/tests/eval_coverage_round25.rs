//! Round-25 eval: match_pattern miss keys, VariantCheck edges, Cont RecordGet/Set/Cast.

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
fn eval_round25_match_cast_cont_residuals() {
    let env = HashMap::new();

    // Tuple pattern missing key → match_pattern None (`?` at find)
    let _ = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Record {
                fields: vec![("0".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Tuple(vec![
                        CorePattern::Bind("a".into()),
                        CorePattern::Bind("b".into()),
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

    // Record pattern missing label
    let _ = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Record {
                        fields: vec![("b".into(), CorePattern::Bind("x".into()))],
                    },
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

    // Nested sub-pattern fail on record field
    let _ = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Record {
                        fields: vec![("a".into(), CorePattern::Lit(CoreLiteral::Bool(true)))],
                    },
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

    // RecordGet / Set Cont resume
    let _ = eval_expr(
        &handle(CoreExpr::RecordGet {
            record: Box::new(perform_ask(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            })),
            field: "a".into(),
        }),
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &handle(CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
            body: Box::new(CoreExpr::Set {
                name: "c".into(),
                value: Box::new(perform_ask(CoreExpr::Lit(CoreLiteral::Int(9)))),
            }),
        }),
        &env,
        &mut UnitHost,
    );

    // Cast / TryCast / CheckCast Cont (expr Performs)
    let _ = eval_expr(
        &handle(CoreExpr::Cast {
            expr: Box::new(perform_ask(CoreExpr::Lit(CoreLiteral::Int(1)))),
            evidence: CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 1,
        }),
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &handle(CoreExpr::TryCast {
            expr: Box::new(perform_ask(CoreExpr::Lit(CoreLiteral::Int(1)))),
            target: CoreType::Int,
            cast_id: 2,
        }),
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &handle(CoreExpr::CheckCast {
            expr: Box::new(perform_ask(CoreExpr::Lit(CoreLiteral::Int(1)))),
            target: CoreType::Int,
            cast_id: 3,
        }),
        &env,
        &mut UnitHost,
    );

    // RecordUpdate field Performs + Cont other via Forward after resume
    let mut env_fwd = HashMap::new();
    env_fwd.insert("fk".into(), forward_k());
    let base = CoreExpr::Record {
        fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
    };
    let _ = eval_expr(
        &handle(CoreExpr::RecordUpdate {
            record: Box::new(base.clone()),
            fields: vec![("a".into(), perform_ask(CoreExpr::Lit(CoreLiteral::Int(1))))],
        }),
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &handle(CoreExpr::RecordExtend {
            record: Box::new(base),
            fields: vec![("b".into(), perform_ask(CoreExpr::Lit(CoreLiteral::Int(2))))],
        }),
        &env,
        &mut UnitHost,
    );
    // Cont other: record Performs, resume then Forward
    let _ = eval_expr(
        &handle(CoreExpr::RecordUpdate {
            record: Box::new(perform_ask(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("fk".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            })),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
        }),
        &env_fwd,
        &mut UnitHost,
    );

    // Variant payload Performs
    let _ = eval_expr(
        &handle(CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(perform_ask(CoreExpr::Lit(CoreLiteral::Int(1))))),
        }),
        &env,
        &mut UnitHost,
    );

    // Seq Cont other on first item resume → Forward
    let _ = eval_expr(
        &handle(CoreExpr::Seq(vec![
            perform_ask(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("fk".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            }),
            CoreExpr::Lit(CoreLiteral::Int(1)),
        ])),
        &env_fwd,
        &mut UnitHost,
    );

    // Builtin IsString / IsBool via App
    let penv = primitive_env();
    for (op, lit) in [
        ("string?", CoreLiteral::String("x".into())),
        ("bool?", CoreLiteral::Bool(true)),
        ("string?", CoreLiteral::Int(1)),
        ("bool?", CoreLiteral::Int(1)),
    ] {
        let _ = eval_expr(
            &CoreExpr::App {
                fun: Box::new(CoreExpr::Var(op.into())),
                args: vec![CoreExpr::Lit(lit)],
            },
            &penv,
            &mut UnitHost,
        );
    }

    // VariantCheck (Some, None) / tag mismatch via CheckCast evidence path
    let _ = eval_expr(
        &CoreExpr::CheckCast {
            expr: Box::new(CoreExpr::Variant {
                tag: "ok".into(),
                payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Int(1)))),
            }),
            target: CoreType::Variant {
                variants: vec![("ok".into(), None)],
            },
            cast_id: 9,
        },
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &CoreExpr::CheckCast {
            expr: Box::new(CoreExpr::Variant {
                tag: "err".into(),
                payload: None,
            }),
            target: CoreType::Variant {
                variants: vec![("ok".into(), None)],
            },
            cast_id: 10,
        },
        &env,
        &mut UnitHost,
    );
}
