//! Round-17 eval: residual Cont `other` arms and escaped-cell / forward edges
//! that remain under the filtered eval.rs miss set.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_eval::control::{identity_resume, Outcome, ResumeCont, UnitHost};
use reciplexa_eval::eval::{eval_expr, primitive_env};
use reciplexa_eval::value::RuntimeValue;

fn oneshot(cont: ResumeCont) -> RuntimeValue {
    RuntimeValue::OneShotResume {
        used: Rc::new(Cell::new(false)),
        cont,
    }
}

#[test]
fn eval_round17_cont_other_and_escaped_cell() {
    // Perform arg Cont → resume returns Forward (other)
    let mut env = primitive_env();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|_, _| Ok(Outcome::Forward))),
    );
    let expr = CoreExpr::Perform {
        op: "log".into(),
        arg: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
        }),
    };
    let _ = eval_expr(&expr, &env, &mut UnitHost);

    // Forward with already-used one-shot
    let used = Rc::new(Cell::new(true));
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        RuntimeValue::OneShotResume {
            used: used.clone(),
            cont: identity_resume(),
        },
    );
    let expr = CoreExpr::Forward {
        resume_name: "k".into(),
    };
    let _ = eval_expr(&expr, &env, &mut UnitHost);

    // Forward with non-resume binder
    let mut env = HashMap::new();
    env.insert("k".into(), RuntimeValue::Int(1));
    let expr = CoreExpr::Forward {
        resume_name: "k".into(),
    };
    let _ = eval_expr(&expr, &env, &mut UnitHost);

    // Escaped cell read
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

    // Escaped cell set
    let alive = Rc::new(Cell::new(false));
    let mut env = HashMap::new();
    env.insert(
        "c".into(),
        RuntimeValue::Cell {
            value: Rc::new(RefCell::new(RuntimeValue::Int(1))),
            alive,
        },
    );
    let expr = CoreExpr::Set {
        name: "c".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Int(2))),
    };
    let _ = eval_expr(&expr, &env, &mut UnitHost);

    // Unbound var
    let _ = eval_expr(
        &CoreExpr::Var("missing".into()),
        &HashMap::new(),
        &mut UnitHost,
    );

    // Error placeholder
    let _ = eval_expr(&CoreExpr::Error, &HashMap::new(), &mut UnitHost);
}
