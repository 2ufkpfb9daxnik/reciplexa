//! Round-27 eval: Cont `inner` other via Handle + oneshot that returns
//! Performed with a Forward resume (identity Perform resume can never Forward).

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

/// OneShot that yields Performed(ask) whose resume returns Forward.
fn ask_fwd_k() -> RuntimeValue {
    oneshot(Rc::new(|v, _| {
        Ok(Outcome::Performed {
            op: "ask".into(),
            arg: v,
            resume: Rc::new(|_, _| Ok(Outcome::Forward)),
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

fn call_fwd() -> CoreExpr {
    CoreExpr::App {
        fun: Box::new(CoreExpr::Var("fk".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
    }
}

#[test]
fn eval_round27_cont_inner_forward_via_performed_resume() {
    let mut env = HashMap::new();
    env.insert("fk".into(), ask_fwd_k());

    // Let Cont: inner resume Forwards
    let _ = eval_expr(
        &handle(CoreExpr::Let {
            name: "x".into(),
            value: Box::new(call_fwd()),
            body: Box::new(CoreExpr::Var("x".into())),
        }),
        &env,
        &mut UnitHost,
    );

    // LocalVar Cont
    let _ = eval_expr(
        &handle(CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(call_fwd()),
            body: Box::new(CoreExpr::Var("c".into())),
        }),
        &env,
        &mut UnitHost,
    );

    // Set Cont
    let _ = eval_expr(
        &handle(CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
            body: Box::new(CoreExpr::Set {
                name: "c".into(),
                value: Box::new(call_fwd()),
            }),
        }),
        &env,
        &mut UnitHost,
    );

    // If Cont
    let _ = eval_expr(
        &handle(CoreExpr::If {
            cond: Box::new(call_fwd()),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        }),
        &env,
        &mut UnitHost,
    );

    // RecordGet Cont
    let _ = eval_expr(
        &handle(CoreExpr::RecordGet {
            record: Box::new(call_fwd()),
            field: "a".into(),
        }),
        &env,
        &mut UnitHost,
    );

    // RecordUpdate Cont
    let _ = eval_expr(
        &handle(CoreExpr::RecordUpdate {
            record: Box::new(call_fwd()),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
        }),
        &env,
        &mut UnitHost,
    );

    // RecordExtend Cont
    let _ = eval_expr(
        &handle(CoreExpr::RecordExtend {
            record: Box::new(call_fwd()),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
        }),
        &env,
        &mut UnitHost,
    );

    // Match Cont
    let _ = eval_expr(
        &handle(CoreExpr::Match {
            scrutinee: Box::new(call_fwd()),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(1)),
            }],
        }),
        &env,
        &mut UnitHost,
    );

    // Variant Cont
    let _ = eval_expr(
        &handle(CoreExpr::Variant {
            tag: "ok".into(),
            payload: Some(Box::new(call_fwd())),
        }),
        &env,
        &mut UnitHost,
    );

    // App fun Cont
    let mut penv = primitive_env();
    penv.insert("fk".into(), ask_fwd_k());
    let _ = eval_expr(
        &handle(CoreExpr::App {
            fun: Box::new(call_fwd()),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        }),
        &penv,
        &mut UnitHost,
    );

    // App arg Cont
    let _ = eval_expr(
        &handle(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![call_fwd(), CoreExpr::Lit(CoreLiteral::Int(2))],
        }),
        &penv,
        &mut UnitHost,
    );

    // Record field Cont
    let _ = eval_expr(
        &handle(CoreExpr::Record {
            fields: vec![("a".into(), call_fwd())],
        }),
        &env,
        &mut UnitHost,
    );

    // Seq Cont
    let _ = eval_expr(
        &handle(CoreExpr::Seq(vec![
            call_fwd(),
            CoreExpr::Lit(CoreLiteral::Int(1)),
        ])),
        &env,
        &mut UnitHost,
    );

    // Cast / TryCast / CheckCast Cont
    for mk in [
        CoreExpr::Cast {
            expr: Box::new(call_fwd()),
            evidence: CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 1,
        },
        CoreExpr::TryCast {
            expr: Box::new(call_fwd()),
            target: CoreType::Int,
            cast_id: 2,
        },
        CoreExpr::CheckCast {
            expr: Box::new(call_fwd()),
            target: CoreType::Int,
            cast_id: 3,
        },
    ] {
        // Rebuild fk each time (oneshot used)
        let mut e = HashMap::new();
        e.insert("fk".into(), ask_fwd_k());
        let _ = eval_expr(&handle(mk), &e, &mut UnitHost);
    }

    // With handler Cont
    let mut e = HashMap::new();
    e.insert("fk".into(), ask_fwd_k());
    let _ = eval_expr(
        &handle(CoreExpr::With {
            handler: Box::new(call_fwd()),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        }),
        &e,
        &mut UnitHost,
    );

    // Perform does not Cont-wrap arg Performed (bubbles); still tip path
    let mut e = HashMap::new();
    e.insert("fk".into(), ask_fwd_k());
    let _ = eval_expr(
        &handle(CoreExpr::Perform {
            op: "ask".into(),
            arg: Box::new(call_fwd()),
        }),
        &e,
        &mut UnitHost,
    );
}
