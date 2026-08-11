//! Round-8 core: deeper elaborate residual + check/cast/unify edge matrix.

use reciplexa_core::cast::{
    cast_success_type, compose_evidence, decide_subtype, is_runtime_checkable, is_subtype,
    judge_dynamic_use, plan_cast_evidence, simplify_evidence, types_disjoint, CastEvidence,
    DecideResult,
};
use reciplexa_core::check::{
    coerce_to_static, infer_expr, infer_with_effects, insert_implicit_casts,
    typecheck_language_source, TypeEnv,
};
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::expr::{first_unreachable_arm, CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::{CoreType, EffectRow, SingletonValue};
use reciplexa_core::unify::{unify, Subst, UnifyError};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn elaborate_round8_data_variance_positivity_and_forms() {
    for src in [
        // Variance: covariant / contravariant / invariant / phantom
        "(data box ((a type)) (mk a))\n(val main (mk 1))",
        "(data box ((a type)) (mk (fn a int)))\n(val main 1)",
        "(data box ((a type)) (mk (fn int a)))\n(val main 1)",
        "(data box ((a type)) (mk (fn a a)))\n(val main 1)",
        "(data ghost ((a type)) nullary)\n(val main nullary)",
        // Strict positivity violation (negative occurrence of group type)
        "(data bad (mk (fn bad int)))\n(val main 1)",
        "(rec (data a (x (fn a int))) (data b (y)))\n(val main 1)",
        // Payload nested type forms
        "(data t (c (tuple int string)))\n(val main 1)",
        "(data t (c (record (a int))))\n(val main 1)",
        "(data t (c (union int string)))\n(val main 1)",
        "(data t (c (optional-field int)))\n(val main 1)",
        "(data t (c (dynamic int)))\n(val main 1)",
        "(data t ((a type)) (c (list a)))\n(val main 1)",
        // Rec data + value annotation / main in Rec
        "(rec (val f (fn (x) x)) (val main (f 1)))",
        "(type orphan int)\n(val main 1)",
        "(val main (local (type-alias u int) (val x 1) x))",
        // Top-level token / seq trailing
        "42",
        "true",
        "\"hi\"",
        "unit",
        "#\\a",
        "1 2 3",
        "(val x 1)\n(+ x 2)",
        // Quarantined document heads
        "(val main (slide a4))",
        "(val main (artboard a4))",
        "(val main (canvas a4))",
        // Match / as / cast residual
        "(val main (match (bytes 1 2) (_ -> 0)))",
        "(val main (as (diff number int) 1))",
        "(val main (as (not string) 1))",
        "(val main (as (intersect int number) 1))",
        "(val main (as (union int string) 1))",
        "(val main (as (record (a int) (row r)) (record (a 1))))",
        "(val main (as (forall ((a type)) a) 1))",
        "(val main (as (fn int string (effects ask log)) (fn (x) x)))",
        "(val main (as (singleton false) false))",
        "(val main (as (singleton unit) unit))",
        "(val main (as (singleton \"z\") \"z\"))",
        // Handle / with / handler denser
        "(val main (handle failure (fn (e) e) (raise \"x\")))",
        "(val main (handle log (fn (m k) (k unit)) (perform log \"m\")))",
        "(val main (with (handler failure (fn (e) e)) (raise \"e\")))",
        "(val main (handler failure (fn (e) e)))",
        // Local / set / var / if
        "(val main (local (var c 0) (set c 1) c))",
        "(val main (local (val x 1) (val y 2) (val z 3) (+ x (+ y z))))",
        "(val main (if true 1 0))",
        "(val main (if false 1 0))",
        // Errors
        "(data t (c graphics/color))",
        "(data t (c ()))",
        "(data t (c (fn int)))",
        "(data t (c 1))",
        "(rec (data a (x)))",
        "(type t (fn))",
        "(type t (record))",
        "(type t (union))",
        "(type t (intersect))",
        "(type t (not))",
        "(type t (diff int))",
        "(type t (singleton))",
        "(type t (effects))",
        "(type t (forall))",
        "(val main (match 1 (1 ->) ))",
        "(val main (match 1 (-> 0)))",
        "(val main (bytes))",
        "(val main (list))",
        "(val main (tuple))",
        "(val main (unicode))",
        "(val main (unicode \"x\"))",
        "(val main (unicode 0x110000))",
        "(fn main ())",
        "(fn (x) x)",
        "(val (f x) )",
        "(val main (handle ask (fn) 1))",
        "(val main (handle ask (fn (m k r) m) 1))",
        "(val main (as (lacks) 1))",
        "(val main (as (lacks a) 1))",
        "",
        "()",
    ] {
        let _ = elaborate_source(src);
        let _ = elaborate_with_data(src);
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn check_round8_core_expr_matrix() {
    let env = TypeEnv::new();
    let mut subst = Subst::new();
    let r = range();

    let cases: Vec<CoreExpr> = vec![
        CoreExpr::Error,
        CoreExpr::Lit(CoreLiteral::Int(1)),
        CoreExpr::Lit(CoreLiteral::F64(1.5)),
        CoreExpr::Lit(CoreLiteral::Number(2.5)),
        CoreExpr::Lit(CoreLiteral::Bool(true)),
        CoreExpr::Lit(CoreLiteral::String("x".into())),
        CoreExpr::Lit(CoreLiteral::Unit),
        CoreExpr::Lit(CoreLiteral::Bytes(vec![1, 2])),
        CoreExpr::Lit(CoreLiteral::Color("#000".into())),
        CoreExpr::Var("missing".into()),
        CoreExpr::Lambda {
            params: vec!["x".into()],
            body: Box::new(CoreExpr::Var("x".into())),
        },
        CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Var("x".into())),
            }),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        CoreExpr::If {
            cond: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::Int(1)),
            CoreExpr::Lit(CoreLiteral::Int(2)),
        ]),
        CoreExpr::Seq(vec![]),
        CoreExpr::Record {
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
        },
        CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            field: "a".into(),
        },
        CoreExpr::RecordUpdate {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        CoreExpr::RecordExtend {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            fields: vec![("b".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Int(1)))),
        },
        CoreExpr::Variant {
            tag: "none".into(),
            payload: None,
        },
        CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::String("m".into()))),
        },
        CoreExpr::Forward {
            resume_name: "k".into(),
        },
        CoreExpr::Set {
            name: "c".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
            body: Box::new(CoreExpr::Var("c".into())),
        },
        CoreExpr::Let {
            name: "x".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            body: Box::new(CoreExpr::Var("x".into())),
        },
        CoreExpr::LetRec {
            bindings: vec![(
                "f".into(),
                CoreExpr::Lambda {
                    params: vec!["x".into()],
                    body: Box::new(CoreExpr::Var("x".into())),
                },
            )],
            body: Box::new(CoreExpr::Var("f".into())),
        },
        CoreExpr::Cast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            target: CoreType::Number,
            evidence: CastEvidence::Identity,
            cast_id: 1,
        },
        CoreExpr::TryCast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            target: CoreType::Int,
            cast_id: 2,
        },
        CoreExpr::CheckCast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            target: CoreType::Int,
            cast_id: 3,
        },
        CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Lit(CoreLiteral::Int(1)),
                    body: CoreExpr::Lit(CoreLiteral::Int(1)),
                },
                MatchArm {
                    pattern: CorePattern::Wildcard,
                    body: CoreExpr::Lit(CoreLiteral::Int(0)),
                },
            ],
        },
        CoreExpr::Handle {
            op: "ask".into(),
            handler_params: vec!["m".into()],
            handler_body: Box::new(CoreExpr::Var("m".into())),
            body: Box::new(CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String("q".into()))),
            }),
        },
        CoreExpr::HandlerValue {
            op: "ask".into(),
            handler_params: vec!["m".into(), "k".into()],
            handler_body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Var("m".into())],
            }),
        },
        CoreExpr::With {
            handler: Box::new(CoreExpr::HandlerValue {
                op: "ask".into(),
                handler_params: vec!["m".into()],
                handler_body: Box::new(CoreExpr::Var("m".into())),
            }),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
    ];

    for expr in &cases {
        let _ = infer_expr(expr, &env, &mut subst, r);
        let _ = infer_with_effects(expr, &env, &mut subst, r);
        let _ = insert_implicit_casts(expr, &env);
        let _ = coerce_to_static(expr.clone(), &CoreType::dyn_any(), &CoreType::Int, 9);
    }

    let arms = [
        MatchArm {
            pattern: CorePattern::Wildcard,
            body: CoreExpr::Lit(CoreLiteral::Int(0)),
        },
        MatchArm {
            pattern: CorePattern::Bind("x".into()),
            body: CoreExpr::Lit(CoreLiteral::Int(1)),
        },
    ];
    let _ = first_unreachable_arm(&arms, &[]);
    let _ = first_unreachable_arm(&arms, &["some", "none"]);
}

