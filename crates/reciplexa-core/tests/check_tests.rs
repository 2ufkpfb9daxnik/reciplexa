//! Integration tests moved from src/check.rs for region coverage.

use reciplexa_core::check::*;
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
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
            params: vec!["x".into()],
            body: Box::new(CoreExpr::Var("x".into())),
        }),
        args: vec![CoreExpr::Lit(CoreLiteral::Number(2.0))],
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Number);
}

#[test]
fn infers_let_binding_via_var() {
    // let x = 1 in x
    let expr = CoreExpr::Let {
        name: "x".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        body: Box::new(CoreExpr::Var("x".into())),
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(ty, CoreType::Number);
}

#[test]
fn unbound_var_errors() {
    let expr = CoreExpr::Var("missing".into());
    let err = infer_expr(&expr, &TypeEnv::new(), &mut Subst::new(), range()).unwrap_err();
    assert!(err.message.contains("unbound variable"));
}

#[test]
fn type_env_lookup_for_var() {
    let mut env = TypeEnv::new();
    env.insert("x", CoreType::String);
    let ty = infer_expr(&CoreExpr::Var("x".into()), &env, &mut Subst::new(), range()).unwrap();
    assert_eq!(ty, CoreType::String);
}

#[test]
fn infers_if_with_bool() {
    let expr = CoreExpr::If {
        cond: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
        then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Number);
}

#[test]
fn if_requires_bool_cond() {
    let expr = CoreExpr::If {
        cond: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
    };
    assert!(infer_expr(&expr, &TypeEnv::new(), &mut Subst::new(), range()).is_err());
}

#[test]
fn if_branch_types_form_union_when_distinct() {
    // TYP if-branch: distinct branch types yield Union rather than hard error.
    let expr = CoreExpr::If {
        cond: Box::new(CoreExpr::Lit(CoreLiteral::Bool(false))),
        then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
    };
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut Subst::new(), range()).unwrap();
    assert!(matches!(ty, CoreType::Union(_)), "got {ty:?}");
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
            MatchArm::variant("A".into(), None, CoreExpr::Lit(CoreLiteral::Number(1.0))),
            MatchArm::variant("B".into(), None, CoreExpr::Lit(CoreLiteral::Number(2.0))),
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
            MatchArm::variant("A".into(), None, CoreExpr::Lit(CoreLiteral::Number(1.0))),
            MatchArm::variant(
                "B".into(),
                None,
                CoreExpr::Lit(CoreLiteral::String("no".into())),
            ),
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
        arms: vec![MatchArm::variant(
            "Some".into(),
            Some("v".into()),
            CoreExpr::Lit(CoreLiteral::Number(5.0)),
        )],
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Number);
}

#[test]
fn match_binds_record_fields_in_arm() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Record {
            fields: vec![
                (
                    "title".into(),
                    CoreExpr::Lit(CoreLiteral::String("Hi".into())),
                ),
                ("n".into(), CoreExpr::Lit(CoreLiteral::Number(1.0))),
            ],
        }),
        arms: vec![MatchArm {
            pattern: CorePattern::Record {
                fields: vec![
                    ("title".into(), CorePattern::Bind("t".into())),
                    ("n".into(), CorePattern::Bind("n".into())),
                ],
            },
            body: CoreExpr::Var("n".into()),
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
fn syntax_error_expr_suppresses_cascade() {
    let mut subst = Subst::new();
    let env = TypeEnv::new();
    let r = range();
    let expr = CoreExpr::Seq(vec![
        CoreExpr::Error,
        CoreExpr::Lit(CoreLiteral::Number(1.0)),
    ]);
    let ty = infer_expr(&expr, &env, &mut subst, r).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Number);
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
            params: vec!["x".into()],
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
            args: vec![CoreExpr::Lit(CoreLiteral::Number(1.0))],
        },
        &env,
        &mut subst,
        r
    )
    .is_err());

    assert!(infer_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
            }),
            args: vec![bad.clone()],
        },
        &env,
        &mut subst,
        r
    )
    .is_err());

    assert!(infer_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(bad.clone()),
            arms: vec![MatchArm::variant(
                "A".into(),
                None,
                CoreExpr::Lit(CoreLiteral::Number(0.0))
            )],
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
        args: vec![CoreExpr::Lit(CoreLiteral::Number(2.0))],
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
        params: vec!["x".into()],
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
        arms: vec![MatchArm::variant(
            "ignored".into(),
            None,
            CoreExpr::Lit(CoreLiteral::Number(2.0)),
        )],
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
        arms: vec![MatchArm::variant("A".into(), None, bad_record_get())],
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
        arms: vec![MatchArm::variant(
            "None".into(),
            Some("x".into()),
            CoreExpr::Lit(CoreLiteral::Number(0.0)),
        )],
    };
    let mut subst = Subst::new();
    assert!(infer_expr(&nullary, &TypeEnv::new(), &mut subst, range()).is_ok());

    // Arm tag absent from scrutinee type still typechecks the body.
    let mismatch_tag = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "A".into(),
            payload: None,
        }),
        arms: vec![MatchArm::variant(
            "B".into(),
            Some("x".into()),
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
        )],
    };
    let mut subst = Subst::new();
    assert!(infer_expr(&mismatch_tag, &TypeEnv::new(), &mut subst, range()).is_ok());
}

#[test]
fn infers_nary_lambda_application() {
    // (λ(x y). x) 1 2
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Lambda {
            params: vec!["x".into(), "y".into()],
            body: Box::new(CoreExpr::Var("x".into())),
        }),
        args: vec![
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
            CoreExpr::Lit(CoreLiteral::Number(2.0)),
        ],
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Number);
}
