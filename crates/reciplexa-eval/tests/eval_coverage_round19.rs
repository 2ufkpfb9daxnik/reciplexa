//! Round-19 eval: Cont Forward on rest-args / numeric + predicate leftovers
//! still under eval.rs miss set after round18.

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

fn call_k() -> CoreExpr {
    CoreExpr::App {
        fun: Box::new(CoreExpr::Var("k".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
    }
}

#[test]
fn eval_round19_forward_rest_args_and_predicates() {
    let mut env = primitive_env();
    env.insert("k".into(), forward_k());

    // App: Cont Forward on 2nd arg (rest loop `other`)
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1)), call_k()],
        },
        &env,
        &mut UnitHost,
    );

    // Record literal: Cont Forward on 2nd field
    let _ = eval_expr(
        &CoreExpr::Record {
            fields: vec![
                ("a".into(), CoreExpr::Lit(CoreLiteral::Int(1))),
                ("b".into(), call_k()),
            ],
        },
        &env,
        &mut UnitHost,
    );

    // Seq rest Cont Forward after first value
    let _ = eval_expr(
        &CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::Int(1)),
            call_k(),
            CoreExpr::Lit(CoreLiteral::Int(3)),
        ]),
        &env,
        &mut UnitHost,
    );

    // Predicate builtins
    for (op, lit) in [
        ("number?", CoreLiteral::Int(1)),
        ("string?", CoreLiteral::String("x".into())),
        ("bool?", CoreLiteral::Bool(true)),
        ("is-none", CoreLiteral::Unit),
        ("is-some", CoreLiteral::Unit),
    ] {
        let _ = eval_expr(
            &CoreExpr::App {
                fun: Box::new(CoreExpr::Var(op.into())),
                args: vec![CoreExpr::Lit(lit)],
            },
            &env,
            &mut UnitHost,
        );
    }

    // Numeric binop mixed Int/F64 + Div via surface vars
    for (op, a, b) in [
        ("+", CoreLiteral::Int(1), CoreLiteral::F64(2.5)),
        ("/", CoreLiteral::F64(4.0), CoreLiteral::Int(2)),
        ("<", CoreLiteral::Number(1.0), CoreLiteral::F64(2.0)),
        ("int-div", CoreLiteral::Int(7), CoreLiteral::Int(2)),
        ("mod", CoreLiteral::Int(7), CoreLiteral::Int(3)),
    ] {
        let _ = eval_expr(
            &CoreExpr::App {
                fun: Box::new(CoreExpr::Var(op.into())),
                args: vec![CoreExpr::Lit(a), CoreExpr::Lit(b)],
            },
            &env,
            &mut UnitHost,
        );
    }

    // as_numeric Err
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![
                CoreExpr::Lit(CoreLiteral::String("a".into())),
                CoreExpr::Lit(CoreLiteral::Int(1)),
            ],
        },
        &env,
        &mut UnitHost,
    );

    // Unicode scalar from Int + bad
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("unicode".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(0x41))],
        },
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("unicode".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(-1))],
        },
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("unicode".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::F64(65.0))],
        },
        &env,
        &mut UnitHost,
    );

    // Cast evidence NumericPromote / Compose
    let _ = eval_expr(
        &CoreExpr::Cast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            evidence: CastEvidence::NumericPromote,
            target: CoreType::F64,
            cast_id: 9,
        },
        &env,
        &mut UnitHost,
    );
    let _ = eval_expr(
        &CoreExpr::Cast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.5))),
            evidence: CastEvidence::Compose(vec![
                CastEvidence::Widen,
                CastEvidence::NumericPromote,
            ]),
            target: CoreType::F64,
            cast_id: 10,
        },
        &env,
        &mut UnitHost,
    );

    // Handle with Forward in handler body (deep Cont other)
    let _ = eval_expr(
        &CoreExpr::Handle {
            op: "ask".into(),
            handler_params: vec!["m".into(), "k".into()],
            handler_body: Box::new(call_k()),
            body: Box::new(CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String("q".into()))),
            }),
        },
        &env,
        &mut UnitHost,
    );

    // Match tuple pattern miss then wildcard
    let _ = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Record {
                fields: vec![
                    ("0".into(), CoreExpr::Lit(CoreLiteral::Int(1))),
                    ("1".into(), CoreExpr::Lit(CoreLiteral::Int(2))),
                ],
            }),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Tuple(vec![
                        CorePattern::Lit(CoreLiteral::Int(1)),
                        CorePattern::Lit(CoreLiteral::Int(9)),
                    ]),
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
    );
}
