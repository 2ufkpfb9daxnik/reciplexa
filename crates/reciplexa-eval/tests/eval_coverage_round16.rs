//! Round-16 eval: Cont `other` arms through LocalVar/Set/If/Seq/App/Cast and
//! residual builtin / match_pattern edges.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::control::{identity_resume, Outcome, ResumeCont, UnitHost};
use reciplexa_eval::eval::{eval_expr, eval_source, primitive_env};
use reciplexa_eval::value::{BuiltinOp, RuntimeValue};

fn oneshot(cont: ResumeCont) -> RuntimeValue {
    RuntimeValue::OneShotResume {
        used: Rc::new(Cell::new(false)),
        cont,
    }
}

#[test]
fn cont_other_through_local_set_if_seq_app() {
    // LocalVar init Cont → resume returns Performed (other)
    let mut env = primitive_env();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|v, _| {
            Ok(Outcome::Performed {
                op: "outer".into(),
                arg: v,
                resume: identity_resume(),
            })
        })),
    );
    let expr = CoreExpr::LocalVar {
        name: "c".into(),
        init: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        }),
        body: Box::new(CoreExpr::Var("c".into())),
    };
    let _ = eval_expr(&expr, &env, &mut UnitHost);

    // Set Cont → resume returns Forward (other)
    let alive = Rc::new(Cell::new(true));
    let mut env = HashMap::new();
    env.insert(
        "c".into(),
        RuntimeValue::Cell {
            value: Rc::new(RefCell::new(RuntimeValue::Int(0))),
            alive,
        },
    );
    env.insert("k".into(), oneshot(Rc::new(|_, _| Ok(Outcome::Forward))));
    let expr = CoreExpr::Set {
        name: "c".into(),
        value: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(9))],
        }),
    };
    let _ = eval_expr(&expr, &env, &mut UnitHost);

    // If cond Cont → resume returns non-Bool / Performed
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|_, _| Ok(Outcome::Value(RuntimeValue::Int(1))))),
    );
    let expr = CoreExpr::If {
        cond: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
        }),
        then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
    };
    let _ = eval_expr(&expr, &env, &mut UnitHost);

    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|v, _| {
            Ok(Outcome::Performed {
                op: "log".into(),
                arg: v,
                resume: identity_resume(),
            })
        })),
    );
    let expr = CoreExpr::If {
        cond: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Bool(true))],
        }),
        then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
    };
    let _ = eval_expr(&expr, &env, &mut UnitHost);

    // Seq Cont → resume returns Performed (other)
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|v, _| {
            Ok(Outcome::Performed {
                op: "log".into(),
                arg: v,
                resume: identity_resume(),
            })
        })),
    );
    let expr = CoreExpr::Seq(vec![
        CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::String("a".into()))],
        },
        CoreExpr::Lit(CoreLiteral::Int(2)),
    ]);
    let _ = eval_expr(&expr, &env, &mut UnitHost);

    // App fun Cont → resume returns Performed
    let mut env = primitive_env();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|_, _| {
            Ok(Outcome::Performed {
                op: "ask".into(),
                arg: RuntimeValue::Unit,
                resume: identity_resume(),
            })
        })),
    );
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
        }),
        args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
    };
    let _ = eval_expr(&expr, &env, &mut UnitHost);

    // App arg Cont → resume returns Performed mid-args
    let mut env = primitive_env();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|v, _| {
            Ok(Outcome::Performed {
                op: "ask".into(),
                arg: v,
                resume: identity_resume(),
            })
        })),
    );
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Var("+".into())),
        args: vec![
            CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Int(3))],
            },
            CoreExpr::Lit(CoreLiteral::Int(4)),
        ],
    };
    let _ = eval_expr(&expr, &env, &mut UnitHost);
}

#[test]
fn cont_other_through_cast_forms_and_surface_nested() {
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|v, _| {
            Ok(Outcome::Performed {
                op: "log".into(),
                arg: v,
                resume: identity_resume(),
            })
        })),
    );
    let inner = CoreExpr::App {
        fun: Box::new(CoreExpr::Var("k".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
    };
    let _ = eval_expr(
        &CoreExpr::Cast {
            expr: Box::new(inner.clone()),
            evidence: CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 1,
        },
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &CoreExpr::TryCast {
            expr: Box::new(inner.clone()),
            target: CoreType::Int,
            cast_id: 2,
        },
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &CoreExpr::CheckCast {
            expr: Box::new(inner),
            target: CoreType::Int,
            cast_id: 3,
        },
        &env,
        &mut UnitHost,
    );

    // Surface nested Cont other through local/set/if/seq
    let _ = eval_source(
        r#"(val main
  (handle outer (fn (msg) 1)
    (handle ask (fn (_ k) (k (perform outer unit)))
      (local (var c (perform ask unit)) c))))"#,
    );
    let _ = eval_source(
        r#"(val main
  (handle outer (fn (msg) 2)
    (handle ask (fn (_ k) (k (perform outer unit)))
      (local (var c 0)
        (seq (set c (perform ask unit)) c)))))"#,
    );
    let _ = eval_source(
        r#"(val main
  (handle outer (fn (msg) true)
    (handle ask (fn (_ k) (k (perform outer unit)))
      (if (perform ask unit) 1 0))))"#,
    );
    let _ = eval_source(
        r#"(val main
  (handle outer (fn (msg) 3)
    (handle ask (fn (_ k) (k (perform outer unit)))
      (seq (perform ask unit) 9))))"#,
    );
}