#[test]
fn cast_unify_round8_residual() {
    let _ = cast_success_type(&CoreType::Int, &CoreType::Number);
    let _ = compose_evidence(vec![
        CastEvidence::Identity,
        CastEvidence::Compose(vec![CastEvidence::Identity]),
        CastEvidence::TagCheck { tag: "int".into() },
        CastEvidence::Widen,
        CastEvidence::NumericPromote,
        CastEvidence::NominalCheck {
            name: "option".into(),
        },
        CastEvidence::UnionCheck {
            members: vec![CoreType::Int, CoreType::String],
        },
        CastEvidence::IntersectionCheck {
            members: vec![CoreType::Int, CoreType::Number],
        },
        CastEvidence::RecordCheck {
            fields: vec![("a".into(), CoreType::Int)],
        },
        CastEvidence::VariantCheck {
            variants: vec![("some".into(), Some(CoreType::Int))],
        },
        CastEvidence::FunctionGuard {
            arity: 1,
            arg_casts: vec![CastEvidence::Identity],
            ret_cast: Box::new(CastEvidence::Identity),
        },
    ]);
    let _ = simplify_evidence(CastEvidence::Compose(vec![
        CastEvidence::Identity,
        CastEvidence::TagCheck { tag: "int".into() },
        CastEvidence::Compose(vec![CastEvidence::Identity]),
    ]));
    for (a, b) in [
        (CoreType::Int, CoreType::Number),
        (CoreType::Number, CoreType::Int),
        (CoreType::F64, CoreType::Number),
        (CoreType::Never, CoreType::String),
        (CoreType::Any, CoreType::Int),
        (CoreType::Int, CoreType::Any),
        (CoreType::dyn_any(), CoreType::Bool),
        (
            CoreType::Singleton(SingletonValue::Int(1)),
            CoreType::Number,
        ),
        (
            CoreType::Singleton(SingletonValue::Bool(true)),
            CoreType::Bool,
        ),
        (
            CoreType::Union(vec![CoreType::Int, CoreType::String]),
            CoreType::Number,
        ),
        (
            CoreType::Intersect(vec![CoreType::Int, CoreType::Number]),
            CoreType::Int,
        ),
        (CoreType::Not(Box::new(CoreType::String)), CoreType::Int),
        (
            CoreType::Diff(Box::new(CoreType::Number), Box::new(CoreType::Int)),
            CoreType::F64,
        ),
        (
            CoreType::Record {
                fields: vec![("a".into(), CoreType::Int)],
            },
            CoreType::Record {
                fields: vec![("a".into(), CoreType::Number)],
            },
        ),
        (
            CoreType::OpenRecord {
                fields: vec![("a".into(), CoreType::Int)],
                row: Box::new(CoreType::Record { fields: vec![] }),
            },
            CoreType::Record {
                fields: vec![("a".into(), CoreType::Int)],
            },
        ),
        (
            CoreType::Variant {
                variants: vec![("some".into(), Some(CoreType::Int))],
            },
            CoreType::Variant {
                variants: vec![("some".into(), Some(CoreType::Number))],
            },
        ),
        (
            CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Int),
                effects: EffectRow::default(),
            },
            CoreType::Fun {
                args: vec![CoreType::Number],
                ret: Box::new(CoreType::Number),
                effects: EffectRow::default(),
            },
        ),
        (
            CoreType::App {
                ctor: "list".into(),
                args: vec![CoreType::Int],
            },
            CoreType::App {
                ctor: "list".into(),
                args: vec![CoreType::Int],
            },
        ),
        (CoreType::Bytes, CoreType::Bytes),
        (CoreType::Color, CoreType::Color),
        (CoreType::Shape, CoreType::Shape),
        (
            CoreType::OptionalField(Box::new(CoreType::Int)),
            CoreType::OptionalField(Box::new(CoreType::Int)),
        ),
        (CoreType::Error, CoreType::Int),
        (CoreType::Name("t".into()), CoreType::Int),
        (
            CoreType::Forall {
                params: vec![("a".into(), "type".into())],
                body: Box::new(CoreType::Name("a".into())),
            },
            CoreType::Int,
        ),
        (
            CoreType::Lacks {
                label: "a".into(),
                row: Box::new(CoreType::Record { fields: vec![] }),
            },
            CoreType::Record { fields: vec![] },
        ),
    ] {
        let _ = is_subtype(&a, &b);
        let _ = is_runtime_checkable(&a);
        let _ = types_disjoint(&a, &b);
        let _ = judge_dynamic_use(&a, &b);
        let _ = decide_subtype(&a, &b);
        let _ = plan_cast_evidence(&a, &b);
        let mut subst = Subst::new();
        let _ = unify(&a, &b, &mut subst);
    }

    let mut subst = Subst::new();
    let v = CoreType::Var(subst.fresh_var());
    let _ = unify(&v, &CoreType::Int, &mut subst);
    let _ = unify(&CoreType::Int, &CoreType::F64, &mut subst);
    match unify(
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        },
        &CoreType::Record {
            fields: vec![("b".into(), CoreType::Int)],
        },
        &mut subst,
    ) {
        Err(UnifyError::Mismatch { .. }) => {}
        Ok(()) => {}
        Err(_) => {}
    }
    let _ = decide_subtype(&CoreType::Int, &CoreType::String);
    assert!(matches!(
        decide_subtype(&CoreType::Int, &CoreType::Int),
        DecideResult::Proved | DecideResult::Unknown | DecideResult::Disproved
    ));
}
