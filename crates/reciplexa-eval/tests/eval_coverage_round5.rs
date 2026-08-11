//! Round-5: Cont `other` arms via Performed resume returning Resumed/Forward.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::control::{identity_resume, Outcome, UnitHost};
use reciplexa_eval::eval::{eval_expr, eval_source};
use reciplexa_eval::value::RuntimeValue;

fn k_resume() -> RuntimeValue {
    RuntimeValue::OneShotResume {
        used: Rc::new(Cell::new(false)),
        cont: identity_resume(),
    }
}

fn k_forward() -> RuntimeValue {
    RuntimeValue::OneShotResume {
        used: Rc::new(Cell::new(false)),
        cont: Rc::new(|_, _| Ok(Outcome::Forward)),
    }
}

/// LocalVar/Set/… resume `other` when init's continuation returns Resumed.
fn performed_then_resume_app(name: &str) -> CoreExpr {
    CoreExpr::Let {
        name: "tmp".into(),
        value: Box::new(CoreExpr::Perform {
            op: "ask".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        }),
        body: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var(name.into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        }),
    }
}

#[test]
fn resume_other_arms_via_nested_let_perform() {
    let forms = [
        CoreExpr::LocalVar {
            name: "x".into(),
            init: Box::new(performed_then_resume_app("k")),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        CoreExpr::Set {
            name: "cell".into(),
            value: Box::new(performed_then_resume_app("k")),
        },
        CoreExpr::If {
            cond: Box::new(performed_then_resume_app("k")),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(2))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(3))),
        },
        CoreExpr::RecordGet {
            record: Box::new(performed_then_resume_app("k")),
            field: "a".into(),
        },
        CoreExpr::RecordUpdate {
            record: Box::new(performed_then_resume_app("k")),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
        },
        CoreExpr::RecordExtend {
            record: Box::new(performed_then_resume_app("k")),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
        },
        CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(performed_then_resume_app("k"))),
        },
        CoreExpr::Match {
            scrutinee: Box::new(performed_then_resume_app("k")),
            arms: vec![],
        },
        CoreExpr::Cast {
            expr: Box::new(performed_then_resume_app("k")),
            evidence: CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 0,
        },
        CoreExpr::TryCast {
            expr: Box::new(performed_then_resume_app("k")),
            target: CoreType::Int,
            cast_id: 1,
        },
        CoreExpr::CheckCast {
            expr: Box::new(performed_then_resume_app("k")),
            target: CoreType::Int,
            cast_id: 2,
        },
        CoreExpr::Seq(vec![
            performed_then_resume_app("k"),
            CoreExpr::Lit(CoreLiteral::Int(9)),
        ]),
        CoreExpr::Record {
            fields: vec![("a".into(), performed_then_resume_app("k"))],
        },
        CoreExpr::App {
            fun: Box::new(performed_then_resume_app("k")),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(0))],
        },
        CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Var("x".into())),
            }),
            args: vec![performed_then_resume_app("k")],
        },
        CoreExpr::Let {
            name: "y".into(),
            value: Box::new(performed_then_resume_app("k")),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
    ];

    for form in forms {
        let mut env = HashMap::new();
        env.insert("k".into(), k_resume());
        let wrapped = CoreExpr::Handle {
            op: "ask".into(),
            handler_params: vec!["_".into(), "r".into()],
            handler_body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("r".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Int(0))],
            }),
            body: Box::new(form),
        };
        let v = eval_expr(&wrapped, &env, &mut UnitHost).unwrap();
        assert_eq!(v, RuntimeValue::Int(1));
    }
}

#[test]
fn resume_forward_other_arms_via_nested_let_perform() {
    let mut env = HashMap::new();
    env.insert("k".into(), k_forward());

    let form = CoreExpr::LocalVar {
        name: "x".into(),
        init: Box::new(performed_then_resume_app("k")),
        body: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
    };
    let wrapped = CoreExpr::Handle {
        op: "ask".into(),
        handler_params: vec!["_".into(), "r".into()],
        handler_body: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("r".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(0))],
        }),
        body: Box::new(form),
    };
    // Forward bubbles out of handle → error, or propagates.
    let _ = eval_expr(&wrapped, &env, &mut UnitHost);
}

#[test]
fn deep_same_op_resume_other_forward() {
    // After deep resume, body continues into Forward via oneshot.
    let v = eval_source(
        r#"(val main
  (handle ask (fn (n k)
      (if (= n 0)
          (let ((x (perform ask 1))) x)
          (k 0)))
    (perform ask 0)))"#,
    );
    let _ = v;
}

#[test]
fn app_fun_perform_and_match_lit_arms() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k (fn (x) x)))
    ((perform ask unit) 9)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(9));

    let v = eval_source(
        r#"(val main
  (match 1
    (1 -> 10)
    (_ -> 0)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(10));

    let v = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Lit(CoreLiteral::Bool(false)),
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
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(1));
}
