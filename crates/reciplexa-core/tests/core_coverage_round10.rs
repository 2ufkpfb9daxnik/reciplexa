//! Round-10 core: residual Err matrices for elaborate/check + denser CoreExpr edges.

use reciplexa_core::cast::{
    cast_success_type, compose_evidence, decide_subtype, intersect_types, is_runtime_checkable,
    is_subtype, judge_dynamic_use, normalize_type, plan_cast_evidence, simplify_evidence,
    types_disjoint, CastEvidence, DecideResult,
};
use reciplexa_core::check::{
    coerce_to_static, infer_expr, infer_with_effects, insert_implicit_casts,
    typecheck_language_source, TypeEnv,
};
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data, DataEnv};
use reciplexa_core::expr::{first_unreachable_arm, CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::{CoreType, EffectRow, SingletonValue};
use reciplexa_core::unify::{unify, Subst};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn elaborate_round10_effect_pattern_and_form_residuals() {
    for src in [
        // Effect-row and forall residuals
        "(type f (fn unit unit (effects (io string) log ask)))\n(val f (fn () unit))\n(val main (f))",
        "(type id (forall ((a type)(b type)) (fn a b)))\n(val id (fn (x) x))\n(val main 1)",
        "(type bad (forall (a type) a))\n(val main 1)",
        "(type bad (forall ((a)) a))\n(val main 1)",
        "(type bad (forall ((a kind)) a))\n(val main 1)",
        "(type bad (forall ((1 type)) int))\n(val main 1)",
        "(type bad (forall ((a type)(a type)) a))\n(val main 1)",
        // Rec / main / trailing selection
        "(rec (val a (fn () 1)) (val b (fn () 2)) (val main (a)))",
        "(rec (val z (fn () 0)))",
        "(val a 1)\n(val b 2)",
        // Pattern denser Err
        "(val main (match (tuple 1 2) ((tuple a) -> a) (_ -> 0)))",
        "(val main (match (record (a 1)) ((record) -> 0) (_ -> 1)))",
        "(val main (match (record (a 1)) ((record (a)) -> 0) (_ -> 1)))",
        "(val main (match (record (a 1)) ((record (1 x)) -> 0) (_ -> 1)))",
        "(data opt (none) (some x))\n(val main (match (some 1) ((some) -> 0) (none -> 1)))",
        "(data opt (none) (some x))\n(val main (match (some 1) ((some x y) -> x) (none -> 0)))",
        // Local / var / set denser
        "(val main (local (var c 0) (var d 1) (set c (+ c d)) c))",
        "(val main (local (type u int) (type-alias v int) (val u 1) u))",
        "(val main (local (rec (val f (fn (x) x)) (val g (fn (y) y))) (f 1)))",
        "(val main (local (data t (c)) (c)))",
        // Quarantined heads nested
        "(val main (seq (page a4) 1))",
        "(val main (let ((x (circle 1))) x))",
        // Bytes / unicode / special chars
        "(val main (bytes 0 1 2 255))",
        "(val main #\\newline)",
        "(val main #\\space)",
        "(val main #\\tab)",
        "(val main #\\x41)",
        // Perform ambient variants
        "(val main (perform log))",
        "(val main (perform log \"a\" \"b\"))",
        "(val main (raise))",
        "(val main (or-raise))",
        "(val main (or-raise true))",
        "(val main (as-result))",
        "(val main (try-cast))",
        "(val main (check-cast 1))",
        // Record / field residuals
        "(val main (record))",
        "(val main (record (a)))",
        "(val main (record (1 2)))",
        "(val main (record-update (record (a 1))))",
        "(val main (record-extend (record (a 1))))",
        "(val main (field))",
        "(val main (field (record (a 1))))",
        // Empty / comment only
        "(; comment ;)\n(; another ;)",
        "main",
        "foo/bar/baz",
    ] {
        let _ = elaborate_source(src);
        let _ = elaborate_with_data(src);
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn check_round10_core_expr_err_and_ok_matrix() {
    let r = range();
    let mut env = TypeEnv::new();
    env.insert("x", CoreType::Int);
    env.insert(
        "k",
        CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::Unit),
            effects: EffectRow::default(),
        },
    );
    env.local_state.insert("c".into());
    env.insert("c", CoreType::Int);
    // Annotated letrec stub
    env.data.type_aliases.insert(
        "f".into(),
        CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        },
    );
    // ADT for parameterized ctor inference
    env.data.ctor_type.insert("mk".into(), "box".into());
    env.data
        .type_params
        .insert("box".into(), vec!["a".into()]);
    env.data
        .ctor_payloads
        .insert("mk".into(), vec![CoreType::Name("a".into())]);
    env.data
        .data_ctors
        .insert("box".into(), vec![("mk".into(), 1)]);

    let mut subst = Subst::new();
    let cases: Vec<CoreExpr> = vec![
        // Forward with valid resume
        CoreExpr::Forward {
            resume_name: "k".into(),
        },
        CoreExpr::Forward {
            resume_name: "x".into(), // Int — not Fun → Err
        },
        // Handle failure 1-param / 2-param Err / Ask 2-param
        CoreExpr::Handle {
            op: "failure".into(),
            handler_params: vec!["e".into()],
            handler_body: Box::new(CoreExpr::Var("e".into())),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        CoreExpr::Handle {
            op: "failure".into(),
            handler_params: vec!["e".into(), "k".into()],
            handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
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
        CoreExpr::Handle {
            op: "ask".into(),
            handler_params: vec!["m".into(), "resume".into()],
            handler_body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("resume".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
            }),
            body: Box::new(CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String("q".into()))),
            }),
        },
        CoreExpr::HandlerValue {
            op: "ask".into(),
            handler_params: vec!["m".into()],
            handler_body: Box::new(CoreExpr::Var("m".into())),
        },
        CoreExpr::HandlerValue {
            op: "ask".into(),
            handler_params: vec!["m".into(), "resume".into()],
            handler_body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("resume".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
            }),
        },
        // Let binding = body of same name
        CoreExpr::Let {
            name: "x".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            body: Box::new(CoreExpr::Var("x".into())),
        },
        CoreExpr::Let {
            name: "y".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Var("y".into())],
            }),
        },
        // LetRec with type alias stub + arity match
        CoreExpr::LetRec {
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
        // Wildcard lambda params
        CoreExpr::Lambda {
            params: vec!["_".into(), "a".into()],
            body: Box::new(CoreExpr::Var("a".into())),
        },
        // LocalVar escape path — fun that returns local-state effect in type is hard;
        // still exercise set on local state.
        CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
            body: Box::new(CoreExpr::Set {
                name: "c".into(),
                value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            }),
        },
        CoreExpr::Set {
            name: "c".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(2))),
        },
        // RecordGet OpenRecord present / absent
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
        CoreExpr::RecordUpdate {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            fields: vec![("missing".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        CoreExpr::RecordExtend {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            fields: vec![("b".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        CoreExpr::RecordExtend {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        // Variant ctor with schema
        CoreExpr::Variant {
            tag: "mk".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Int(1)))),
        },
        CoreExpr::Variant {
            tag: "mk".into(),
            payload: None,
        },
        // Match with arms
        CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Lit(CoreLiteral::Int(1)),
                    body: CoreExpr::Lit(CoreLiteral::Int(10)),
                },
                MatchArm {
                    pattern: CorePattern::Wildcard,
                    body: CoreExpr::Lit(CoreLiteral::Int(0)),
                },
            ],
        },
        CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "mk".into(),
                payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Int(1)))),
            }),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Variant {
                        tag: "mk".into(),
                        payload: Some(Box::new(CorePattern::Bind("v".into()))),
                    },
                    body: CoreExpr::Var("v".into()),
                },
                MatchArm {
                    pattern: CorePattern::Wildcard,
                    body: CoreExpr::Lit(CoreLiteral::Int(0)),
                },
            ],
        },
        // If with occurrence
        CoreExpr::If {
            cond: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("number?".into())),
                args: vec![CoreExpr::Var("x".into())],
            }),
            then_branch: Box::new(CoreExpr::Var("x".into())),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        // Numeric builtins
        CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![
                CoreExpr::Lit(CoreLiteral::Int(1)),
                CoreExpr::Lit(CoreLiteral::Int(2)),
            ],
        },
        CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![
                CoreExpr::Lit(CoreLiteral::String("a".into())),
                CoreExpr::Lit(CoreLiteral::Int(2)),
            ],
        },
        CoreExpr::App {
            fun: Box::new(CoreExpr::Var("<".into())),
            args: vec![
                CoreExpr::Lit(CoreLiteral::F64(1.0)),
                CoreExpr::Lit(CoreLiteral::F64(2.0)),
            ],
        },
        CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::Int(1)),
            CoreExpr::Lit(CoreLiteral::Int(2)),
        ]),
        CoreExpr::With {
            handler: Box::new(CoreExpr::HandlerValue {
                op: "ask".into(),
                handler_params: vec!["m".into()],
                handler_body: Box::new(CoreExpr::Var("m".into())),
            }),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        // Cast forms
        CoreExpr::Cast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            evidence: CastEvidence::Identity,
            target: CoreType::Number,
            cast_id: 1,
        },
        CoreExpr::TryCast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            target: CoreType::String,
            cast_id: 2,
        },
        CoreExpr::CheckCast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            target: CoreType::Int,
            cast_id: 3,
        },
    ];

    for expr in cases {
        let _ = infer_with_effects(&expr, &env, &mut subst, r);
        let _ = infer_expr(&expr, &env, &mut subst, r);
        let _ = insert_implicit_casts(&expr, &env);
        let _ = coerce_to_static(expr.clone(), &CoreType::Int, &CoreType::dyn_any(), 0);
    }

    for src in [
        "(val main (local (var c 0) (set c 1) c))",
        "(type f (forall ((a type)) (fn a a)))\n(val f (fn (x) x))\n(val main (f 1))",
        "(type f (forall ((a type)) (fn a a)))\n(val f ((fn (x) x) 1))\n(val main f)",
        "(data box ((a type)) (mk a))\n(val main (mk 1))",
        "(data pair ((a type)(b type)) (mk a b))\n(val main (mk 1 \"x\"))",
        "(val main (if (number? 1) 1 0))",
        "(val main (record-update (record (a 1)) (a \"x\")))",
        "(val main (field (record (a 1) (b 2)) b))",
        "(val main (+ 1 2))",
        "(val main (+ 1.0 2.0))",
        "(val main (< 1 2))",
        "(val main (handle failure (fn (e) e) (raise \"x\")))",
        "(val main (match (variant some 1) ((some x) -> x) ((none) -> 0)))",
    ] {
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn cast_unify_round10_residuals() {
    let mut subst = Subst::new();
    for (a, b) in [
        (
            CoreType::Lacks {
                label: "a".into(),
                row: Box::new(CoreType::Unit),
            },
            CoreType::Record {
                fields: vec![("b".into(), CoreType::Int)],
            },
        ),
        (
            CoreType::OpenRecord {
                fields: vec![("a".into(), CoreType::Int)],
                row: Box::new(CoreType::Var(subst.fresh_var())),
            },
            CoreType::OpenRecord {
                fields: vec![("a".into(), CoreType::Int), ("b".into(), CoreType::Bool)],
                row: Box::new(CoreType::Var(subst.fresh_var())),
            },
        ),
        (
            CoreType::Singleton(SingletonValue::Unit),
            CoreType::Unit,
        ),
        (
            CoreType::Singleton(SingletonValue::String("z".into())),
            CoreType::String,
        ),
        (CoreType::Color, CoreType::Shape),
        (
            CoreType::Diff(Box::new(CoreType::Any), Box::new(CoreType::Never)),
            CoreType::Any,
        ),
    ] {
        let _ = unify(&a, &b, &mut subst);
        let _ = is_subtype(&a, &b);
        let _ = decide_subtype(&a, &b);
        let _ = types_disjoint(&a, &b);
        let _ = plan_cast_evidence(&a, &b);
        let _ = judge_dynamic_use(&a, &b);
        let _ = intersect_types(&a, &b);
        let _ = normalize_type(&a);
    }
    let _ = cast_success_type(&CoreType::Number, &CoreType::Int);
    let _ = is_runtime_checkable(&CoreType::Never);
    let ev = compose_evidence(vec![
        CastEvidence::Widen,
        CastEvidence::TagCheck { tag: "int".into() },
        CastEvidence::Identity,
    ]);
    let _ = simplify_evidence(ev);
    assert!(!matches!(
        decide_subtype(&CoreType::String, &CoreType::Int),
        DecideResult::Proved
    ));
    let arms = [
        MatchArm {
            pattern: CorePattern::Variant {
                tag: "none".into(),
                payload: None,
            },
            body: CoreExpr::Lit(CoreLiteral::Int(0)),
        },
        MatchArm {
            pattern: CorePattern::Variant {
                tag: "some".into(),
                payload: Some(Box::new(CorePattern::Bind("x".into()))),
            },
            body: CoreExpr::Var("x".into()),
        },
        MatchArm {
            pattern: CorePattern::Wildcard,
            body: CoreExpr::Lit(CoreLiteral::Int(1)),
        },
    ];
    let _ = first_unreachable_arm(&arms, &["none", "some"]);
    let _ = DataEnv::default().adt_for_tag("missing");
}
