//! Integration tests moved from src/check.rs for region coverage.

use reciplexa_core::check::*;
use reciplexa_core::expr::{CoreExpr, CoreLiteral, MatchArm};
use reciplexa_core::ty::*;
use reciplexa_core::unify::Subst;
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn infers_lambda_application() {
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Lambda {
            param: "x".into(),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        }),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Number);
}

#[test]
fn infers_record_get() {
    let expr = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Record {
            fields: vec![
                ("x".into(), CoreExpr::Lit(CoreLiteral::Number(1.0))),
                ("y".into(), CoreExpr::Lit(CoreLiteral::Number(2.0))),
            ],
        }),
        field: "y".into(),
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(ty, CoreType::Number);
}

#[test]
fn perform_requires_string_arg() {
    let expr = CoreExpr::Perform {
        op: "log".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
    };
    let mut subst = Subst::new();
    let err = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap_err();
    assert!(err.message.contains("string"));
}

#[test]
fn record_get_unknown_field_errors() {
    let expr = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Record {
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Number(1.0)))],
        }),
        field: "missing".into(),
    };
    let mut subst = Subst::new();
    let err = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap_err();
    assert!(err.message.contains("unknown field"));
}

#[test]
fn record_get_on_non_record_errors() {
    let expr = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        field: "x".into(),
    };
    let mut subst = Subst::new();
    let err = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap_err();
    assert!(err.message.contains("expected record"));
}

#[test]
fn infers_seq_returns_last() {
    let expr = CoreExpr::Seq(vec![
        CoreExpr::Lit(CoreLiteral::String("a".into())),
        CoreExpr::Lit(CoreLiteral::Number(2.0)),
    ]);
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(ty, CoreType::Number);
}

#[test]
fn infers_let_and_variant() {
    let expr = CoreExpr::Let {
        name: "v".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        body: Box::new(CoreExpr::Variant {
            tag: "Ok".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0)))),
        }),
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert!(matches!(ty, CoreType::Variant { .. }));
}

#[test]
fn typecheck_value_wraps_expr() {
    let expr = CoreExpr::Lit(CoreLiteral::Color("red".into()));
    let cv = typecheck_value(expr, &TypeEnv::new(), range()).unwrap();
    assert_eq!(cv.ty, CoreType::Color);
}

#[test]
fn type_env_insert() {
    let mut env = TypeEnv::new();
    env.insert("x", CoreType::Number);
    let expr = CoreExpr::Lit(CoreLiteral::Number(1.0));
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &env, &mut subst, range()).unwrap();
    assert_eq!(ty, CoreType::Number);
}

#[test]
fn match_arm_unifies_return_types() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "A".into(),
            payload: None,
        }),
        arms: vec![
            MatchArm {
                tag: "A".into(),
                bind: None,
                body: CoreExpr::Lit(CoreLiteral::Number(1.0)),
            },
            MatchArm {
                tag: "B".into(),
                bind: None,
                body: CoreExpr::Lit(CoreLiteral::Number(2.0)),
            },
        ],
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Number);
}

#[test]
fn perform_with_string_ok() {
    let expr = CoreExpr::Perform {
        op: "log".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("ok".into()))),
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(ty, CoreType::Unit);
}

#[test]
fn match_arm_return_type_mismatch() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "A".into(),
            payload: None,
        }),
        arms: vec![
            MatchArm {
                tag: "A".into(),
                bind: None,
                body: CoreExpr::Lit(CoreLiteral::Number(1.0)),
            },
            MatchArm {
                tag: "B".into(),
                bind: None,
                body: CoreExpr::Lit(CoreLiteral::String("no".into())),
            },
        ],
    };
    let mut subst = Subst::new();
    assert!(infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).is_err());
}

#[test]
fn match_binds_payload_in_arm() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "Some".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(5.0)))),
        }),
        arms: vec![MatchArm {
            tag: "Some".into(),
            bind: Some("v".into()),
            body: CoreExpr::Lit(CoreLiteral::Number(5.0)),
        }],
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Number);
}

fn bad_record_get() -> CoreExpr {
    CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        field: "missing".into(),
    }
}

