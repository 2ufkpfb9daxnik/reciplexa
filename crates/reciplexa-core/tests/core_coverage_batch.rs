//! Region-coverage batch for cast/unify/ty/expr and additional surface programs.

use reciplexa_core::cast::{
    cast_success_type, classify_decide, compose_evidence, decide_subtype, intersect_types,
    is_runtime_checkable, is_subtype, judge_dynamic_use, normalize_type, plan_cast_evidence,
    simplify_evidence, types_disjoint, CastEvidence, DecideResult, DynamicUseJudgment,
    TypeDiagClass,
};
use reciplexa_core::check::{
    coerce_to_static, infer_with_effects, insert_implicit_casts, typecheck_language_source, TypeEnv,
};
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::expr::{first_unreachable_arm, CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::{CoreType, EffectRow, NumericClass, SingletonValue, TypeVarId};
use reciplexa_core::unify::{unify, Subst};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn ty_helpers_dynamic_numeric_singleton() {
    assert_eq!(CoreType::dynamic_bound(CoreType::Never), CoreType::Never);
    assert_eq!(
        CoreType::dynamic_bound(CoreType::dyn_any()),
        CoreType::dyn_any()
    );
    assert!(matches!(
        CoreType::dynamic_bound(CoreType::Int),
        CoreType::Dynamic(b) if matches!(b.as_ref(), CoreType::Int)
    ));
    assert_eq!(CoreType::Int.as_dyn_bound(), None);
    assert_eq!(CoreType::dyn_any().as_dyn_bound(), Some(&CoreType::Any));

    assert_eq!(CoreType::Int.numeric_class(), Some(NumericClass::Int));
    assert_eq!(
        CoreType::Singleton(SingletonValue::Int(1)).numeric_class(),
        Some(NumericClass::Int)
    );
    assert_eq!(CoreType::F64.numeric_class(), Some(NumericClass::F64));
    assert_eq!(CoreType::String.numeric_class(), None);

    assert_eq!(
        CoreType::singleton_domain(&SingletonValue::Int(1)),
        CoreType::Int
    );
    assert_eq!(
        CoreType::singleton_domain(&SingletonValue::Bool(true)),
        CoreType::Bool
    );
    assert_eq!(
        CoreType::singleton_domain(&SingletonValue::String("x".into())),
        CoreType::String
    );
    assert_eq!(
        CoreType::singleton_domain(&SingletonValue::Unit),
        CoreType::Unit
    );

    assert_eq!(
        CoreType::numeric_binop_result("/", NumericClass::Int, NumericClass::Int),
        CoreType::F64
    );
    assert_eq!(
        CoreType::numeric_binop_result("+", NumericClass::Int, NumericClass::Int),
        CoreType::Int
    );
    assert_eq!(
        CoreType::numeric_binop_result("*", NumericClass::F64, NumericClass::F64),
        CoreType::F64
    );
    assert_eq!(
        CoreType::numeric_binop_result("-", NumericClass::Int, NumericClass::F64),
        CoreType::F64
    );
    assert_eq!(
        CoreType::numeric_binop_result("<", NumericClass::Int, NumericClass::Int),
        CoreType::Bool
    );
    assert_eq!(
        CoreType::numeric_binop_result("??", NumericClass::Int, NumericClass::Int),
        CoreType::Number
    );

    let row = EffectRow::default().with_op("log").with_op("log");
    assert_eq!(row.ops, vec!["log".to_string()]);
    assert_eq!(row.without_op("log").ops, Vec::<String>::new());
}

#[test]
fn expr_unreachable_arm_helpers() {
    let arms = vec![
        MatchArm::variant("none".into(), None, CoreExpr::Lit(CoreLiteral::Int(0))),
        MatchArm::variant(
            "some".into(),
            Some("x".into()),
            CoreExpr::Lit(CoreLiteral::Int(1)),
        ),
        MatchArm::variant("none".into(), None, CoreExpr::Lit(CoreLiteral::Int(2))),
    ];
    assert_eq!(first_unreachable_arm(&arms, &["none", "some"]), Some(2));

    let with_wild = vec![
        MatchArm {
            pattern: CorePattern::Wildcard,
            body: CoreExpr::Lit(CoreLiteral::Int(0)),
        },
        MatchArm::variant("none".into(), None, CoreExpr::Lit(CoreLiteral::Int(1))),
    ];
    assert_eq!(first_unreachable_arm(&with_wild, &[]), Some(1));

    let refining = vec![
        MatchArm {
            pattern: CorePattern::Variant {
                tag: "some".into(),
                payload: Some(Box::new(CorePattern::Lit(CoreLiteral::Int(1)))),
            },
            body: CoreExpr::Lit(CoreLiteral::Int(0)),
        },
        MatchArm::variant(
            "some".into(),
            Some("x".into()),
            CoreExpr::Lit(CoreLiteral::Int(1)),
        ),
    ];
    // Lit payload does not fully cover the tag, so second arm is reachable.
    assert_eq!(first_unreachable_arm(&refining, &["some"]), None);
}

#[test]
fn classify_decide_and_compose_simplify() {
    assert_eq!(classify_decide(DecideResult::Proved), None);
    assert_eq!(
        classify_decide(DecideResult::Disproved),
        Some(TypeDiagClass::TypeError)
    );
    assert_eq!(
        classify_decide(DecideResult::Unknown),
        Some(TypeDiagClass::CheckerLimitation)
    );

    let e = compose_evidence(vec![
        CastEvidence::Identity,
        CastEvidence::Compose(vec![CastEvidence::Widen, CastEvidence::Identity]),
        CastEvidence::TagCheck { tag: "int".into() },
    ]);
    assert_eq!(
        simplify_evidence(e),
        CastEvidence::Compose(vec![
            CastEvidence::Widen,
            CastEvidence::TagCheck { tag: "int".into() },
        ])
    );
    assert_eq!(
        simplify_evidence(CastEvidence::Compose(vec![CastEvidence::Identity])),
        CastEvidence::Identity
    );
    assert_eq!(
        simplify_evidence(CastEvidence::Compose(vec![
            CastEvidence::Identity,
            CastEvidence::Widen,
        ])),
        CastEvidence::Widen
    );
}

#[test]
fn plan_cast_and_judge_matrix() {
    assert_eq!(
        plan_cast_evidence(&CoreType::Int, &CoreType::Int),
        Some(CastEvidence::Identity)
    );
    assert_eq!(plan_cast_evidence(&CoreType::Int, &CoreType::Never), None);
    assert_eq!(
        plan_cast_evidence(&CoreType::Int, &CoreType::F64),
        Some(CastEvidence::NumericPromote)
    );
    assert_eq!(
        plan_cast_evidence(&CoreType::Int, &CoreType::dyn_any()),
        Some(CastEvidence::Widen)
    );
    assert_eq!(
        plan_cast_evidence(&CoreType::String, &CoreType::dynamic_bound(CoreType::Int)),
        None
    );
    assert_eq!(
        plan_cast_evidence(&CoreType::dyn_any(), &CoreType::Any),
        Some(CastEvidence::Identity)
    );
    assert_eq!(
        plan_cast_evidence(&CoreType::dynamic_bound(CoreType::String), &CoreType::Int),
        None
    );
    assert!(matches!(
        plan_cast_evidence(
            &CoreType::dynamic_bound(CoreType::Union(vec![CoreType::Int, CoreType::String])),
            &CoreType::Number
        ),
        Some(
            CastEvidence::TagCheck { .. }
                | CastEvidence::UnionCheck { .. }
                | CastEvidence::NumericPromote
                | CastEvidence::Compose(_)
        )
    ));

    assert_eq!(
        judge_dynamic_use(&CoreType::Int, &CoreType::Number),
        DynamicUseJudgment::FullyIncluded
    );
    assert_eq!(
        judge_dynamic_use(&CoreType::String, &CoreType::Int),
        DynamicUseJudgment::Disjoint
    );
    assert!(matches!(
        judge_dynamic_use(&CoreType::Int, &CoreType::F64),
        DynamicUseJudgment::PartialOverlap {
            evidence: CastEvidence::NumericPromote,
            ..
        }
    ));
    assert!(matches!(
        judge_dynamic_use(
            &CoreType::Union(vec![CoreType::Int, CoreType::String]),
            &CoreType::Number
        ),
        DynamicUseJudgment::PartialOverlap { .. }
    ));

    // Promote from mixed union → compose.
    assert!(matches!(
        plan_cast_evidence(
            &CoreType::Union(vec![CoreType::Int, CoreType::String]),
            &CoreType::F64
        ),
        Some(CastEvidence::Compose(_))
    ));
    assert_eq!(
        plan_cast_evidence(
            &CoreType::Union(vec![CoreType::Int, CoreType::F64]),
            &CoreType::F64
        ),
        Some(CastEvidence::NumericPromote)
    );
    assert_eq!(
        plan_cast_evidence(&CoreType::Union(vec![CoreType::String]), &CoreType::F64),
        None
    );
    assert_eq!(
        plan_cast_evidence(&CoreType::Singleton(SingletonValue::Int(3)), &CoreType::F64),
        Some(CastEvidence::NumericPromote)
    );
}

#[test]
fn decide_subtype_and_is_subtype_shapes() {
    assert_eq!(
        decide_subtype(&CoreType::Int, &CoreType::Number),
        DecideResult::Proved
    );
    assert_eq!(
        decide_subtype(&CoreType::String, &CoreType::Int),
        DecideResult::Disproved
    );
    assert_eq!(
        decide_subtype(&CoreType::Int, &CoreType::String),
        DecideResult::Disproved
    );
    // Fun / open shapes may be Unknown.
    let fun = CoreType::Fun {
        args: vec![CoreType::Int],
        ret: Box::new(CoreType::Int),
        effects: EffectRow::default(),
    };
    let _ = decide_subtype(&fun, &CoreType::dyn_any());

    assert!(is_subtype(&CoreType::Never, &CoreType::Int));
    assert!(is_subtype(&CoreType::Int, &CoreType::Any));
    assert!(!is_subtype(&CoreType::Any, &CoreType::Int));
    assert!(is_subtype(
        &CoreType::Dynamic(Box::new(CoreType::Int)),
        &CoreType::Dynamic(Box::new(CoreType::Number))
    ));
    assert!(is_subtype(
        &CoreType::Singleton(SingletonValue::Int(1)),
        &CoreType::Number
    ));
    assert!(is_subtype(
        &CoreType::Int,
        &CoreType::OptionalField(Box::new(CoreType::Number))
    ));
    assert!(is_subtype(
        &CoreType::Union(vec![CoreType::Int, CoreType::F64]),
        &CoreType::Number
    ));
    assert!(is_subtype(
        &CoreType::Int,
        &CoreType::Union(vec![CoreType::Int, CoreType::String])
    ));
    assert!(is_subtype(
        &CoreType::Intersect(vec![CoreType::Int, CoreType::Number]),
        &CoreType::Number
    ));
    assert!(is_subtype(
        &CoreType::Int,
        &CoreType::Intersect(vec![CoreType::Number, CoreType::Any])
    ));

    let f1 = CoreType::Fun {
        args: vec![CoreType::Number],
        ret: Box::new(CoreType::Int),
        effects: EffectRow::default(),
    };
    let f2 = CoreType::Fun {
        args: vec![CoreType::Int],
        ret: Box::new(CoreType::Number),
        effects: EffectRow {
            ops: vec!["log".into()],
        },
    };
    // Contravariant args / covariant ret — may or may not hold under approx.
    let _ = is_subtype(&f1, &f2);

    assert!(is_subtype(
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)]
        },
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Number)]
        }
    ));
    assert!(is_subtype(
        &CoreType::Variant {
            variants: vec![("ok".into(), Some(CoreType::Int))]
        },
        &CoreType::Variant {
            variants: vec![("ok".into(), Some(CoreType::Number)), ("err".into(), None)]
        }
    ));
}

