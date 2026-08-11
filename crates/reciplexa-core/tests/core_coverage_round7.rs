//! Round-7 core: dense check CoreExpr arms + elaborate residual Err/Ok sweeps.

use reciplexa_core::cast::{
    decide_subtype, is_runtime_checkable, is_subtype, judge_dynamic_use, plan_cast_evidence,
    types_disjoint, DecideResult,
};
use reciplexa_core::check::{
    coerce_to_static, infer_expr, infer_with_effects, insert_implicit_casts,
    typecheck_language_source, TypeEnv,
};
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::expr::{first_unreachable_arm, CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::{CoreType, EffectRow, SingletonValue};
use reciplexa_core::unify::{unify, Subst};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn check_handle_with_failure_and_open_record() {
    let (ty, _) = infer_with_effects(
        &CoreExpr::Handle {
            op: "failure".into(),
            handler_params: vec!["e".into()],
            handler_body: Box::new(CoreExpr::Var("e".into())),
            body: Box::new(CoreExpr::Perform {
                op: "failure".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String("e".into()))),
            }),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    let _ = ty;

    let err = infer_with_effects(
        &CoreExpr::Handle {
            op: "failure".into(),
            handler_params: vec!["e".into(), "k".into()],
            handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());

    let err = infer_with_effects(
        &CoreExpr::Handle {
            op: "ask".into(),
            handler_params: vec![],
            handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());

    let (ty, _) = infer_with_effects(
        &CoreExpr::Handle {
            op: "ask".into(),
            handler_params: vec!["m".into(), "k".into()],
            handler_body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Var("m".into())],
            }),
            body: Box::new(CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
            }),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    let _ = ty;

    let mut env = TypeEnv::new();
    env.insert(
        "r",
        CoreType::OpenRecord {
            fields: vec![("a".into(), CoreType::Int)],
            row: Box::new(CoreType::Var(Subst::new().fresh_var())),
        },
    );
    let mut subst = Subst::new();
    env.insert(
        "r",
        CoreType::OpenRecord {
            fields: vec![("a".into(), CoreType::Int)],
            row: Box::new(CoreType::Var(subst.fresh_var())),
        },
    );
    let ty = infer_expr(
        &CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Var("r".into())),
            field: "a".into(),
        },
        &env,
        &mut subst,
        range(),
    )
    .unwrap();
    assert_eq!(ty, CoreType::Int);

    let ty = infer_expr(
        &CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Var("r".into())),
            field: "b".into(),
        },
        &env,
        &mut subst,
        range(),
    );
    let _ = ty;

    let err = infer_expr(
        &CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            field: "a".into(),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());
}

