//! Round-28 core: nested elaborate `?` Err via quarantined/bad atoms in
//! record/list/match/cast/local nests + check insert_casts / cast Singleton tips.

use reciplexa_core::cast::{
    decide_subtype, intersect_types, judge_dynamic_use, plan_cast_evidence, types_disjoint,
    CastEvidence,
};
use reciplexa_core::check::{
    coerce_to_static, insert_implicit_casts, typecheck_language_source, TypeEnv,
};
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::{CoreType, SingletonValue};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn elaborate_round28_nested_atom_err_matrix() {
    let bad = "(page a4)";
    let cases = [
        // record / field / update / extend nests
        &format!("(val main (record (a {bad})))"),
        &format!("(val main (field {bad} a))"),
        &format!("(val main (record-update {bad} (a 1)))"),
        &format!("(val main (record-update (record (a 1)) (a {bad})))"),
        &format!("(val main (record-extend {bad} (b 1)))"),
        &format!("(val main (record-extend (record (a 1)) (b {bad})))"),
        // list / seq / ctor arity
        &format!("(val main (list {bad}))"),
        &format!("(val main (list 1 {bad}))"),
        &format!("(val main (seq {bad}))"),
        &format!("(val main (seq 1 {bad}))"),
        "(data t (b int))\n(val main (b (page a4)))",
        "(data t (c int string))\n(val main (c 1 (page a4)))",
        "(data t (c int string))\n(val main (c (page a4) \"x\"))",
        // match / perform / raise / or-raise / as-result
        &format!("(val main (match {bad} (_ -> 0)))"),
        &format!("(val main (match 1 (_ -> {bad})))"),
        &format!("(val main (perform ask {bad}))"),
        &format!("(val main (raise {bad}))"),
        &format!("(val main (or-raise {bad}))"),
        &format!("(val main (as-result {bad}))"),
        // casts
        &format!("(val main (try-cast {bad} int))"),
        &format!("(val main (check-cast {bad} int))"),
        &format!("(val main (as int {bad}))"),
        &format!("(val main (as {bad} 1))"),
        // handle / with / if / set / let / letrec / local / var
        &format!("(val main (handle ask (fn (m k) (k m)) {bad}))"),
        &format!("(val main (handle ask {bad} 1))"),
        &format!("(val main (with {bad} 1))"),
        &format!("(val main (with (handler ask (fn (m k) (k m))) {bad}))"),
        &format!("(val main (if {bad} 1 0))"),
        &format!("(val main (if true {bad} 0))"),
        &format!("(val main (if true 1 {bad}))"),
        &format!("(val main (set x {bad}))"),
        &format!("(val main (let ((x {bad})) x))"),
        &format!("(val main (let ((x 1)) {bad}))"),
        &format!("(val main (letrec ((f (fn (x) {bad}))) (f 1)))"),
        &format!("(val main (local (val x {bad}) x))"),
        &format!("(val main (local (val x 1) {bad}))"),
        &format!("(val main (var x {bad} x))"),
        // tuple / unicode / bytes nested
        &format!("(val main (tuple {bad} 1))"),
        &format!("(val main (tuple 1 {bad}))"),
        "(val main (unicode (page a4)))",
        "(val main (bytes (page a4)))",
        // nested ErrorNode / empty
        "(val main (record (a )))",
        "(val main (match 1 ( -> 0) (_ -> 1)))",
        "(val main (let ((x )) x))",
        // pattern nested payload Err
        "(data w (mk int))\n(val main (match (mk 1) ((mk (page a4)) -> 0) (_ -> 1)))",
        "(val main (match (tuple 1 2) ((tuple (page a4) b) -> 0) (_ -> 1)))",
        "(val main (match (record (a 1)) ((record (a (page a4))) -> 0) (_ -> 1)))",
        // data payload nested Err
        "(data t (c (fn int int)))\n(val main 1)",
        "(rec (data t (c)))\n(val main (c))",
        "(rec (data))\n(val main 1)",
        // type row last + empty param already tipped; nested as
        "(val main (as (record (a int) (row r)) 1))",
        "(val main (as (record (row r) (a int)) 1))",
        // ambient perform arity Ok nests still useful nearby
        "(val main (perform log \"x\"))",
        "(val main (perform random))",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn check_round28_insert_casts_and_open_record() {
    // insert_implicit_casts covers Cast/TryCast/CheckCast/Seq/App coerce arms
    let env = TypeEnv::new();
    let lit = CoreExpr::Lit(CoreLiteral::Int(1));
    for expr in [
        CoreExpr::Seq(vec![lit.clone(), lit.clone()]),
        CoreExpr::Cast {
            expr: Box::new(lit.clone()),
            evidence: reciplexa_core::cast::CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 1,
        },
        CoreExpr::TryCast {
            expr: Box::new(lit.clone()),
            target: CoreType::Int,
            cast_id: 2,
        },
        CoreExpr::CheckCast {
            expr: Box::new(lit.clone()),
            target: CoreType::Int,
            cast_id: 3,
        },
        CoreExpr::Let {
            name: "x".into(),
            value: Box::new(lit.clone()),
            body: Box::new(CoreExpr::Var("x".into())),
        },
        CoreExpr::If {
            cond: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
            then_branch: Box::new(lit.clone()),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        CoreExpr::Lambda {
            params: vec!["x".into(), "_".into()],
            body: Box::new(CoreExpr::Var("x".into())),
        },
        CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Var("x".into())),
            }),
            args: vec![lit.clone()],
        },
        CoreExpr::Match {
            scrutinee: Box::new(lit.clone()),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: lit.clone(),
            }],
        },
        CoreExpr::HandlerValue {
            op: "ask".into(),
            handler_params: vec!["m".into()],
            handler_body: Box::new(lit.clone()),
        },
    ] {
        let _ = insert_implicit_casts(&expr, &env);
    }

    // coerce Identity / Cast / Err leaves
    let _ = coerce_to_static(lit.clone(), &CoreType::Int, &CoreType::Int, 1);
    let _ = coerce_to_static(lit.clone(), &CoreType::Int, &CoreType::Number, 2);
    let _ = coerce_to_static(lit.clone(), &CoreType::dyn_any(), &CoreType::Int, 3);
    let _ = coerce_to_static(lit.clone(), &CoreType::String, &CoreType::Int, 4);
    let _ = coerce_to_static(
        lit,
        &CoreType::Union(vec![CoreType::Int, CoreType::String]),
        &CoreType::Bool,
        5,
    );

    // parameterized ADT + open-record / match / handle surfaces
    for src in [
        "(data box ((a type)) (mk a))\n(val main (mk 1))",
        "(data pair ((a type)(b type)) (mk a b))\n(val main (mk 1 \"x\"))",
        "(data opt (none) (some int))\n(val main (match (some 1) ((some x) -> x) (none -> 0)))",
        "(type r (record (a int) (row rho)))\n(val main (fn (x) (.: x a)))",
        "(type r (record (optional a int)))\n(val main (fn (x) (.: x a)))",
        "(val main (record-update (record (a 1)) (a 2)))",
        "(val main (record-extend (record (a 1)) (b 2)))",
        "(val main (handle ask (fn (m k) (k m)) (perform ask 1)))",
        "(val main (with (handler ask (fn (m k) (k m))) 1))",
        "(val main (letrec ((f (fn (x) (f x)))) f))",
        "(val main (var s 1 (fn () s)))",
        "(val main (as (union int string) 1))",
        "(val main (check-cast (intersect int number) 1))",
        "(val main (try-cast (diff int string) 1))",
        "(val main (fn (x) (if (number? x) (+ x 1) 0)))",
        "(val main (fn (x) (if (string? x) x \"\")))",
        "(val main (set x 1))",
    ] {
        tip(src);
    }
}

