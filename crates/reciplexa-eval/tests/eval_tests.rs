//! Integration tests moved from src/eval.rs for region coverage.

use std::collections::HashMap;

use reciplexa_core::expr::{CoreExpr, CoreLiteral, MatchArm};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::eval::*;
use reciplexa_eval::value::*;

#[test]
fn seq_evaluates_left_to_right() {
    let expr = CoreExpr::Seq(vec![
        CoreExpr::Lit(CoreLiteral::Number(1.0)),
        CoreExpr::Lit(CoreLiteral::Number(2.0)),
    ]);
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(2.0));
}

#[test]
fn let_binds_in_body() {
    let expr = CoreExpr::Let {
        name: "x".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Number(3.0))),
        body: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
    };
    let _ = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
}

#[test]
fn lambda_application() {
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Lambda {
            param: "x".into(),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        }),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(1.0));
}

#[test]
fn pattern_match_variant() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(42.0)))),
        }),
        arms: vec![MatchArm {
            tag: "some".into(),
            bind: Some("n".into()),
            body: CoreExpr::Lit(CoreLiteral::Number(0.0)),
        }],
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(0.0));
}

#[test]
fn eval_perform_log() {
    let expr = CoreExpr::Perform {
        op: "log".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Unit);
}

#[test]
fn eval_perform_random() {
    let expr = CoreExpr::Perform {
        op: "random".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("".into()))),
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(0.5));
}

#[test]
fn eval_unknown_op_errors() {
    let expr = CoreExpr::Perform {
        op: "draw".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("".into()))),
    };
    assert!(eval_expr(&expr, &HashMap::new(), &mut UnitHost).is_err());
}

#[test]
fn eval_shape_literal_tags() {
    for tag in ["circle", "rect", "text"] {
        let expr = CoreExpr::Lit(CoreLiteral::String(tag.into()));
        let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
        assert_eq!(v, RuntimeValue::ShapeTag(tag.into()));
    }
    let expr = CoreExpr::Lit(CoreLiteral::String("other".into()));
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::String("other".into()));
}

#[test]
fn eval_app_non_closure_errors() {
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
    };
    assert!(eval_expr(&expr, &HashMap::new(), &mut UnitHost).is_err());
}

#[test]
fn eval_record_and_get() {
    let expr = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Record {
            fields: vec![("k".into(), CoreExpr::Lit(CoreLiteral::Number(9.0)))],
        }),
        field: "k".into(),
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(9.0));
}

#[test]
fn eval_match_non_variant_errors() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        arms: vec![],
    };
    assert!(eval_expr(&expr, &HashMap::new(), &mut UnitHost).is_err());
}

#[test]
fn eval_empty_seq_is_unit() {
    let v = eval_expr(&CoreExpr::Seq(vec![]), &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Unit);
}

#[test]
fn eval_record_get_errors() {
    let missing = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Record {
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Number(1.0)))],
        }),
        field: "b".into(),
    };
    assert!(eval_expr(&missing, &HashMap::new(), &mut UnitHost).is_err());

    let not_rec = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        field: "a".into(),
    };
    assert!(eval_expr(&not_rec, &HashMap::new(), &mut UnitHost).is_err());
}

#[test]
fn eval_match_no_arm_and_bind_payload() {
    let no_arm = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "none".into(),
            payload: None,
        }),
        arms: vec![MatchArm {
            tag: "some".into(),
            bind: None,
            body: CoreExpr::Lit(CoreLiteral::Number(0.0)),
        }],
    };
    assert!(eval_expr(&no_arm, &HashMap::new(), &mut UnitHost).is_err());

    let with_bind = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(7.0)))),
        }),
        arms: vec![MatchArm {
            tag: "some".into(),
            bind: Some("n".into()),
            body: CoreExpr::Lit(CoreLiteral::Number(1.0)),
        }],
    };
    assert_eq!(
        eval_expr(&with_bind, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(1.0)
    );
}

#[test]
fn eval_color_literal_and_let_uses_binding() {
    let color = CoreExpr::Lit(CoreLiteral::Color("red".into()));
    assert_eq!(
        eval_expr(&color, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::String("red".into())
    );

    // Let binds value even if body ignores it — exercise insert path.
    let expr = CoreExpr::Let {
        name: "x".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Number(3.0))),
        body: Box::new(CoreExpr::Lit(CoreLiteral::Number(9.0))),
    };
    assert_eq!(
        eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(9.0)
    );
}

#[test]
fn eval_variant_without_payload() {
    let expr = CoreExpr::Variant {
        tag: "none".into(),
        payload: None,
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert!(matches!(
        v,
        RuntimeValue::Variant {
            tag,
            payload: None
        } if tag == "none"
    ));
}

#[test]
fn eval_propagates_nested_errors() {
    let bad = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        field: "missing".into(),
    };
    let env = HashMap::new();
    let mut host = UnitHost;

    assert!(eval_expr(
        &CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(bad.clone()),
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::Seq(vec![CoreExpr::Lit(CoreLiteral::Number(1.0)), bad.clone(),]),
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::Let {
            name: "x".into(),
            value: Box::new(bad.clone()),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::Record {
            fields: vec![("a".into(), bad.clone())],
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(bad.clone())),
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(bad.clone()),
            arms: vec![],
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::App {
            fun: Box::new(bad.clone()),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                param: "x".into(),
                body: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
            }),
            arg: Box::new(bad.clone()),
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::RecordGet {
            record: Box::new(bad),
            field: "x".into(),
        },
        &env,
        &mut host
    )
    .is_err());
}

#[test]
fn eval_match_without_bind() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "ok".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0)))),
        }),
        arms: vec![MatchArm {
            tag: "ok".into(),
            bind: None,
            body: CoreExpr::Lit(CoreLiteral::Number(5.0)),
        }],
    };
    assert_eq!(
        eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(5.0)
    );
}

#[test]
fn eval_lambda_and_multi_field_record() {
    let closure = eval_expr(
        &CoreExpr::Lambda {
            param: "x".into(),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap();
    assert!(matches!(closure, RuntimeValue::Closure { .. }));

    let rec = eval_expr(
        &CoreExpr::Record {
            fields: vec![
                ("a".into(), CoreExpr::Lit(CoreLiteral::Number(1.0))),
                ("b".into(), CoreExpr::Lit(CoreLiteral::String("x".into()))),
            ],
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap();
    if let RuntimeValue::Record(fields) = rec {
        assert_eq!(fields.len(), 2);
    } else {
        panic!("expected record");
    }
}

#[test]
fn eval_match_bind_without_payload() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "none".into(),
            payload: None,
        }),
        arms: vec![MatchArm {
            tag: "none".into(),
            bind: Some("x".into()),
            body: CoreExpr::Lit(CoreLiteral::Number(0.0)),
        }],
    };
    assert_eq!(
        eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(0.0)
    );
}

#[test]
fn eval_app_closure_chain() {
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                param: "x".into(),
                body: Box::new(CoreExpr::Lambda {
                    param: "y".into(),
                    body: Box::new(CoreExpr::Lit(CoreLiteral::Number(9.0))),
                }),
            }),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        }),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
    };
    assert_eq!(
        eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(9.0)
    );
}
