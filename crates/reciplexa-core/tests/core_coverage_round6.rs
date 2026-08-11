//! Round-6 core: expansive generalization, local escapes, cast/check residuals.

use reciplexa_core::cast::{
    decide_subtype, is_runtime_checkable, is_subtype, judge_dynamic_use, plan_cast_evidence,
};
use reciplexa_core::check::{
    coerce_to_static, infer_expr, infer_with_effects, insert_implicit_casts,
    typecheck_language_source, TypeEnv,
};
use reciplexa_core::elaborate::elaborate_source;
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::{CoreType, EffectRow};
use reciplexa_core::unify::{unify, Subst};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn expansive_under_annotation_and_is_expansive_forms() {
    // Force value-restriction / expansive checks across forms.
    for src in [
        r#"
(type x (forall ((a type)) a))
(val x (perform log "e"))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x ((fn (y) y) 1))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (handle log (fn (m) m) (perform log "e")))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (with (handler log (fn (m) m)) (perform log "e")))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (seq (perform log "e") 1))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (let ((y (perform log "e"))) y))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (var y 0 (seq (set y 1) y)))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (if true (perform log "e") 1))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (record (a (perform log "e"))))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (record-update (record (a 1)) (a (perform log "e"))))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (field (record (a (perform log "e"))) a))
(val main x)
"#,
        r#"
(data opt (none) (some v))
(type x (forall ((a type)) a))
(val x (match (some (perform log "e")) (none -> 0) (some v -> v)))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (some (perform log "e")))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (as int (perform log "e")))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (try-cast (perform log "e") int))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (check-cast (perform log "e") int))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (forward k))
(val main x)
"#,
        r#"
(type x (forall ((a type)) a))
(val x (set y 1))
(val main x)
"#,
    ] {
        let _ = typecheck_language_source(src);
        let _ = elaborate_source(src);
    }
}

#[test]
fn local_escape_through_fun_record_variant_union() {
    // Escape: return a lambda that closes over local var → effect mention.
    let err = typecheck_language_source(
        r#"(val main
  (var s 0
    (fn () s)))"#,
    );
    let _ = err;

    let err = typecheck_language_source(
        r#"(val main
  (var s 0
    (record (f (fn () s)))))"#,
    );
    let _ = err;

    let ty = typecheck_language_source(
        r#"(val main
  (var s 0
    (seq (set s 1) (handle log (fn (m) m) (perform log "x")) s)))"#,
    );
    let _ = ty;
}

#[test]
fn cast_judge_and_coerce_matrix() {
    let _ = judge_dynamic_use(&CoreType::Int, &CoreType::String);
    let _ = judge_dynamic_use(&CoreType::Int, &CoreType::Int);
    let _ = judge_dynamic_use(&CoreType::Any, &CoreType::Int);
    let _ = judge_dynamic_use(&CoreType::Never, &CoreType::Int);

    let err = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::Dynamic(Box::new(CoreType::Int)),
        &CoreType::String,
        9,
    );
    assert!(err.is_err());

    let ok = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::Dynamic(Box::new(CoreType::Int)),
        &CoreType::Int,
        10,
    );
    assert!(ok.is_ok());

    let ok = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::Int,
        &CoreType::Number,
        11,
    );
    assert!(ok.is_ok());

    // Partial overlap inserts cast
    let _ = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::dyn_any(),
        &CoreType::Int,
        12,
    );

    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Lambda {
            params: vec!["x".into()],
            body: Box::new(CoreExpr::Var("x".into())),
        }),
        args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
    };
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());

    let expr = CoreExpr::Let {
        name: "x".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        body: Box::new(CoreExpr::Var("x".into())),
    };
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());

    let expr = CoreExpr::If {
        cond: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
        then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
    };
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());

    let expr = CoreExpr::Seq(vec![
        CoreExpr::Lit(CoreLiteral::Int(1)),
        CoreExpr::Lit(CoreLiteral::Int(2)),
    ]);
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());

    let expr = CoreExpr::TryCast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        target: CoreType::Int,
        cast_id: 1,
    };
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());

    let expr = CoreExpr::CheckCast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        target: CoreType::Int,
        cast_id: 2,
    };
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());
}

#[test]
fn typecheck_and_infer_residual_forms() {
    let (ty, effs) = infer_with_effects(
        &CoreExpr::HandlerValue {
            op: "ask".into(),
            handler_params: vec!["x".into(), "k".into()],
            handler_body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Var("x".into())],
            }),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    let _ = (ty, effs);

    let mut env = TypeEnv::new();
    env.insert(
        "r",
        CoreType::Fun {
            args: vec![CoreType::String],
            ret: Box::new(CoreType::Unit),
            effects: EffectRow::default().with_op("log"),
        },
    );
    let ty = infer_expr(
        &CoreExpr::Forward {
            resume_name: "r".into(),
        },
        &env,
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert_eq!(ty, CoreType::Unit);

    for src in [
        "(val main (as (optional int) 1))",
        "(val main (as (lacks a (record (b int))) (record (b 1))))",
        "(data tree ((a type)) (leaf) (node (tree a) (tree a)))\n(val main leaf)",
        "(val main (match 1.5 (1 -> 0) (_ -> 1)))",
        "(val main (match 1 (() -> 0) (_ -> 1)))",
        "(val main (local (type orphan int) 1))",
        "(val main (local (var if 1) if))",
        "(val main (local (val if 1) if))",
        "(data t (c a b c))\n(val main (c 1 2 3))",
        "(data t (c a b))\n(val main (match (c 1 2) ((c x y) -> x) (_ -> 0)))",
    ] {
        let _ = typecheck_language_source(src);
        let _ = elaborate_source(src);
    }
}

#[test]
fn cast_decide_matrix_extended() {
    for (a, b) in [
        (CoreType::Color, CoreType::Shape),
        (CoreType::Bytes, CoreType::String),
        (CoreType::Unit, CoreType::Bool),
        (
            CoreType::Intersect(vec![CoreType::Int, CoreType::String]),
            CoreType::Int,
        ),
        (
            CoreType::Union(vec![CoreType::Int, CoreType::Never]),
            CoreType::Int,
        ),
        (
            CoreType::Not(Box::new(CoreType::Int)),
            CoreType::String,
        ),
        (
            CoreType::Diff(Box::new(CoreType::Number), Box::new(CoreType::Int)),
            CoreType::F64,
        ),
        (
            CoreType::OptionalField(Box::new(CoreType::Int)),
            CoreType::dyn_any(),
        ),
        (
            CoreType::Lacks {
                label: "a".into(),
                row: Box::new(CoreType::Record { fields: vec![] }),
            },
            CoreType::Record { fields: vec![] },
        ),
        (
            CoreType::Forall {
                params: vec![("a".into(), "type".into())],
                body: Box::new(CoreType::Name("a".into())),
            },
            CoreType::Int,
        ),
    ] {
        let _ = decide_subtype(&a, &b);
        let _ = is_subtype(&a, &b);
        let _ = is_runtime_checkable(&a);
        let _ = plan_cast_evidence(&a, &b);
    }

    let mut s = Subst::new();
    let _ = unify(
        &CoreType::Color,
        &CoreType::Color,
        &mut s,
    );
    let arms = vec![
        MatchArm {
            pattern: CorePattern::Wildcard,
            body: CoreExpr::Lit(CoreLiteral::Int(0)),
        },
        MatchArm {
            pattern: CorePattern::Bind("x".into()),
            body: CoreExpr::Lit(CoreLiteral::Int(1)),
        },
    ];
    let _ = reciplexa_core::expr::first_unreachable_arm(&arms, &[]);
}