#[test]
fn infer_propagates_nested_errors() {
    let bad = bad_record_get();
    let mut subst = Subst::new();
    let env = TypeEnv::new();
    let r = range();

    assert!(infer_expr(
        &CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(bad.clone()),
        },
        &env,
        &mut subst,
        r
    )
    .is_err());

    assert!(infer_expr(
        &CoreExpr::Seq(vec![CoreExpr::Lit(CoreLiteral::Number(1.0)), bad.clone(),]),
        &env,
        &mut subst,
        r
    )
    .is_err());

    assert!(infer_expr(
        &CoreExpr::Let {
            name: "x".into(),
            value: Box::new(bad.clone()),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
        },
        &env,
        &mut subst,
        r
    )
    .is_err());

    assert!(infer_expr(
        &CoreExpr::Record {
            fields: vec![("a".into(), bad.clone())],
        },
        &env,
        &mut subst,
        r
    )
    .is_err());

    assert!(infer_expr(
        &CoreExpr::Variant {
            tag: "Some".into(),
            payload: Some(Box::new(bad.clone())),
        },
        &env,
        &mut subst,
        r
    )
    .is_err());

    assert!(infer_expr(
        &CoreExpr::Let {
            name: "x".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            body: Box::new(bad.clone()),
        },
        &env,
        &mut subst,
        r
    )
    .is_err());

    assert!(infer_expr(
        &CoreExpr::Lambda {
            param: "x".into(),
            body: Box::new(bad.clone()),
        },
        &env,
        &mut subst,
        r
    )
    .is_err());

    assert!(infer_expr(
        &CoreExpr::App {
            fun: Box::new(bad.clone()),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        },
        &env,
        &mut subst,
        r
    )
    .is_err());

    assert!(infer_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                param: "x".into(),
                body: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
            }),
            arg: Box::new(bad.clone()),
        },
        &env,
        &mut subst,
        r
    )
    .is_err());

    assert!(infer_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(bad.clone()),
            arms: vec![MatchArm {
                tag: "A".into(),
                bind: None,
                body: CoreExpr::Lit(CoreLiteral::Number(0.0)),
            }],
        },
        &env,
        &mut subst,
        r
    )
    .is_err());

    assert!(infer_expr(
        &CoreExpr::RecordGet {
            record: Box::new(bad),
            field: "x".into(),
        },
        &env,
        &mut subst,
        r
    )
    .is_err());
}

#[test]
fn app_non_function_unify_fails() {
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
    };
    let err = infer_expr(&expr, &TypeEnv::new(), &mut Subst::new(), range()).unwrap_err();
    assert!(err.message.contains("Mismatch"));
}

#[test]
fn typecheck_value_propagates_infer_error() {
    let expr = bad_record_get();
    assert!(typecheck_value(expr, &TypeEnv::new(), range()).is_err());
}

#[test]
fn infers_empty_seq_and_lambda() {
    let mut subst = Subst::new();
    let empty = infer_expr(&CoreExpr::Seq(vec![]), &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(empty, CoreType::Unit);

    let lam = CoreExpr::Lambda {
        param: "x".into(),
        body: Box::new(CoreExpr::Lit(CoreLiteral::String("ok".into()))),
    };
    let ty = infer_expr(&lam, &TypeEnv::new(), &mut subst, range()).unwrap();
    if let CoreType::Fun { args, .. } = ty {
        assert_eq!(args.len(), 1);
    } else {
        panic!("expected Fun");
    }
}

#[test]
fn match_on_non_variant_scrutinee() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        arms: vec![MatchArm {
            tag: "ignored".into(),
            bind: None,
            body: CoreExpr::Lit(CoreLiteral::Number(2.0)),
        }],
    };
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut Subst::new(), range()).unwrap();
    assert_eq!(ty, CoreType::Number);
}

#[test]
fn check_arm_body_infer_error() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "A".into(),
            payload: None,
        }),
        arms: vec![MatchArm {
            tag: "A".into(),
            bind: None,
            body: bad_record_get(),
        }],
    };
    assert!(infer_expr(&expr, &TypeEnv::new(), &mut Subst::new(), range()).is_err());
}

#[test]
fn match_arm_bind_without_payload_and_unknown_tag() {
    // Nullary variant + bind name: no payload to insert.
    let nullary = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "None".into(),
            payload: None,
        }),
        arms: vec![MatchArm {
            tag: "None".into(),
            bind: Some("x".into()),
            body: CoreExpr::Lit(CoreLiteral::Number(0.0)),
        }],
    };
    let mut subst = Subst::new();
    assert!(infer_expr(&nullary, &TypeEnv::new(), &mut subst, range()).is_ok());

    // Arm tag absent from scrutinee type still typechecks the body.
    let mismatch_tag = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "A".into(),
            payload: None,
        }),
        arms: vec![MatchArm {
            tag: "B".into(),
            bind: Some("x".into()),
            body: CoreExpr::Lit(CoreLiteral::Number(1.0)),
        }],
    };
    let mut subst = Subst::new();
    assert!(infer_expr(&mismatch_tag, &TypeEnv::new(), &mut subst, range()).is_ok());
}
