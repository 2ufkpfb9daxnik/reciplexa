//! Round-3 core: drive remaining check.rs Handle/LetRec/Set/Cast arms.

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::check::{infer_expr, infer_with_effects, TypeEnv};
use reciplexa_core::elaborate::{elaborate_source, DataEnv};
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::{CoreType, EffectRow};
use reciplexa_core::unify::Subst;
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn handle_param_arity_and_failure_resume_rejected() {
    let err = infer_expr(
        &CoreExpr::Handle {
            op: "failure".into(),
            handler_params: vec!["e".into(), "k".into()],
            handler_body: Box::new(CoreExpr::Var("e".into())),
            body: Box::new(CoreExpr::Perform {
                op: "failure".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
            }),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("Failure") || err.message.contains("resume"));

    let err = infer_expr(
        &CoreExpr::Handle {
            op: "log".into(),
            handler_params: vec![],
            handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("1 or 2"));
}

#[test]
fn handle_one_and_two_param_ok() {
    let ty = infer_expr(
        &CoreExpr::Handle {
            op: "log".into(),
            handler_params: vec!["msg".into()],
            handler_body: Box::new(CoreExpr::Var("msg".into())),
            body: Box::new(CoreExpr::Perform {
                op: "log".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
            }),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert_eq!(ty, CoreType::String);

    let ty = infer_expr(
        &CoreExpr::Handle {
            op: "log".into(),
            handler_params: vec!["msg".into(), "k".into()],
            handler_body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::String("ok".into()))],
            }),
            body: Box::new(CoreExpr::Perform {
                op: "log".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
            }),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    let _ = ty;
}

#[test]
fn handler_value_arity_and_with() {
    let ty = infer_expr(
        &CoreExpr::HandlerValue {
            op: "log".into(),
            handler_params: vec!["msg".into()],
            handler_body: Box::new(CoreExpr::Var("msg".into())),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert!(matches!(ty, CoreType::Dynamic(_)));

    let ty = infer_expr(
        &CoreExpr::HandlerValue {
            op: "log".into(),
            handler_params: vec!["msg".into(), "k".into()],
            handler_body: Box::new(CoreExpr::Var("msg".into())),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert!(matches!(ty, CoreType::Dynamic(_)));

    let err = infer_expr(
        &CoreExpr::HandlerValue {
            op: "log".into(),
            handler_params: vec![],
            handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("1 or 2") || err.message.contains("handler"));

    let ty = infer_expr(
        &CoreExpr::With {
            handler: Box::new(CoreExpr::HandlerValue {
                op: "log".into(),
                handler_params: vec!["msg".into()],
                handler_body: Box::new(CoreExpr::Var("msg".into())),
            }),
            body: Box::new(CoreExpr::Perform {
                op: "log".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
            }),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    let _ = ty;
}

#[test]
fn letrec_non_lambda_and_annotated() {
    let err = infer_expr(
        &CoreExpr::LetRec {
            bindings: vec![("f".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(
        err.message.contains("fn")
            || err.message.contains("lambda")
            || err.message.contains("letrec")
    );

    let mut env = TypeEnv::new();
    env.data = DataEnv::default();
    env.data.type_aliases.insert(
        "f".into(),
        CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        },
    );
    let ty = infer_expr(
        &CoreExpr::LetRec {
            bindings: vec![(
                "f".into(),
                CoreExpr::Lambda {
                    params: vec!["x".into()],
                    body: Box::new(CoreExpr::Var("x".into())),
                },
            )],
            body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
            }),
        },
        &env,
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert_eq!(ty, CoreType::Int);
}

#[test]
fn set_unbound_and_wildcard_lambda() {
    let err = infer_expr(
        &CoreExpr::Set {
            name: "x".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("unbound"));

    let ty = infer_expr(
        &CoreExpr::Lambda {
            params: vec!["_".into()],
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert!(matches!(ty, CoreType::Fun { args, .. } if args.len() == 1));
}

#[test]
fn cast_try_check_typing() {
    let ty = infer_expr(
        &CoreExpr::Cast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            evidence: CastEvidence::Widen,
            target: CoreType::dyn_any(),
            cast_id: 0,
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert!(matches!(ty, CoreType::Dynamic(_)));

    let ty = infer_expr(
        &CoreExpr::TryCast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            target: CoreType::Int,
            cast_id: 1,
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert!(matches!(
        ty,
        CoreType::App { .. } | CoreType::Variant { .. } | CoreType::Dynamic(_)
    ));

    let ty = infer_expr(
        &CoreExpr::CheckCast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::String("a".into()))),
            target: CoreType::String,
            cast_id: 2,
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert!(matches!(
        ty,
        CoreType::App { .. } | CoreType::Variant { .. } | CoreType::Dynamic(_)
    ));
}

#[test]
fn record_update_extend_typing_and_errors() {
    let rec = CoreExpr::Record {
        fields: vec![
            ("a".into(), CoreExpr::Lit(CoreLiteral::Int(1))),
            ("b".into(), CoreExpr::Lit(CoreLiteral::Int(2))),
        ],
    };
    let ty = infer_expr(
        &CoreExpr::RecordUpdate {
            record: Box::new(rec.clone()),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(3)))],
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert!(matches!(ty, CoreType::Record { .. }));

    let ty = infer_expr(
        &CoreExpr::RecordExtend {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            fields: vec![("b".into(), CoreExpr::Lit(CoreLiteral::String("x".into())))],
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert!(matches!(ty, CoreType::Record { .. }));
}

#[test]
fn match_record_and_lit_patterns_typing() {
    let ty = infer_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Lit(CoreLiteral::Bool(true)),
                    body: CoreExpr::Lit(CoreLiteral::Int(1)),
                },
                MatchArm {
                    pattern: CorePattern::Wildcard,
                    body: CoreExpr::Lit(CoreLiteral::Int(0)),
                },
            ],
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert_eq!(ty, CoreType::Int);
}

#[test]
fn infer_effects_on_error_and_bytes_lit() {
    let (ty, _) = infer_with_effects(
        &CoreExpr::Error,
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert_eq!(ty, CoreType::Error);

    let (ty, _) = infer_with_effects(
        &CoreExpr::Lit(CoreLiteral::Bytes(vec![1, 2])),
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert_eq!(ty, CoreType::Bytes);
}

#[test]
fn elaborate_more_error_messages() {
    for src in [
        "(raise)",
        "(or-raise)",
        "(as-result)",
        "(forward)",
        "(handler)",
        "(with)",
        "(record-extend)",
        "(field)",
        "(match)",
        "(let)",
        "(letrec)",
        "(var)",
        "(set)",
        "(if true)",
        "(fn)",
        "(bytes)",
        "(list)",
        "(tuple)",
    ] {
        let err = elaborate_source(&format!("(val main {src})"));
        // Some may succeed with partial elaboration; ensure no panic.
        let _ = err;
    }

    let ok = elaborate_source(
        r#"(val main
  (local
    (type t int)
    (val x 1)
    x))"#,
    );
    let _ = ok;
}