#[test]
fn match_pattern_and_builtin_residual_edges() {
    // Tuple / record / variant pattern miss leaves via CoreExpr Match
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        arms: vec![
            MatchArm {
                pattern: CorePattern::Tuple(vec![CorePattern::Bind("x".into())]),
                body: CoreExpr::Lit(CoreLiteral::Int(0)),
            },
            MatchArm {
                pattern: CorePattern::Record {
                    fields: vec![("a".into(), CorePattern::Bind("x".into()))],
                },
                body: CoreExpr::Lit(CoreLiteral::Int(0)),
            },
            MatchArm {
                pattern: CorePattern::Variant {
                    tag: "some".into(),
                    payload: Some(Box::new(CorePattern::Lit(CoreLiteral::Int(1)))),
                },
                body: CoreExpr::Lit(CoreLiteral::Int(0)),
            },
            MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(9)),
            },
        ],
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Int(9));

    // Happy tuple/record/variant matches
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Record {
            fields: vec![
                ("0".into(), CoreExpr::Lit(CoreLiteral::Int(1))),
                ("1".into(), CoreExpr::Lit(CoreLiteral::Int(2))),
            ],
        }),
        arms: vec![MatchArm {
            pattern: CorePattern::Tuple(vec![
                CorePattern::Bind("a".into()),
                CorePattern::Bind("b".into()),
            ]),
            body: CoreExpr::Var("a".into()),
        }],
    };
    let _ = eval_expr(&expr, &HashMap::new(), &mut UnitHost);

    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Record {
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(7)))],
        }),
        arms: vec![MatchArm {
            pattern: CorePattern::Record {
                fields: vec![("a".into(), CorePattern::Bind("x".into()))],
            },
            body: CoreExpr::Var("x".into()),
        }],
    };
    let _ = eval_expr(&expr, &HashMap::new(), &mut UnitHost);

    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "none".into(),
            payload: None,
        }),
        arms: vec![
            MatchArm {
                pattern: CorePattern::Variant {
                    tag: "none".into(),
                    payload: Some(Box::new(CorePattern::Wildcard)),
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

    // Builtin residual: int-div/mod zero + wrong arity + mixed numeric
    let _ = eval_source("(val main (int-div 10 0))");
    let _ = eval_source("(val main (mod 10 0))");
    let _ = eval_source("(val main (int-div 10))");
    let _ = eval_source("(val main (mod 1 2 3))");
    let _ = eval_source("(val main (int-div 1.5 2))");
    let _ = eval_source("(val main (+ 1 2.5))");
    let _ = eval_source("(val main (< 1.0 2))");
    let _ = eval_source("(val main (number? 1))");
    let _ = eval_source("(val main (string? \"x\"))");
    let _ = eval_source("(val main (bool? true))");
    let _ = eval_source("(data opt (none) (some x))\n(val main (is-none none))");
    let _ = eval_source("(data opt (none) (some x))\n(val main (is-some (some 1)))");
    let _ = eval_source("(val main (= 1 1))");
    let _ = eval_source("(val main (!= 1 2))");
    let _ = eval_source("(val main (decode-utf8 (bytes 255)))");
    let _ = eval_source("(val main (decode-utf8 1))");

    // Direct builtin apply via CoreExpr App on Builtin value
    let mut env = HashMap::new();
    env.insert("d".into(), RuntimeValue::Builtin(BuiltinOp::IntDiv));
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("d".into())),
            args: vec![
                CoreExpr::Lit(CoreLiteral::Int(8)),
                CoreExpr::Lit(CoreLiteral::Int(2)),
            ],
        },
        &env,
        &mut UnitHost,
    );
}

#[test]
fn runtime_cast_evidence_matrix_via_cast_expr() {
    let cases: Vec<(CastEvidence, CoreExpr)> = vec![
        (
            CastEvidence::NumericPromote,
            CoreExpr::Lit(CoreLiteral::Int(3)),
        ),
        (
            CastEvidence::TagCheck {
                tag: "string".into(),
            },
            CoreExpr::Lit(CoreLiteral::String("x".into())),
        ),
        (
            CastEvidence::UnionCheck {
                members: vec![CoreType::Int, CoreType::String],
            },
            CoreExpr::Lit(CoreLiteral::Int(1)),
        ),
        (
            CastEvidence::VariantCheck {
                variants: vec![("none".into(), None), ("some".into(), Some(CoreType::Int))],
            },
            CoreExpr::Variant {
                tag: "none".into(),
                payload: None,
            },
        ),
        (
            CastEvidence::RecordCheck {
                fields: vec![("a".into(), CoreType::Int)],
            },
            CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            },
        ),
        (
            CastEvidence::FunctionGuard {
                arity: 1,
                arg_casts: vec![],
                ret_cast: Box::new(CastEvidence::Identity),
            },
            CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Var("x".into())),
            },
        ),
        (
            CastEvidence::IntersectionCheck {
                members: vec![CoreType::Int],
            },
            CoreExpr::Lit(CoreLiteral::Int(1)),
        ),
        (
            CastEvidence::Compose(vec![CastEvidence::Widen, CastEvidence::Identity]),
            CoreExpr::Lit(CoreLiteral::Int(1)),
        ),
        (
            CastEvidence::NominalCheck { name: "int".into() },
            CoreExpr::Lit(CoreLiteral::Int(1)),
        ),
    ];
    for (evidence, expr) in cases {
        let cast = CoreExpr::Cast {
            expr: Box::new(expr),
            evidence,
            target: CoreType::dyn_any(),
            cast_id: 42,
        };
        let _ = eval_expr(&cast, &HashMap::new(), &mut UnitHost);
    }
}
