//! Round-22 eval: Cont Forward through Seq/Let/If/Match rest + cast/builtin leaves.

use std::cell::Cell;
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

#[test]
fn eval_round22_cont_forward_let_if_match_seq() {
    let mut env = primitive_env();
    env.insert("k".into(), forward_k());

    // Seq: first Performs; resume then Forward via k
    let _ = eval_expr(
        &CoreExpr::Seq(vec![
            CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            },
            CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            },
        ]),
        &env,
        &mut UnitHost,
    );

    // Let value Performs; body Forward
    let _ = eval_expr(
        &CoreExpr::Let {
            name: "x".into(),
            value: Box::new(CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            }),
            body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            }),
        },
        &env,
        &mut UnitHost,
    );

    // If cond Performs; then Forward
    let _ = eval_expr(
        &CoreExpr::If {
            cond: Box::new(CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
            }),
            then_branch: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            }),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        &env,
        &mut UnitHost,
    );

    // Match scrutinee Performs; arm body Forward
    let _ = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            }),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("k".into())),
                    args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
                },
            }],
        },
        &env,
        &mut UnitHost,
    );

    // RecordUpdate / RecordExtend Perform then Forward
    let base = CoreExpr::Record {
        fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
    };
    let _ = eval_expr(
        &CoreExpr::RecordUpdate {
            record: Box::new(CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(base.clone()),
            }),
            fields: vec![(
                "a".into(),
                CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("k".into())),
                    args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
                },
            )],
        },
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &CoreExpr::RecordExtend {
            record: Box::new(CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(base),
            }),
            fields: vec![(
                "b".into(),
                CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("k".into())),
                    args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
                },
            )],
        },
        &env,
        &mut UnitHost,
    );

    // Cast evidence leaves
    let _ = eval_expr(
        &CoreExpr::Cast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            evidence: CastEvidence::UnionCheck {
                members: vec![CoreType::Int, CoreType::String],
            },
            target: CoreType::Int,
            cast_id: 0,
        },
        &env,
        &mut UnitHost,
    );
}