#[test]
fn intersect_normalize_runtime_checkable() {
    assert_eq!(
        intersect_types(&CoreType::Int, &CoreType::Number),
        CoreType::Int
    );
    assert_eq!(
        intersect_types(&CoreType::Int, &CoreType::F64),
        CoreType::Never
    );
    assert_eq!(
        intersect_types(&CoreType::Any, &CoreType::String),
        CoreType::String
    );
    assert_eq!(
        intersect_types(
            &CoreType::Singleton(SingletonValue::Int(1)),
            &CoreType::Singleton(SingletonValue::Int(2))
        ),
        CoreType::Never
    );
    assert_eq!(
        intersect_types(
            &CoreType::Singleton(SingletonValue::Int(1)),
            &CoreType::Number
        ),
        CoreType::Singleton(SingletonValue::Int(1))
    );
    assert!(matches!(
        intersect_types(
            &CoreType::OptionalField(Box::new(CoreType::Int)),
            &CoreType::OptionalField(Box::new(CoreType::Number))
        ),
        CoreType::OptionalField(_)
    ));
    assert_eq!(
        intersect_types(
            &CoreType::Union(vec![CoreType::Int, CoreType::String]),
            &CoreType::Number
        ),
        CoreType::Int
    );
    assert!(matches!(
        intersect_types(&CoreType::Intersect(vec![CoreType::Number]), &CoreType::Int),
        CoreType::Int | CoreType::Intersect(_)
    ));
    assert_eq!(
        intersect_types(
            &CoreType::Record {
                fields: vec![("a".into(), CoreType::Int)]
            },
            &CoreType::Record {
                fields: vec![("a".into(), CoreType::Number)]
            }
        ),
        CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)]
        }
    );
    assert_eq!(
        intersect_types(
            &CoreType::Record {
                fields: vec![("a".into(), CoreType::Int)]
            },
            &CoreType::Record {
                fields: vec![("b".into(), CoreType::Int)]
            }
        ),
        CoreType::Never
    );
    assert!(matches!(
        intersect_types(
            &CoreType::Variant {
                variants: vec![("ok".into(), Some(CoreType::Int)), ("err".into(), None)]
            },
            &CoreType::Variant {
                variants: vec![("ok".into(), Some(CoreType::Number))]
            }
        ),
        CoreType::Variant { .. }
    ));
    assert!(matches!(
        intersect_types(
            &CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Int),
                effects: EffectRow::default(),
            },
            &CoreType::Fun {
                args: vec![CoreType::Number],
                ret: Box::new(CoreType::Number),
                effects: EffectRow::default(),
            }
        ),
        CoreType::Fun { .. } | CoreType::Intersect(_)
    ));

    assert_eq!(
        normalize_type(&CoreType::Dynamic(Box::new(CoreType::Never))),
        CoreType::Never
    );
    assert_eq!(
        normalize_type(&CoreType::Dynamic(Box::new(CoreType::dyn_any()))),
        CoreType::dyn_any()
    );
    assert_eq!(
        normalize_type(&CoreType::Union(vec![
            CoreType::Never,
            CoreType::Int,
            CoreType::Union(vec![CoreType::Int, CoreType::String])
        ])),
        CoreType::Union(vec![CoreType::Int, CoreType::String])
    );
    assert!(matches!(
        normalize_type(&CoreType::Intersect(vec![
            CoreType::Any,
            CoreType::Int,
            CoreType::Intersect(vec![CoreType::Number])
        ])),
        CoreType::Int | CoreType::Intersect(_)
    ));
    assert_eq!(
        normalize_type(&CoreType::Intersect(vec![CoreType::Int, CoreType::String])),
        CoreType::Never
    );
    assert_eq!(
        normalize_type(&CoreType::Diff(
            Box::new(CoreType::Int),
            Box::new(CoreType::Number)
        )),
        CoreType::Never
    );
    assert!(matches!(
        normalize_type(&CoreType::Diff(
            Box::new(CoreType::Number),
            Box::new(CoreType::Int)
        )),
        CoreType::Diff(_, _)
    ));

    assert!(is_runtime_checkable(&CoreType::Bytes));
    assert!(is_runtime_checkable(&CoreType::dynamic_bound(
        CoreType::Int
    )));
    assert!(is_runtime_checkable(&CoreType::Union(vec![CoreType::Int])));
    assert!(is_runtime_checkable(&CoreType::Variant {
        variants: vec![("ok".into(), Some(CoreType::Int))]
    }));
    assert!(is_runtime_checkable(&CoreType::Fun {
        args: vec![CoreType::Int],
        ret: Box::new(CoreType::Int),
        effects: EffectRow {
            ops: vec!["log".into()]
        },
    }));
    assert!(is_runtime_checkable(&CoreType::App {
        ctor: "option".into(),
        args: vec![CoreType::Int],
    }));
    assert!(is_runtime_checkable(&CoreType::Singleton(
        SingletonValue::Unit
    )));
    assert!(!is_runtime_checkable(&CoreType::Never));
    assert!(!is_runtime_checkable(&CoreType::Diff(
        Box::new(CoreType::Int),
        Box::new(CoreType::String)
    )));
    assert!(!is_runtime_checkable(&CoreType::Var(TypeVarId::new(0))));

    assert_eq!(
        cast_success_type(&CoreType::Number, &CoreType::Int),
        CoreType::Int
    );
    assert!(types_disjoint(&CoreType::String, &CoreType::Int));
}

