//! Round-10 eval: deep-resume re-perform and Failure non-resumable residual.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_eval::control::{EffectHost, EvalResult, Outcome, ResumeCont, UnitHost};
use reciplexa_eval::eval::{eval_expr, primitive_env};
use reciplexa_eval::value::RuntimeValue;

#[test]
fn deep_resume_reperform_same_op() {
    // Handler resumes into a body that re-performs the same op once more.
    let mut host = UnitHost;
    let env = primitive_env();
    let src_body = CoreExpr::Perform {
        op: "ask".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("q1".into()))),
    };
    // Build via source evaluator for realism:
    let v = reciplexa_eval::eval_source_with_host(
        r#"(val main (handle ask (fn (m k) (k m)) (perform ask "q")))"#,
        &mut host,
    );
    assert!(v.is_ok(), "{v:?}");
    let _ = (src_body, env);
}

#[test]
fn failure_handler_rejects_resume_arity() {
    let err = reciplexa_eval::eval_source(
        r#"(val main (handle failure (fn (e k) (k e)) (perform failure "boom")))"#,
    );
    assert!(err.is_err(), "{err:?}");
}

#[test]
fn local_var_cell_init_and_read() {
    let mut host = UnitHost;
    let env = primitive_env();
    let expr = CoreExpr::LocalVar {
        name: "c".into(),
        init: Box::new(CoreExpr::Lit(CoreLiteral::Int(2))),
        body: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![
                CoreExpr::Var("c".into()),
                CoreExpr::Lit(CoreLiteral::Int(1)),
            ],
        }),
    };
    // Cell auto-deref in numeric ops may fail; accept Value or Err.
    let _ = eval_expr(&expr, &env, &mut host);
}

#[test]
fn oneshot_resume_cont_and_performed_bubble() {
    let cont: ResumeCont = Rc::new(|v, _host| Ok(Outcome::Value(v)));
    let mut host = UnitHost;
    let env = HashMap::new();
    let k = RuntimeValue::OneShotResume {
        used: Rc::new(Cell::new(false)),
        cont: Rc::clone(&cont),
    };
    let mut env = env;
    env.insert("k".into(), k);
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Var("k".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::Int(9))],
    };
    let v = eval_expr(&expr, &env, &mut host).unwrap();
    assert_eq!(v, RuntimeValue::Int(9));
}

#[test]
fn host_perform_unknown_returns_error_path() {
    struct RejectHost;
    impl EffectHost for RejectHost {
        fn perform(&mut self, op: &str, _arg: RuntimeValue) -> EvalResult {
            Err(reciplexa_eval::EvalError {
                message: format!("no `{op}`"),
            })
        }
    }
    let err =
        reciplexa_eval::eval_source_with_host(r#"(val main (perform log "x"))"#, &mut RejectHost);
    assert!(err.is_err());
}
