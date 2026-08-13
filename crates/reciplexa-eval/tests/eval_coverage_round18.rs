//! Round-18 eval: Cont `other` (Forward) through remaining match arms +
//! deep-resume Cont leftovers still under eval.rs miss set.

use std::cell::{Cell, RefCell};
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

fn call_k() -> CoreExpr {
    CoreExpr::App {
        fun: Box::new(CoreExpr::Var("k".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
    }
}

#[test]
fn eval_round18_forward_cont_through_forms() {
    let mut env = HashMap::new();
    env.insert("k".into(), forward_k());

    // Cast / TryCast / CheckCast Cont other via Forward
    for expr in [
        CoreExpr::Cast {
            expr: Box::new(call_k()),
            evidence: CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 1,
        },
        CoreExpr::TryCast {
            expr: Box::new(call_k()),
            target: CoreType::Int,
            cast_id: 2,
        },
        CoreExpr::CheckCast {
            expr: Box::new(call_k()),
            target: CoreType::Int,
            cast_id: 3,
        },
    ] {
        let _ = eval_expr(&expr, &env, &mut UnitHost);
    }

    // With handler Cont other
    let _ = eval_expr(
        &CoreExpr::With {
            handler: Box::new(call_k()),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &env,
        &mut UnitHost,
    );

    // LocalVar / Set / If Cont other
    let _ = eval_expr(
        &CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(call_k()),
            body: Box::new(CoreExpr::Var("c".into())),
        },
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &CoreExpr::Set {
            name: "c".into(),
            value: Box::new(call_k()),
        },
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &CoreExpr::If {
            cond: Box::new(call_k()),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        &env,
        &mut UnitHost,
    );

    // RecordUpdate / RecordExtend field Cont other
    let base = CoreExpr::Record {
        fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
    };
    let _ = eval_expr(
        &CoreExpr::RecordUpdate {
            record: Box::new(base.clone()),
            fields: vec![("a".into(), call_k())],
        },
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &CoreExpr::RecordExtend {
            record: Box::new(base),
            fields: vec![("b".into(), call_k())],
        },
        &env,
        &mut UnitHost,
    );

    // Seq / Let Cont other
    let _ = eval_expr(
        &CoreExpr::Seq(vec![call_k(), CoreExpr::Lit(CoreLiteral::Int(9))]),
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &CoreExpr::Let {
            name: "x".into(),
            value: Box::new(call_k()),
            body: Box::new(CoreExpr::Var("x".into())),
        },
        &env,
        &mut UnitHost,
    );

    // App fun Cont + arg Cont
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(call_k()),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &env,
        &mut UnitHost,
    );
    let mut env2 = primitive_env();
    env2.insert("k".into(), forward_k());
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![call_k(), CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &env2,
        &mut UnitHost,
    );

    // Perform arg Cont Forward
    let _ = eval_expr(
        &CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(call_k()),
        },
        &env,
        &mut UnitHost,
    );

    // Match scrutinee Cont
    let _ = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(call_k()),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(0)),
            }],
        },
        &env,
        &mut UnitHost,
    );
}

#[test]
fn eval_round18_deep_resume_cont_and_match_edges() {
    // OneShotResume cont returns Forward (L913 other)
    let mut env = HashMap::new();
    env.insert("k".into(), forward_k());
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &env,
        &mut UnitHost,
    );

    // Nested resume: outer perform, resume returns Forward via oneshot
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|v, _| {
            Ok(Outcome::Performed {
                op: "inner".into(),
                arg: v,
                resume: Rc::new(|_, _| Ok(Outcome::Forward)),
            })
        })),
    );
    let _ = eval_expr(
        &CoreExpr::Let {
            name: "x".into(),
            value: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
            }),
            body: Box::new(CoreExpr::Var("x".into())),
        },
        &env,
        &mut UnitHost,
    );

    // Record pattern missing key / tuple length miss / variant payload miss
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Record {
            fields: vec![("b".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
        }),
        arms: vec![
            MatchArm {
                pattern: CorePattern::Record {
                    fields: vec![("a".into(), CorePattern::Bind("x".into()))],
                },
                body: CoreExpr::Lit(CoreLiteral::Int(1)),
            },
            MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(0)),
            },
        ],
    };
    let _ = eval_expr(&expr, &HashMap::new(), &mut UnitHost);

    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Record {
            fields: vec![("0".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
        }),
        arms: vec![
            MatchArm {
                pattern: CorePattern::Tuple(vec![
                    CorePattern::Bind("a".into()),
                    CorePattern::Bind("b".into()),
                ]),
                body: CoreExpr::Lit(CoreLiteral::Int(1)),
            },
            MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(0)),
            },
        ],
    };
    let _ = eval_expr(&expr, &HashMap::new(), &mut UnitHost);

    // Happy nested record field pattern
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Record {
            fields: vec![(
                "a".into(),
                CoreExpr::Record {
                    fields: vec![("b".into(), CoreExpr::Lit(CoreLiteral::Int(7)))],
                },
            )],
        }),
        arms: vec![MatchArm {
            pattern: CorePattern::Record {
                fields: vec![(
                    "a".into(),
                    CorePattern::Record {
                        fields: vec![("b".into(), CorePattern::Bind("x".into()))],
                    },
                )],
            },
            body: CoreExpr::Var("x".into()),
        }],
    };
    let _ = eval_expr(&expr, &HashMap::new(), &mut UnitHost);

    // Escaped cell still covered nearby; wrong builtin arity
    let alive = Rc::new(Cell::new(false));
    let mut env = HashMap::new();
    env.insert(
        "c".into(),
        RuntimeValue::Cell {
            value: Rc::new(RefCell::new(RuntimeValue::Int(1))),
            alive,
        },
    );
    let _ = eval_expr(&CoreExpr::Var("c".into()), &env, &mut UnitHost);

    let env = primitive_env();
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("/".into())),
            args: vec![
                CoreExpr::Lit(CoreLiteral::Int(1)),
                CoreExpr::Lit(CoreLiteral::Int(2)),
            ],
        },
        &env,
        &mut UnitHost,
    );
}