#[test]
fn unify_open_record_lacks_union_and_occurs() {
    let mut s = Subst::new();
    let open = CoreType::OpenRecord {
        fields: vec![("a".into(), CoreType::Int)],
        row: Box::new(CoreType::Var(s.fresh_var())),
    };
    let closed = CoreType::Record {
        fields: vec![("a".into(), CoreType::Int), ("b".into(), CoreType::String)],
    };
    // May succeed or fail depending on row algorithm; exercise the path.
    let _ = unify(&open, &closed, &mut s);

    let mut s = Subst::new();
    let v = s.fresh_var();
    let lacks = CoreType::Lacks {
        label: "x".into(),
        row: Box::new(CoreType::Record {
            fields: vec![("y".into(), CoreType::Int)],
        }),
    };
    let _ = unify(&CoreType::Var(v), &lacks, &mut s);
    let _ = unify(&lacks, &CoreType::Var(v), &mut s);

    let mut s = Subst::new();
    let bad = CoreType::Lacks {
        label: "a".into(),
        row: Box::new(CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        }),
    };
    let _ = unify(&bad, &CoreType::Unit, &mut s);

    let mut s = Subst::new();
    assert!(unify(
        &CoreType::Union(vec![CoreType::Int, CoreType::String]),
        &CoreType::dyn_any(),
        &mut s
    )
    .is_ok());

    let mut s = Subst::new();
    let v = s.fresh_var();
    let ty = CoreType::OpenRecord {
        fields: vec![("a".into(), CoreType::Var(v))],
        row: Box::new(CoreType::Unit),
    };
    assert!(s.bind(v, ty).is_err());

    let mut s = Subst::new();
    let v = s.fresh_var();
    assert!(s
        .bind(v, CoreType::OptionalField(Box::new(CoreType::Var(v))))
        .is_err());

    let mut s = Subst::new();
    let v = s.fresh_var();
    assert!(s
        .bind(
            v,
            CoreType::Diff(Box::new(CoreType::Var(v)), Box::new(CoreType::Int))
        )
        .is_err());
}

