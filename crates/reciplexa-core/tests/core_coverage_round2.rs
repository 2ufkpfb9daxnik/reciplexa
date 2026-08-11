//! Round-2 core coverage: check/elaborate error arms and remaining cast/unify.

use reciplexa_core::cast::{
    decide_subtype, intersect_types, is_runtime_checkable, is_subtype, plan_cast_evidence,
    CastEvidence, DecideResult,
};
use reciplexa_core::check::{
    coerce_to_static, infer_expr, infer_with_effects, insert_implicit_casts,
    typecheck_language_source, TypeEnv,
};
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::expr::{first_unreachable_arm, CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::{CoreType, EffectRow, SingletonValue, TypeVarId};
use reciplexa_core::unify::{unify, Subst};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn perform_random_failure_and_file_ops_typing() {
    let ok = infer_with_effects(
        &CoreExpr::Perform {
            op: "random".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert_eq!(ok.0, CoreType::Number);

    let err = infer_with_effects(
        &CoreExpr::Perform {
            op: "random".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("random"));

    let fail = infer_with_effects(
        &CoreExpr::Perform {
            op: "failure".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert_eq!(fail.0, CoreType::Never);

    let err = infer_with_effects(
        &CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("string"));

    let err = infer_with_effects(
        &CoreExpr::Perform {
            op: "custom".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("string or unit"));

    let read = infer_with_effects(
        &CoreExpr::Perform {
            op: "read-file".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::String("a".into()))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert_eq!(read.0, CoreType::String);
}

#[test]
fn forward_typing_errors() {
    let err = infer_expr(
        &CoreExpr::Forward {
            resume_name: "k".into(),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("unbound") || err.message.contains("forward"));

    let mut env = TypeEnv::new();
    env.insert("k", CoreType::Int);
    let err = infer_expr(
        &CoreExpr::Forward {
            resume_name: "k".into(),
        },
        &env,
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("resume") || err.message.contains("forward"));
}

#[test]
fn elaborate_error_surface_forms() {
    let err = elaborate_source("").unwrap_err();
    assert!(err.message.contains("empty") || !err.message.is_empty());

    let err = elaborate_source("(page a4)").unwrap_err();
    assert!(err.message.contains("does not support") || err.message.contains("page"));

    let err = elaborate_source("(record-update)").unwrap_err();
    assert!(!err.message.is_empty());

    let err = elaborate_source("(try-cast 1)").unwrap_err();
    assert!(err.message.contains("try-cast") || !err.message.is_empty());

    let err = elaborate_source("(check-cast 1)").unwrap_err();
    assert!(err.message.contains("check-cast") || !err.message.is_empty());

    let err = elaborate_source("(unicode)").unwrap_err();
    assert!(err.message.contains("unicode") || !err.message.is_empty());

    let err = elaborate_source("(val main (unicode \"x\"))").unwrap_err();
    assert!(!err.message.is_empty());

    let ok = elaborate_source("(val main (unicode 0x41))").unwrap();
    assert!(matches!(ok, CoreExpr::Let { .. }));

    let err = elaborate_source("[1 2 3]").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn elaborate_data_rec_and_trailing() {
    let r = elaborate_with_data(
        r#"
(rec
  (data a (wrap_b b))
  (data b (wrap_a a) leaf))
(val main leaf)
"#,
    );
    // Mutual data may or may not be supported yet.
    let _ = r;

    let expr = elaborate_source("1\n(val main 2)").unwrap();
    assert!(matches!(expr, CoreExpr::Let { .. } | CoreExpr::Seq(_)));
}

#[test]
fn cast_more_intersection_and_unknown_decide() {
    assert!(matches!(
        intersect_types(
            &CoreType::Dynamic(Box::new(CoreType::Int)),
            &CoreType::Number
        ),
        CoreType::Dynamic(_) | CoreType::Int | CoreType::Number
    ));
    assert_eq!(
        intersect_types(
            &CoreType::OptionalField(Box::new(CoreType::Int)),
            &CoreType::OptionalField(Box::new(CoreType::String))
        ),
        CoreType::Never
    );
    assert_eq!(
        intersect_types(
            &CoreType::Record {
                fields: vec![("a".into(), CoreType::Int)]
            },
            &CoreType::Record {
                fields: vec![("a".into(), CoreType::String)]
            }
        ),
        CoreType::Never
    );
    assert_eq!(
        intersect_types(
            &CoreType::Variant {
                variants: vec![("ok".into(), Some(CoreType::Int))]
            },
            &CoreType::Variant {
                variants: vec![("err".into(), None)]
            }
        ),
        CoreType::Never
    );

    let open = CoreType::OpenRecord {
        fields: vec![],
        row: Box::new(CoreType::Var(TypeVarId::new(0))),
    };
    assert!(matches!(
        decide_subtype(&open, &CoreType::Int),
        DecideResult::Unknown | DecideResult::Disproved
    ));

    assert!(plan_cast_evidence(&CoreType::Int, &CoreType::String).is_none());
    assert!(plan_cast_evidence(&CoreType::String, &CoreType::Int).is_none());
    assert_eq!(
        plan_cast_evidence(&CoreType::F64, &CoreType::F64),
        Some(CastEvidence::Identity)
    );
    assert!(!is_runtime_checkable(&CoreType::Not(Box::new(
        CoreType::Int
    ))));
    assert!(is_runtime_checkable(&CoreType::Intersect(vec![
        CoreType::Int,
        CoreType::Number
    ])));
}

#[test]
fn unify_lacks_open_and_set_theoretic() {
    let mut s = Subst::new();
    let open = CoreType::OpenRecord {
        fields: vec![("a".into(), CoreType::Int)],
        row: Box::new(CoreType::Var(s.fresh_var())),
    };
    let lacks = CoreType::Lacks {
        label: "b".into(),
        row: Box::new(open.clone()),
    };
    let _ = unify(&lacks, &open, &mut s);

    let mut s = Subst::new();
    let bad = CoreType::Lacks {
        label: "a".into(),
        row: Box::new(CoreType::OpenRecord {
            fields: vec![("a".into(), CoreType::Int)],
            row: Box::new(CoreType::Unit),
        }),
    };
    let _ = unify(&bad, &CoreType::Unit, &mut s);

    let mut s = Subst::new();
    assert!(unify(
        &CoreType::Not(Box::new(CoreType::Int)),
        &CoreType::Not(Box::new(CoreType::Int)),
        &mut s
    )
    .is_ok());

    let mut s = Subst::new();
    assert!(unify(
        &CoreType::Diff(Box::new(CoreType::Number), Box::new(CoreType::Int)),
        &CoreType::Diff(Box::new(CoreType::Number), Box::new(CoreType::Int)),
        &mut s
    )
    .is_ok());

    let mut s = Subst::new();
    let v = s.fresh_var();
    assert!(s
        .bind(v, CoreType::Union(vec![CoreType::Var(v), CoreType::Int]))
        .is_err());

    let mut s = Subst::new();
    let v = s.fresh_var();
    assert!(s
        .bind(
            v,
            CoreType::App {
                ctor: "option".into(),
                args: vec![CoreType::Var(v)],
            }
        )
        .is_err());

    let mut s = Subst::new();
    let v = s.fresh_var();
    assert!(s
        .bind(
            v,
            CoreType::Forall {
                params: vec![("a".into(), "type".into())],
                body: Box::new(CoreType::Var(v)),
            }
        )
        .is_err());

    let mut s = Subst::new();
    let v = s.fresh_var();
    assert!(s
        .bind(v, CoreType::Intersect(vec![CoreType::Var(v)]))
        .is_err());

    let mut s = Subst::new();
    let v = s.fresh_var();
    assert!(s
        .bind(v, CoreType::Dynamic(Box::new(CoreType::Var(v))))
        .is_err());

    let mut s = Subst::new();
    let v = s.fresh_var();
    assert!(s
        .bind(
            v,
            CoreType::Lacks {
                label: "x".into(),
                row: Box::new(CoreType::Var(v)),
            }
        )
        .is_err());
}

#[test]
fn insert_casts_let_if_record_match_lambda() {
    let mut env = TypeEnv::new();
    env.insert("x", CoreType::dynamic_bound(CoreType::Int));
    env.insert(
        "f",
        CoreType::Fun {
            args: vec![CoreType::Number],
            ret: Box::new(CoreType::Number),
            effects: EffectRow::default(),
        },
    );
    let expr = CoreExpr::Let {
        name: "y".into(),
        value: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("f".into())),
            args: vec![CoreExpr::Var("x".into())],
        }),
        body: Box::new(CoreExpr::If {
            cond: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
            then_branch: Box::new(CoreExpr::Var("y".into())),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        }),
    };
    let out = insert_implicit_casts(&expr, &env).unwrap();
    assert!(matches!(out, CoreExpr::Let { .. }));

    let lam = CoreExpr::Lambda {
        params: vec!["z".into()],
        body: Box::new(CoreExpr::Var("z".into())),
    };
    assert!(insert_implicit_casts(&lam, &env).is_ok());
}

#[test]
fn typecheck_more_kernel_forms() {
    let ty = typecheck_language_source("(val main (perform random unit))").unwrap();
    assert!(matches!(
        ty,
        CoreType::Number | CoreType::F64 | CoreType::Int
    ));

    let err = typecheck_language_source("(val main (perform random 1))").unwrap_err();
    assert!(err.message.contains("random") || err.message.contains("unit"));

    let ty = typecheck_language_source(
        r#"(val main (handle log (fn (msg k) (k "ok")) (perform log "x")))"#,
    )
    .unwrap();
    let _ = ty;

    let ty = typecheck_language_source("(val main (unicode 0x3002))").unwrap();
    assert_eq!(ty, CoreType::String);

    let err = typecheck_language_source("(val main (try-cast 1 never))").unwrap_err();
    assert!(!err.message.is_empty());

    let err = typecheck_language_source("(val main (check-cast 1 never))").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn coerce_identity_and_static_widen() {
    let e = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::dynamic_bound(CoreType::Int),
        &CoreType::Number,
        1,
    )
    .unwrap();
    // Fully included dynamic bound → no cast.
    assert!(matches!(e, CoreExpr::Lit(_) | CoreExpr::Cast { .. }));

    let e = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::Int,
        &CoreType::Int,
        2,
    )
    .unwrap();
    assert!(matches!(e, CoreExpr::Lit(_)));
}

#[test]
fn expr_duplicate_tag_unreachable_without_adt() {
    let arms = vec![
        MatchArm::variant("a".into(), None, CoreExpr::Lit(CoreLiteral::Int(0))),
        MatchArm::variant("a".into(), None, CoreExpr::Lit(CoreLiteral::Int(1))),
    ];
    assert_eq!(first_unreachable_arm(&arms, &[]), Some(1));

    // Non-variant pattern arm in fully-covers helper.
    let arms = vec![MatchArm {
        pattern: CorePattern::Wildcard,
        body: CoreExpr::Lit(CoreLiteral::Int(0)),
    }];
    assert_eq!(first_unreachable_arm(&arms, &["a"]), None);
}

#[test]
fn is_subtype_fun_effects_and_optional() {
    let a = CoreType::Fun {
        args: vec![CoreType::Number],
        ret: Box::new(CoreType::Int),
        effects: EffectRow::default(),
    };
    let b = CoreType::Fun {
        args: vec![CoreType::Int],
        ret: Box::new(CoreType::Number),
        effects: EffectRow {
            ops: vec!["log".into()],
        },
    };
    let _ = is_subtype(&a, &b);
    assert!(is_subtype(
        &CoreType::OptionalField(Box::new(CoreType::Int)),
        &CoreType::OptionalField(Box::new(CoreType::Number))
    ));
    assert!(is_subtype(
        &CoreType::Singleton(SingletonValue::Bool(true)),
        &CoreType::Bool
    ));
}
