//! Round-11 eval: Error node, perform-resume loop, cast/cell/builtin residual leaves.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::control::{EffectHost, EvalResult, Outcome, ResumeCont, UnitHost};
use reciplexa_eval::eval::{eval_expr, eval_source, primitive_env};
use reciplexa_eval::value::RuntimeValue;

#[test]
fn syntax_error_placeholder_and_unbound_var() {
    let err = eval_expr(&CoreExpr::Error, &HashMap::new(), &mut UnitHost);
    assert!(err.is_err());
    let err = eval_expr(
        &CoreExpr::Var("missing".into()),
        &HashMap::new(),
        &mut UnitHost,
    );
    assert!(err.is_err());
}

#[test]
fn dead_cell_and_forward_binder_errors() {
    let mut env = HashMap::new();
    env.insert(
        "c".into(),
        RuntimeValue::Cell {
            value: Rc::new(RefCell::new(RuntimeValue::Int(1))),
            alive: Rc::new(Cell::new(false)),
        },
    );
    assert!(eval_expr(&CoreExpr::Var("c".into()), &env, &mut UnitHost).is_err());

    env.insert("k".into(), RuntimeValue::Int(0));
    assert!(eval_expr(
        &CoreExpr::Forward {
            resume_name: "k".into(),
        },
        &env,
        &mut UnitHost,
    )
    .is_err());
}

#[test]
fn cast_fail_and_perform_resume_chain() {
    let mut env = HashMap::new();
    let cast = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::String("nope".into()))),
        evidence: CastEvidence::TagCheck { tag: "int".into() },
        target: CoreType::Int,
        cast_id: 0,
    };
    assert!(eval_expr(&cast, &env, &mut UnitHost).is_err());

    struct ChainHost;
    impl EffectHost for ChainHost {
        fn perform(&mut self, op: &str, arg: RuntimeValue) -> EvalResult {
            if op == "step" {
                if let RuntimeValue::Int(n) = arg {
                    return Ok(RuntimeValue::Int(n + 1));
                }
            }
            Ok(arg)
        }
    }
    let resume: ResumeCont = Rc::new(|v, _host| Ok(Outcome::Value(v)));
    env.insert(
        "k".into(),
        RuntimeValue::OneShotResume {
            used: Rc::new(Cell::new(false)),
            cont: Rc::clone(&resume),
        },
    );
    let form = CoreExpr::Handle {
        op: "step".into(),
        handler_params: vec!["_".into(), "r".into()],
        handler_body: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("r".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(0))],
        }),
        body: Box::new(CoreExpr::Perform {
            op: "step".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        }),
    };
    let _ = eval_expr(&form, &env, &mut ChainHost);
}

#[test]
fn builtin_unicode_bytes_and_predicates() {
    let env = primitive_env();
    let mut host = UnitHost;
    let bad_unicode = CoreExpr::App {
        fun: Box::new(CoreExpr::Var("unicode".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::String("x".into()))],
    };
    assert!(eval_expr(&bad_unicode, &env, &mut host).is_err());

    let bad_encode = CoreExpr::App {
        fun: Box::new(CoreExpr::Var("encode-utf8".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
    };
    assert!(eval_expr(&bad_encode, &env, &mut host).is_err());

    let bad_decode = CoreExpr::App {
        fun: Box::new(CoreExpr::Var("decode-utf8".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::String("x".into()))],
    };
    assert!(eval_expr(&bad_decode, &env, &mut host).is_err());

    let ok_decode = CoreExpr::App {
        fun: Box::new(CoreExpr::Var("decode-utf8".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::Bytes(vec![0xff, 0xfe]))],
    };
    let v = eval_expr(&ok_decode, &env, &mut host).unwrap();
    assert!(matches!(
        v,
        RuntimeValue::Variant { tag, .. } if tag == "err"
    ));

    for (name, lit) in [
        ("is-number", CoreLiteral::String("x".into())),
        ("is-string", CoreLiteral::Int(1)),
        ("is-bool", CoreLiteral::Int(1)),
        ("is-none", CoreLiteral::Int(1)),
        ("is-some", CoreLiteral::Int(1)),
    ] {
        let expr = CoreExpr::App {
            fun: Box::new(CoreExpr::Var(name.into())),
            args: vec![CoreExpr::Lit(lit)],
        };
        let _ = eval_expr(&expr, &env, &mut host);
    }

    let _ = eval_source(r#"(val main (unicode 65))"#);
    let _ = eval_source(r#"(val main (encode-utf8 "hi"))"#);
}