#[test]
fn check_record_update_extend_match_cast_residuals() {
    let rec = CoreExpr::Record {
        fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
    };
    let err = infer_expr(
        &CoreExpr::RecordUpdate {
            record: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());

    let err = infer_expr(
        &CoreExpr::RecordUpdate {
            record: Box::new(rec.clone()),
            fields: vec![("missing".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());

    let ok = infer_expr(
        &CoreExpr::RecordUpdate {
            record: Box::new(rec.clone()),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(9)))],
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(ok.is_ok());

    let err = infer_expr(
        &CoreExpr::RecordExtend {
            record: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());

    let err = infer_expr(
        &CoreExpr::RecordExtend {
            record: Box::new(rec.clone()),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());

    let ok = infer_expr(
        &CoreExpr::RecordExtend {
            record: Box::new(rec),
            fields: vec![("b".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(ok.is_ok());

    // ADT match exhaustive / unreachable via DataEnv
    let mut env = TypeEnv::new();
    env.data.ctors.insert("none".into(), 0);
    env.data.ctors.insert("some".into(), 1);
    env.data.ctor_type.insert("none".into(), "opt".into());
    env.data.ctor_type.insert("some".into(), "opt".into());
    env.data
        .data_ctors
        .insert("opt".into(), vec![("none".into(), 0), ("some".into(), 1)]);

    let err = infer_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "none".into(),
                payload: None,
            }),
            arms: vec![MatchArm {
                pattern: CorePattern::Variant {
                    tag: "none".into(),
                    payload: None,
                },
                body: CoreExpr::Lit(CoreLiteral::Int(0)),
            }],
        },
        &env,
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());

    let err = infer_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "none".into(),
                payload: None,
            }),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Wildcard,
                    body: CoreExpr::Lit(CoreLiteral::Int(0)),
                },
                MatchArm {
                    pattern: CorePattern::Variant {
                        tag: "some".into(),
                        payload: Some(Box::new(CorePattern::Bind("x".into()))),
                    },
                    body: CoreExpr::Lit(CoreLiteral::Int(1)),
                },
            ],
        },
        &env,
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());

    let err = infer_expr(
        &CoreExpr::Cast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            evidence: reciplexa_core::cast::CastEvidence::Identity,
            target: CoreType::String,
            cast_id: 1,
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());

    let err = infer_expr(
        &CoreExpr::TryCast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            target: CoreType::String,
            cast_id: 2,
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());

    let err = infer_expr(
        &CoreExpr::CheckCast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            target: CoreType::String,
            cast_id: 3,
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());
}

