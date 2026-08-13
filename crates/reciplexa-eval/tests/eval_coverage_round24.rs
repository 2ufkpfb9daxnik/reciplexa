//! Round-24 eval: more Handle Cont Forward/Err + deep_resume double-perform.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_eval::control::{EvalError, Outcome, ResumeCont, UnitHost};
use reciplexa_eval::eval::{eval_expr, primitive_env};
use reciplexa_eval::value::RuntimeValue;

fn oneshot(cont: ResumeCont) -> RuntimeValue {
    RuntimeValue::OneShotResume {
        used: Rc::new(Cell::new(false)),
        cont,
    }
}

fn handle_resume(body: CoreExpr) -> CoreExpr {
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
fn eval_round24_deep_resume_and_forward_handler() {
    let env = HashMap::new();

    // Triple perform for deep_resume nesting
    let _ = eval_expr(
        &handle_resume(CoreExpr::Seq(vec![
            perform_ask(CoreExpr::Lit(CoreLiteral::Int(1))),
            perform_ask(CoreExpr::Lit(CoreLiteral::Int(2))),
            perform_ask(CoreExpr::Lit(CoreLiteral::Int(3))),
        ])),
        &env,
        &mut UnitHost,
    );

    // Handler body = Forward (via CoreExpr::Forward) after binding resume
    let _ = eval_expr(
        &CoreExpr::Handle {
            op: "ask".into(),
            handler_params: vec!["m".into(), "k".into()],
            handler_body: Box::new(CoreExpr::Forward {
                resume_name: "k".into(),
            }),
            body: Box::new(perform_ask(CoreExpr::Lit(CoreLiteral::Unit))),
        },
        &env,
        &mut UnitHost,
    );

    // Handler resumes then body Continues with Set/RecordGet/Variant
    let _ = eval_expr(
        &handle_resume(CoreExpr::Seq(vec![
            perform_ask(CoreExpr::Lit(CoreLiteral::Int(1))),
            CoreExpr::Variant {
                tag: "ok".into(),
                payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Int(1)))),
            },
        ])),
        &env,
        &mut UnitHost,
    );

    // Match with Perform in arm body after scrutinee value
    let _ = eval_expr(
        &handle_resume(CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: perform_ask(CoreExpr::Lit(CoreLiteral::Int(9))),
            }],
        }),
        &env,
        &mut UnitHost,
    );

    // Cont Err via oneshot applied directly + arity / reuse Err
    let mut e = HashMap::new();
    e.insert(
        "ek".into(),
        oneshot(Rc::new(|_, _| {
            Err(EvalError {
                message: "boom".into(),
            })
        })),
    );
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("ek".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
        },
        &e,
        &mut UnitHost,
    );
    // reuse oneshot
    let used = Rc::new(Cell::new(true));
    e.insert(
        "used".into(),
        RuntimeValue::OneShotResume {
            used: Rc::clone(&used),
            cont: Rc::new(|_, _| Ok(Outcome::Value(RuntimeValue::Unit))),
        },
    );
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("used".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
        },
        &e,
        &mut UnitHost,
    );
    // wrong arity
    e.insert(
        "k1".into(),
        oneshot(Rc::new(|_, _| Ok(Outcome::Value(RuntimeValue::Unit)))),
    );
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k1".into())),
            args: vec![
                CoreExpr::Lit(CoreLiteral::Unit),
                CoreExpr::Lit(CoreLiteral::Unit),
            ],
        },
        &e,
        &mut UnitHost,
    );

    // Numeric binop mixed + predicates via App
    let penv = primitive_env();
    for (op, a, b) in [
        ("+", CoreLiteral::Int(1), CoreLiteral::F64(2.0)),
        ("<", CoreLiteral::Number(1.0), CoreLiteral::Int(2)),
        ("<=", CoreLiteral::F64(1.0), CoreLiteral::F64(2.0)),
        (">=", CoreLiteral::Int(3), CoreLiteral::Int(1)),
        ("/", CoreLiteral::F64(1.0), CoreLiteral::F64(2.0)),
        ("*", CoreLiteral::Number(2.0), CoreLiteral::Number(3.0)),
    ] {
        let _ = eval_expr(
            &CoreExpr::App {
                fun: Box::new(CoreExpr::Var(op.into())),
                args: vec![CoreExpr::Lit(a), CoreExpr::Lit(b)],
            },
            &penv,
            &mut UnitHost,
        );
    }
}