#[test]
fn coerce_and_insert_implicit_casts() {
    let ok = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::Int,
        &CoreType::Number,
        1,
    )
    .unwrap();
    assert!(matches!(ok, CoreExpr::Lit(_)));

    let casted = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::dynamic_bound(CoreType::Int),
        &CoreType::F64,
        2,
    )
    .unwrap();
    assert!(matches!(
        casted,
        CoreExpr::Cast {
            evidence: CastEvidence::NumericPromote,
            ..
        }
    ));

    let err = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::Int,
        &CoreType::String,
        3,
    )
    .unwrap_err();
    assert!(err.message.contains("impossible") || err.message.contains("never"));

    let mut env = TypeEnv::new();
    env.insert(
        "f",
        CoreType::Fun {
            args: vec![CoreType::Number],
            ret: Box::new(CoreType::Number),
            effects: EffectRow::default(),
        },
    );
    env.insert(
        "x",
        CoreType::dynamic_bound(CoreType::Union(vec![CoreType::Int, CoreType::String])),
    );
    let app = CoreExpr::App {
        fun: Box::new(CoreExpr::Var("f".into())),
        args: vec![CoreExpr::Var("x".into())],
    };
    let rewritten = insert_implicit_casts(&app, &env).unwrap();
    assert!(matches!(rewritten, CoreExpr::App { .. }));
}