#[test]
fn check_occurrence_numeric_and_generalize_shapes() {
    let pred_ty = CoreType::Fun {
        args: vec![CoreType::dyn_any()],
        ret: Box::new(CoreType::Bool),
        effects: EffectRow::default(),
    };
    let mut env = TypeEnv::new();
    env.insert("x", CoreType::dyn_any());
    env.insert("number?", pred_ty.clone());
    let ty = infer_expr(
        &CoreExpr::If {
            cond: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("number?".into())),
                args: vec![CoreExpr::Var("x".into())],
            }),
            then_branch: Box::new(CoreExpr::Var("x".into())),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        &env,
        &mut Subst::new(),
        range(),
    );
    let _ = ty;

    for pred in ["string?", "bool?"] {
        let mut env = TypeEnv::new();
        env.insert("x", CoreType::Dynamic(Box::new(CoreType::Any)));
        env.insert(pred, pred_ty.clone());
        let _ = infer_expr(
            &CoreExpr::If {
                cond: Box::new(CoreExpr::App {
                    fun: Box::new(CoreExpr::Var(pred.into())),
                    args: vec![CoreExpr::Var("x".into())],
                }),
                then_branch: Box::new(CoreExpr::Var("x".into())),
                else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            },
            &env,
            &mut Subst::new(),
            range(),
        );
    }

    let mut env = TypeEnv::new();
    env.insert(
        "+",
        CoreType::Fun {
            args: vec![CoreType::Int, CoreType::Int],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        },
    );
    env.insert("a", CoreType::Number);
    env.insert("b", CoreType::Number);
    let err = infer_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![CoreExpr::Var("a".into()), CoreExpr::Var("b".into())],
        },
        &env,
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());

    let mut env = TypeEnv::new();
    env.insert(
        "+",
        CoreType::Fun {
            args: vec![CoreType::Int, CoreType::Int],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        },
    );
    env.insert(
        "a",
        CoreType::Intersect(vec![
            CoreType::Union(vec![CoreType::Int, CoreType::F64]),
            CoreType::Number,
        ]),
    );
    env.insert("b", CoreType::Int);
    let _ = infer_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![CoreExpr::Var("a".into()), CoreExpr::Var("b".into())],
        },
        &env,
        &mut Subst::new(),
        range(),
    );

    // Parameterized ctor payload schemas
    let mut env = TypeEnv::new();
    env.data.type_params.insert("box".into(), vec!["a".into()]);
    env.data.ctors.insert("mk".into(), 1);
    env.data.ctor_type.insert("mk".into(), "box".into());
    env.data
        .ctor_payloads
        .insert("mk".into(), vec![CoreType::Name("a".into())]);
    let ty = infer_expr(
        &CoreExpr::Variant {
            tag: "mk".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Int(1)))),
        },
        &env,
        &mut Subst::new(),
        range(),
    );
    let _ = ty;

    env.data.ctor_payloads.insert(
        "pair".into(),
        vec![CoreType::Name("a".into()), CoreType::Int],
    );
    env.data.ctors.insert("pair".into(), 2);
    env.data.ctor_type.insert("pair".into(), "box".into());
    let ty = infer_expr(
        &CoreExpr::Variant {
            tag: "pair".into(),
            payload: Some(Box::new(CoreExpr::Record {
                fields: vec![
                    ("0".into(), CoreExpr::Lit(CoreLiteral::Int(1))),
                    ("1".into(), CoreExpr::Lit(CoreLiteral::Int(2))),
                ],
            })),
        },
        &env,
        &mut Subst::new(),
        range(),
    );
    let _ = ty;

    let err = infer_expr(
        &CoreExpr::Variant {
            tag: "pair".into(),
            payload: None,
        },
        &env,
        &mut Subst::new(),
        range(),
    );
    let _ = err;

    // LetRec annotated principle type
    let mut env = TypeEnv::new();
    env.data.type_aliases.insert(
        "id".into(),
        CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        },
    );
    let ty = infer_expr(
        &CoreExpr::LetRec {
            bindings: vec![(
                "id".into(),
                CoreExpr::Lambda {
                    params: vec!["x".into()],
                    body: Box::new(CoreExpr::Var("x".into())),
                },
            )],
            body: Box::new(CoreExpr::Var("id".into())),
        },
        &env,
        &mut Subst::new(),
        range(),
    );
    let _ = ty;

    let err = infer_expr(
        &CoreExpr::LetRec {
            bindings: vec![("bad".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            body: Box::new(CoreExpr::Var("bad".into())),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());

    let err = infer_expr(
        &CoreExpr::Set {
            name: "gone".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    );
    assert!(err.is_err());
}

#[test]
fn check_coerce_insert_casts_and_complex_types() {
    let _ = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::Int,
        &CoreType::String,
        1,
    );
    let _ = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::Int,
        &CoreType::F64,
        2,
    );
    let _ = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::Dynamic(Box::new(CoreType::String)),
        &CoreType::Int,
        3,
    );
    let _ = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::dyn_any(),
        &CoreType::Number,
        4,
    );

    let mut env = TypeEnv::new();
    env.insert(
        "f",
        CoreType::Fun {
            args: vec![CoreType::F64],
            ret: Box::new(CoreType::F64),
            effects: EffectRow::default(),
        },
    );
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Var("f".into())),
        args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
    };
    let _ = insert_implicit_casts(&expr, &env);

    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        arms: vec![MatchArm {
            pattern: CorePattern::Wildcard,
            body: CoreExpr::Lit(CoreLiteral::Int(0)),
        }],
    };
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());

    let expr = CoreExpr::Handle {
        op: "ask".into(),
        handler_params: vec!["m".into()],
        handler_body: Box::new(CoreExpr::Var("m".into())),
        body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
    };
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());

    let expr = CoreExpr::LocalVar {
        name: "c".into(),
        init: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        body: Box::new(CoreExpr::Set {
            name: "c".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        }),
    };
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());

    let expr = CoreExpr::LetRec {
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
    };
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());

    let expr = CoreExpr::Variant {
        tag: "some".into(),
        payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Int(1)))),
    };
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());

    let expr = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Record {
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
        }),
        field: "a".into(),
    };
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());

    let expr = CoreExpr::Perform {
        op: "log".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
    };
    let _ = insert_implicit_casts(&expr, &TypeEnv::new());

    // Exercise subtype / intersect residual arms
    for (a, b) in [
        (
            CoreType::Singleton(SingletonValue::Int(1)),
            CoreType::Singleton(SingletonValue::Int(2)),
        ),
        (
            CoreType::Singleton(SingletonValue::Int(1)),
            CoreType::Number,
        ),
        (
            CoreType::OptionalField(Box::new(CoreType::Int)),
            CoreType::OptionalField(Box::new(CoreType::String)),
        ),
        (
            CoreType::Union(vec![CoreType::Int, CoreType::String]),
            CoreType::Int,
        ),
        (
            CoreType::Int,
            CoreType::Union(vec![CoreType::Int, CoreType::String]),
        ),
        (
            CoreType::Intersect(vec![CoreType::Int, CoreType::Number]),
            CoreType::Int,
        ),
        (
            CoreType::Int,
            CoreType::Intersect(vec![CoreType::Int, CoreType::Number]),
        ),
        (
            CoreType::Union(vec![CoreType::Int, CoreType::F64]),
            CoreType::F64,
        ),
        (
            CoreType::Union(vec![CoreType::Int, CoreType::String]),
            CoreType::F64,
        ),
        (CoreType::F64, CoreType::F64),
        (CoreType::Any, CoreType::F64),
        (
            CoreType::Lacks {
                label: "a".into(),
                row: Box::new(CoreType::Record { fields: vec![] }),
            },
            CoreType::Record { fields: vec![] },
        ),
        (CoreType::Not(Box::new(CoreType::Int)), CoreType::String),
        (
            CoreType::Diff(Box::new(CoreType::Number), Box::new(CoreType::Int)),
            CoreType::F64,
        ),
    ] {
        let _ = decide_subtype(&a, &b);
        let _ = is_subtype(&a, &b);
        let _ = types_disjoint(&a, &b);
        let _ = plan_cast_evidence(&a, &b);
        let _ = judge_dynamic_use(&a, &b);
        let _ = is_runtime_checkable(&a);
    }
}