#[test]
fn cast_round28_numeric_union_singleton_decidable() {
    // Drive plan_numeric_promote via plan_cast_evidence / judge_dynamic_use
    // so Union+Singleton(Int) filter arm is hit.
    let src_partial = CoreType::Union(vec![
        CoreType::Singleton(SingletonValue::Int(1)),
        CoreType::String,
    ]);
    let _ = plan_cast_evidence(&src_partial, &CoreType::F64);
    let _ = judge_dynamic_use(&src_partial, &CoreType::F64);
    let _ = plan_cast_evidence(
        &CoreType::Union(vec![
            CoreType::Singleton(SingletonValue::Int(2)),
            CoreType::Int,
            CoreType::F64,
        ]),
        &CoreType::F64,
    );
    let _ = plan_cast_evidence(
        &CoreType::Union(vec![CoreType::Bool, CoreType::String]),
        &CoreType::F64,
    );

    // Singleton disjoint / subtype / decide residuals
    let s1 = CoreType::Singleton(SingletonValue::Int(1));
    let s2 = CoreType::Singleton(SingletonValue::Int(2));
    let _ = types_disjoint(&s1, &CoreType::F64);
    let _ = types_disjoint(&s1, &s2);
    let _ = types_disjoint(&s1, &CoreType::Number);
    let _ = decide_subtype(&s1, &CoreType::Number);
    let _ = decide_subtype(&s1, &CoreType::String);
    let _ = plan_cast_evidence(&s1, &CoreType::F64);
    let _ = plan_cast_evidence(&s1, &CoreType::Number);
    let _ = judge_dynamic_use(&s1, &CoreType::F64);

    // Decidable Record / Variant / Diff / Union nests via public decide/plan
    for ty in [
        CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        },
        CoreType::Variant {
            variants: vec![("a".into(), Some(CoreType::Int)), ("b".into(), None)],
        },
        CoreType::Diff(Box::new(CoreType::Number), Box::new(CoreType::Int)),
        CoreType::Union(vec![CoreType::Int, CoreType::String]),
        CoreType::Intersect(vec![CoreType::Int, CoreType::Number]),
        CoreType::Not(Box::new(CoreType::String)),
        CoreType::OptionalField(Box::new(CoreType::Int)),
    ] {
        let _ = decide_subtype(&ty, &CoreType::Any);
        let _ = plan_cast_evidence(&CoreType::dyn_any(), &ty);
        let _ = plan_cast_evidence(&ty, &CoreType::dyn_any());
        let _ = types_disjoint(&ty, &CoreType::Never);
    }
    let _ = CastEvidence::Identity;
    // Direct intersect_types singleton arms (public)
    let _ = intersect_types(
        &CoreType::Singleton(SingletonValue::Int(1)),
        &CoreType::Number,
    );
    let _ = intersect_types(
        &CoreType::Singleton(SingletonValue::Int(1)),
        &CoreType::Union(vec![CoreType::Int, CoreType::String]),
    );
    let _ = intersect_types(&CoreType::Singleton(SingletonValue::Int(1)), &CoreType::F64);
}