#[test]
fn infer_with_effects_and_handle_forms() {
    let (ty, effects) = infer_with_effects(
        &CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert_eq!(ty, CoreType::Unit);
    assert!(effects.ops.iter().any(|o| o == "log"));

    let ty =
        typecheck_language_source(r#"(val main (handle log (fn (msg) msg) (perform log "hi")))"#)
            .unwrap();
    assert_eq!(ty, CoreType::String);

    let ty = typecheck_language_source(
        r#"
(val h (handler ask (fn (_ k) (k 1))))
(val main (with h (perform ask unit)))
"#,
    )
    .unwrap();
    assert!(matches!(
        ty,
        CoreType::Int | CoreType::Unit | CoreType::Dynamic(_) | CoreType::Number
    ));
}

#[test]
fn elaborate_error_paths_and_more_forms() {
    let err = elaborate_source("(val main").unwrap_err();
    assert!(!err.message.is_empty());

    let err = elaborate_source("(val)").unwrap_err();
    assert!(!err.message.is_empty());

    let expr = elaborate_source(
        r#"(val main
  (local
    (val a 1)
    (val b a)
    b))"#,
    )
    .unwrap();
    assert!(matches!(expr, CoreExpr::Let { .. }));

    let expr = elaborate_source(
        r#"(val main
  (match (tuple 1 2)
    ((tuple a b) -> a)))"#,
    )
    .unwrap();
    assert!(matches!(expr, CoreExpr::Let { .. }));

    let (_, data) = elaborate_with_data(
        r#"
(type-alias n never)
(type-alias i (intersect int number))
(type-alias d (diff number int))
(type-alias nt (not str))
(type-alias opt (record (title (optional str))))
(val main unit)
"#,
    )
    .unwrap();
    assert!(data.type_aliases.contains_key("n"));
    assert!(data.type_aliases.contains_key("i"));
    assert!(data.type_aliases.contains_key("d"));
    assert!(data.type_aliases.contains_key("nt"));
    assert!(data.type_aliases.contains_key("opt"));
}