#[test]
fn elaborate_residual_err_ok_matrix() {
    for src in [
        // handle / with / handler / ambient
        "(val main (handle))",
        "(val main (handle ask))",
        "(val main (handle ask (fn (m) m)))",
        "(val main (handle 1 (fn (m) m) 1))",
        "(val main (handle ask 1 1))",
        "(val main (handle ask (fn () 1) 1))",
        "(val main (handle ask (fn (a b c) 1) 1))",
        "(val main (handler))",
        "(val main (handler ask))",
        "(val main (handler ask 1))",
        "(val main (handler 1 (fn (m) m)))",
        "(val main (handler ask (fn () 1)))",
        "(val main (with))",
        "(val main (with 1))",
        "(val main (random 1))",
        "(val main (log))",
        "(val main (log 1 2))",
        "(val main (read-file))",
        "(val main (write-file 1))",
        // local / set / if / path
        "(val main (local (type t int) 1))",
        "(val main (local (type t int) (val t 1) t))",
        "(val main (local (val) 1))",
        "(val main (local (val 1 2) 1))",
        "(val main (local (val if 1) if))",
        "(val main (local (var) 1))",
        "(val main (local (var 1 2) 1))",
        "(val main (local (var if 1) if))",
        "(val main (local (rec) 1))",
        "(val main (local (rec (val f 1)) f))",
        "(val main (local (rec (val f (fn (x) x))) f))",
        "(val main (local (unknown x) 1))",
        "(val main (set 1 2))",
        "(val main (set if 1))",
        "(val main foo/bar)",
        "(val main (as (singleton 1) 1))",
        "(val main (as (singleton true) true))",
        "(val main (as (optional-field int) 1))",
        "(val main (as (lacks a (record)) (record)))",
        "(val main (as (dynamic int) 1))",
        "(val main (as (forall ((a type)) a) 1))",
        "(val main (as (union int string) 1))",
        "(val main (as (intersect int number) 1))",
        "(val main (as (not int) 1))",
        "(val main (as (diff number int) 1))",
        "(val main (as (record (optional a int)) 1))",
        "(val main (as (record (a int) (row r)) 1))",
        "(val main (as (open-record (a int) (row r)) 1))",
        "(val main (as (effects) 1))",
        "(val main (match 1 ((bind x) -> x)))",
        "(val main (match 1 ((tuple a b) -> a) (_ -> 0)))",
        "(val main (match (record (a 1)) ((record (a x)) -> x) (_ -> 0)))",
        "(val main (match true (true -> 1) (false -> 0)))",
        "(data opt (none) (some v))\n(val main (match none (none -> 0) (some x -> x)))",
        "(data t (c a b))\n(val main (c 1 2))",
        "(data t (c a b))\n(val main (match (c 1 2) ((c x y) -> x) (_ -> 0)))",
        "(rec (data a (x)) (data b (y)))\n(val main x)",
        "(val main (field (record (a 1)) a))",
        "(val main (record-update (record (a 1)) (a 2)))",
        "(val main (record-extend (record (a 1)) (b 2)))",
        "(val main (try-cast 1 int))",
        "(val main (check-cast 1 int))",
        "(val main (as int 1))",
        "(val main (perform log \"x\"))",
        "(val main (raise \"e\"))",
        "(val main (or-raise (as-result (fn () 1))))",
        "(val main (as-result (fn () 1)))",
        "(val main (forward k))",
        "1.5",
        "0x10",
        "1e3",
        "\"\\n\\t\\\"\\\\\"",
        "#\\space",
        "#\\newline",
        "#\\tab",
        "#\\return",
        "#\\(",
        "(val main [1 2 3])",
        "(val main (seq))",
        "(val main (seq 1 2 3))",
        "(data tree ((a type)) (leaf a) (node (tree a) (tree a)))\n(val main (leaf 1))",
        "(type f (fn int int))\n(val f (fn (x) x))\n(val main (f 1))",
        "(type id (forall ((a type)) (fn a a)))\n(val id (fn (x) x))\n(val main (id 1))",
    ] {
        let _ = elaborate_source(src);
        let _ = typecheck_language_source(src);
    }

    let _ = elaborate_with_data("(data t (c))\n(val main c)");
    let _ = first_unreachable_arm(
        &[
            MatchArm {
                pattern: CorePattern::Variant {
                    tag: "a".into(),
                    payload: None,
                },
                body: CoreExpr::Lit(CoreLiteral::Unit),
            },
            MatchArm {
                pattern: CorePattern::Variant {
                    tag: "a".into(),
                    payload: Some(Box::new(CorePattern::Bind("x".into()))),
                },
                body: CoreExpr::Lit(CoreLiteral::Unit),
            },
        ],
        &["a", "b"],
    );

    let mut subst = Subst::new();
    let a = CoreType::Var(subst.fresh_var());
    let b = CoreType::OpenRecord {
        fields: vec![("x".into(), CoreType::Int)],
        row: Box::new(CoreType::Var(subst.fresh_var())),
    };
    let _ = unify(&a, &b, &mut subst);
    let _ = unify(
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        },
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::String)],
        },
        &mut Subst::new(),
    );
    let _ = unify(
        &CoreType::Variant {
            variants: vec![("a".into(), Some(CoreType::Int))],
        },
        &CoreType::Variant {
            variants: vec![("a".into(), Some(CoreType::String))],
        },
        &mut Subst::new(),
    );
    let _ = unify(
        &CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default().with_op("io"),
        },
        &CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        },
        &mut Subst::new(),
    );
    let _ = decide_subtype(&CoreType::Never, &CoreType::Int);
    let _ = decide_subtype(&CoreType::Int, &CoreType::Any);
    let _: DecideResult = decide_subtype(&CoreType::String, &CoreType::Int);
}
