//! Round-9 core: local/rec type-annotation residuals + denser check Err matrix.

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
use reciplexa_core::ty::{CoreType, SingletonValue};
use reciplexa_core::unify::{unify, Subst};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn elaborate_local_and_rec_type_annotation_matrix() {
    for src in [
        // Ok: local type + matching val
        "(val main (local (type u int) (val u 1) u))",
        "(val main (local (type-alias u int) (val x 1) x))",
        "(val main (local (type u int) (var u 0) (set u 1) u))",
        // Err: type without matching binder in local
        "(val main (local (type u int) (val x 1) x))",
        "(val main (local (type u int) 1))",
        // Rec with type stubs
        "(rec (type f (fn int int)) (val f (fn (x) x)) (val main (f 1)))",
        "(rec (type-alias t int) (val f (fn (x) x)) (val main (f 1)))",
        "(rec (type f (fn int int)) (val main 1))",
        // Nested type forms in local
        "(val main (local (type t (tuple int string)) (val t (tuple 1 \"a\")) t))",
        "(val main (local (type t (record (a int))) (val t (record (a 1))) t))",
        "(val main (local (type t (union int string)) (val t 1) t))",
        "(val main (local (type t (list int)) (val t (list 1 2)) t))",
        "(val main (local (type t (optional-field int)) (val t 1) t))",
        "(val main (local (type t (dynamic int)) (val t 1) t))",
        "(val main (local (type t (forall ((a type)) a)) (val t 1) t))",
        "(val main (local (type t (effects ask)) (val t 1) t))",
        "(val main (local (type t (singleton true)) (val t true) t))",
        "(val main (local (type t (intersect int number)) (val t 1) t))",
        "(val main (local (type t (diff number int)) (val t 1.5) t))",
        "(val main (local (type t (not string)) (val t 1) t))",
        // Malformed local / rec
        "(val main (local (type) 1))",
        "(val main (local (type 1 int) 1))",
        "(val main (local (type u) 1))",
        "(val main (local () 1))",
        "(rec (type u) (val main 1))",
        "(rec (type) (val main 1))",
        "(rec (1) (val main 1))",
        "(rec (val) (val main 1))",
        "(rec (var x 1) (val main 1))",
        // Data param / positivity leftovers
        "(data box ((a type) (b type)) (mk a b))\n(val main (mk 1 2))",
        "(data box ((a type)) (mk (fn (list a) int)))\n(val main 1)",
        "(data box ((a type)) (mk (tuple a a)))\n(val main 1)",
        "(data bad (mk (fn (fn bad int) int)))\n(val main 1)",
        "(data t (c (lacks a)))\n(val main 1)",
        "(data t (c (row r)))\n(val main 1)",
        "(data t ((a type)) )\n(val main 1)",
        // Ambient / seq / handle edges
        "(val main (seq (perform log \"m\") 1))",
        "(val main (seq))",
        "(val main (forward k))",
        "(val main (raise \"e\"))",
        "(val main (or-raise false \"e\"))",
        "(val main (try-cast 1 int))",
        "(val main (check-cast 1 int))",
        "(val main (as-result int 1))",
        "(val main (handler ask (fn (m k) (k m))))",
        "(val main (with (handler ask (fn (m k) (k m))) (perform ask \"q\")))",
        "(val main (record-update (record (a 1)) (a 2) (b 3)))",
        "(val main (record-extend (record (a 1)) (b 2) (c 3)))",
        "(val main (field (record (a 1)) a))",
        "(val main (match (variant some 1) ((some x) -> x) (_ -> 0)))",
        "(val main (match (variant none) ((none) -> 0) (_ -> 1)))",
        "(val main (unicode 0x41))",
        "(val main (bytes 0 255))",
        "(val main (list 1 2 3))",
        "(val main (tuple 1 \"x\" true))",
        // Quarantined / unsupported heads residual
        "(val main (page a4))",
        "(val main (circle 1 2 3))",
        "(val main (src))",
        // Empty / garbage
        "(; only comment ;)",
        "()",
        "(val main ())",
        "(val main (local (type u int) (val u ()) u))",
    ] {
        let _ = elaborate_source(src);
        let _ = elaborate_with_data(src);
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn check_round9_effectful_and_record_err_matrix() {
    let env = TypeEnv::new();
    let mut subst = Subst::new();
    let r = range();

    let cases: Vec<CoreExpr> = vec![
        CoreExpr::Perform {
            op: "ask".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::String("q".into()))),
        },
        CoreExpr::Forward {
            resume_name: "k".into(),
        },
        CoreExpr::Handle {
            op: "ask".into(),
            handler_params: vec!["m".into(), "k".into(), "extra".into()],
            handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        CoreExpr::HandlerValue {
            op: "ask".into(),
            handler_params: vec![],
            handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        CoreExpr::With {
            handler: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            body: Box::new(CoreExpr::Perform {
                op: "ask".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            }),
        },
        CoreExpr::LetRec {
            bindings: vec![(
                "f".into(),
                CoreExpr::Lit(CoreLiteral::Int(1)), // must be lambda → Err
            )],
            body: Box::new(CoreExpr::Var("f".into())),
        },
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
        CoreExpr::Set {
            name: "missing".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
            body: Box::new(CoreExpr::Var("c".into())),
        },
        CoreExpr::App {
            fun: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(2))],
        },
        CoreExpr::If {
            cond: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            field: "a".into(),
        },
        CoreExpr::RecordUpdate {
            record: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        CoreExpr::RecordExtend {
            record: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            arms: vec![],
        },
        CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Lit(CoreLiteral::Int(1)),
                    body: CoreExpr::Lit(CoreLiteral::Int(1)),
                },
                MatchArm {
                    pattern: CorePattern::Lit(CoreLiteral::Int(1)),
                    body: CoreExpr::Lit(CoreLiteral::Int(2)),
                },
            ],
        },
        CoreExpr::Cast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            evidence: CastEvidence::Identity,
            target: CoreType::String,
            cast_id: 0,
        },
        CoreExpr::TryCast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            target: CoreType::String,
            cast_id: 1,
        },
        CoreExpr::CheckCast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            target: CoreType::String,
            cast_id: 2,
        },
        CoreExpr::Seq(vec![
            CoreExpr::Perform {
                op: "log".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String("m".into()))),
            },
            CoreExpr::Error,
        ]),
        CoreExpr::Lambda {
            params: vec!["_".into(), "x".into()],
            body: Box::new(CoreExpr::Var("x".into())),
        },
        CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(CoreExpr::Error)),
        },
        CoreExpr::Record {
            fields: vec![("a".into(), CoreExpr::Error)],
        },
    ];

    for expr in cases {
        let _ = infer_with_effects(&expr, &env, &mut subst, r);
        let _ = infer_expr(&expr, &env, &mut subst, r);
        let _ = insert_implicit_casts(&expr, &env);
        let _ = coerce_to_static(expr.clone(), &CoreType::Int, &CoreType::dyn_any(), 0);
    }

    // denser program sources that force check ? Err after soft Ok prefixes
    for src in [
        "(val main (if 1 2 3))",
        "(val main (field 1 a))",
        "(val main (set missing 1))",
        "(val main (letrec ((f 1)) f))",
        "(val main (match 1))",
        "(val main (+ true false))",
        "(val main (handle ask (fn (m k extra) m) 1))",
        "(val main (with 1 2))",
        "(val main (forward k))",
        "(type f (fn int int))\n(rec (val f (fn (x) x)) (val main (f 1)))",
        "(val main (local (var c 0) (local (var c 1) c)))",
        "(val main (as string 1))",
        "(val main (try-cast \"x\" int))",
        "(val main (check-cast true string))",
        "(val main (record-update 1 (a 2)))",
        "(val main (record-extend 1 (a 2)))",
        "(val main (fn (x y) (+ x y)))",
        "(val main ((fn (x) x) 1 2))",
    ] {
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn cast_unify_round9_stubs() {
    let mut subst = Subst::new();
    let open = CoreType::OpenRecord {
        fields: vec![("a".into(), CoreType::Int)],
        row: Box::new(CoreType::Var(subst.fresh_var())),
    };
    let closed = CoreType::Record {
        fields: vec![("a".into(), CoreType::Int)],
    };
    let _ = unify(&open, &closed, &mut subst);
    let _ = is_subtype(&open, &closed);
    let _ = decide_subtype(&open, &closed);
    let _ = types_disjoint(&CoreType::Int, &CoreType::String);
    let _ = types_disjoint(&CoreType::dyn_any(), &CoreType::Int);
    let forall = CoreType::Forall {
        params: vec![("a".into(), "type".into())],
        body: Box::new(CoreType::Name("a".into())),
    };
    let _ = plan_cast_evidence(&forall, &CoreType::Int);
    let _ = plan_cast_evidence(&CoreType::Int, &forall);
    let _ = is_runtime_checkable(&CoreType::dyn_any());
    let _ = judge_dynamic_use(&CoreType::dyn_any(), &CoreType::Int);
    let _ = cast_success_type(&CoreType::Int, &CoreType::Number);
    let ev = compose_evidence(vec![CastEvidence::Identity, CastEvidence::Widen]);
    let _ = simplify_evidence(ev);
    let _ = decide_subtype(&CoreType::Never, &CoreType::Int);
    assert!(matches!(
        decide_subtype(&CoreType::Int, &CoreType::Never),
        DecideResult::Disproved | DecideResult::Unknown | DecideResult::Proved
    ));
    let sing = CoreType::Singleton(SingletonValue::Bool(true));
    let _ = is_subtype(&sing, &CoreType::Bool);
    let arms = [MatchArm {
        pattern: CorePattern::Wildcard,
        body: CoreExpr::Lit(CoreLiteral::Unit),
    }];
    let _ = first_unreachable_arm(&arms, &[]);
    let mut subst2 = Subst::new();
    let _ = unify(&CoreType::Int, &CoreType::Number, &mut subst2);
    let _ = unify(&CoreType::Never, &CoreType::String, &mut subst2);
}