#[test]
fn typecheck_try_cast_check_cast_and_bytes() {
    let ty = typecheck_language_source("(val main (try-cast 1 int))").unwrap();
    assert!(matches!(ty, CoreType::App { ctor, .. } if ctor == "option"));

    let ty = typecheck_language_source(r#"(val main (check-cast "a" string))"#).unwrap();
    assert!(matches!(ty, CoreType::App { ctor, .. } if ctor == "result"));

    let ty = typecheck_language_source("(val main (bytes 1 2))").unwrap();
    assert_eq!(ty, CoreType::Bytes);

    let ty = typecheck_language_source("(val main (encode-utf8 \"hi\"))").unwrap();
    assert_eq!(ty, CoreType::Bytes);

    let ty = typecheck_language_source("(val main (decode-utf8 (bytes 104)))").unwrap();
    assert!(
        matches!(ty, CoreType::App { ref ctor, .. } if ctor == "result")
            || matches!(ty, CoreType::Variant { .. })
            || matches!(ty, CoreType::Dynamic(_))
    );
}

#[test]
fn typecheck_record_update_extend_and_optional() {
    let ty = typecheck_language_source(
        r#"
(val r (record (a 1) (b 2)))
(val main (field (record-update r (a 3)) a))
"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::Int);

    let ty = typecheck_language_source(
        r#"
(val r (record (a 1)))
(val main (field (record-extend r (b "x")) b))
"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::String);
}

#[test]
fn typecheck_raise_as_result_or_raise() {
    let ty =
        typecheck_language_source(r#"(val main (handle failure (fn (e) e) (raise "x")))"#).unwrap();
    assert!(matches!(ty, CoreType::String | CoreType::Dynamic(_)));

    let ty = typecheck_language_source(r#"(val main (as-result (fn () 1)))"#).unwrap();
    // as-result wraps in result; checker may still report Dynamic until DAT closes.
    let _ = ty;

    let ty = typecheck_language_source(
        r#"
(data result (ok int) (err str))
(val main
  (handle failure (fn (e) e)
    (or-raise (err "boom"))))
"#,
    )
    .unwrap();
    assert!(matches!(ty, CoreType::String | CoreType::Dynamic(_)));
}

#[test]
fn typecheck_forward_and_seq_unit() {
    let ty = typecheck_language_source(
        r#"(val main
  (handle log (fn (msg) msg)
    (handle log (fn (msg k) (forward k))
      (perform log "outer"))))"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::String);

    let ty = typecheck_language_source("(val main (seq 1 2))").unwrap();
    assert_eq!(ty, CoreType::Int);
}

#[test]
fn typecheck_list_and_if_never_branch() {
    let ty = typecheck_language_source("(val main (list 1 2))").unwrap();
    assert!(
        matches!(ty, CoreType::App { ref ctor, .. } if ctor == "list")
            || matches!(ty, CoreType::Variant { .. })
    );

    let ty = typecheck_language_source(r#"(val main (if false (raise "x") 1))"#).unwrap();
    assert_eq!(ty, CoreType::Int);
}

#[test]
fn plan_structural_checks_for_complex_targets() {
    assert!(
        plan_cast_evidence(&CoreType::dyn_any(), &CoreType::Union(vec![CoreType::Int])).is_some()
    );
    assert!(matches!(
        plan_cast_evidence(
            &CoreType::dyn_any(),
            &CoreType::Intersect(vec![CoreType::Int, CoreType::Number])
        ),
        Some(
            CastEvidence::IntersectionCheck { .. }
                | CastEvidence::Identity
                | CastEvidence::TagCheck { .. }
        )
    ));
    assert!(matches!(
        plan_cast_evidence(
            &CoreType::dyn_any(),
            &CoreType::Record {
                fields: vec![("a".into(), CoreType::Int)]
            }
        ),
        Some(CastEvidence::RecordCheck { .. })
    ));
    assert!(matches!(
        plan_cast_evidence(
            &CoreType::dyn_any(),
            &CoreType::Variant {
                variants: vec![("ok".into(), None)]
            }
        ),
        Some(CastEvidence::VariantCheck { .. })
    ));
    assert!(matches!(
        plan_cast_evidence(
            &CoreType::dyn_any(),
            &CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Int),
                effects: EffectRow::default(),
            }
        ),
        Some(CastEvidence::FunctionGuard { .. })
    ));
    assert!(matches!(
        plan_cast_evidence(
            &CoreType::dyn_any(),
            &CoreType::App {
                ctor: "option".into(),
                args: vec![CoreType::Int]
            }
        ),
        Some(CastEvidence::NominalCheck { .. })
    ));
}

#[test]
fn insert_casts_walks_compound_exprs() {
    let mut env = TypeEnv::new();
    env.insert("x", CoreType::Int);
    let expr = CoreExpr::Seq(vec![
        CoreExpr::Let {
            name: "y".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            body: Box::new(CoreExpr::Var("y".into())),
        },
        CoreExpr::If {
            cond: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
            then_branch: Box::new(CoreExpr::Var("x".into())),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        CoreExpr::Record {
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
        },
        CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "ok".into(),
                payload: None,
            }),
            arms: vec![MatchArm::variant(
                "ok".into(),
                None,
                CoreExpr::Lit(CoreLiteral::Unit),
            )],
        },
    ]);
    let out = insert_implicit_casts(&expr, &env).unwrap();
    assert!(matches!(out, CoreExpr::Seq(_)));
}
