//! Round-14 core: denser elaborate/check/unify/cast residual matrices.

use reciplexa_core::cast::{
    compose_evidence, decide_subtype, is_runtime_checkable, plan_cast_evidence, simplify_evidence,
    CastEvidence, DecideResult,
};
use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::ty::CoreType;
use reciplexa_core::unify::{unify, Subst};

#[test]
fn elaborate_round14_err_matrix() {
    let cases: &[&str] = &[
        "(val main (data T (a)) (val main 1))",
        "(val main (data T a) (val main 1))",
        "(val main (data T (a int) (b)) (val main 1))",
        "(type-alias bad (union int))",
        "(val main (record-update (record (a 1)) (b 2)))",
        "(val main (record-extend (record (a 1)) (a 2)))",
        "(val main (match 1 (1.0 -> 0) (_ -> 1)))",
        "(val main (match 1 ((record (a 1)) -> 0) (_ -> 1)))",
        "(val main (match (tuple 1) ((tuple a b) -> a) (_ -> 0)))",
        "(val main (match (some 1) ((some x y) -> x) (_ -> 0)))",
        "(val main (bytes))",
        "(val main (bytes 1 2 3))",
        "(val main 0x)",
        "(val main 0b)",
        "(val main 0o)",
        "(val main (handle failure (fn (e r) e) (raise \"x\")))",
        "(val main (forward))",
        "(val main (perform))",
        "(rec (val x 1) (val x 2))",
        "(val main (set missing 1))",
        "(val main (. 1 a))",
        "(val main (if 1 2 3))",
        "(val main (1 2))",
        "(val main (try-cast \"a\" int))",
        "(val main (check-cast 1 string))",
        "(val main (unicode))",
        "(val main (unicode 1 2))",
        "(val main (unicode \"x\"))",
        "(val main (unicode (fn (x) x)))",
        "(val main (handle ask (fn (m k r) m) 1))",
        "(unknown-head 1)",
        "(; only comment)",
    ];
    for src in cases {
        let _ = elaborate_source(src);
        let _ = elaborate_with_data(src);
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn elaborate_round14_ok_tails() {
    for src in [
        "(val main (unicode 0x41))",
        "(val main (unicode (+ 30 5)))",
        "(val main (match true (true -> 1) (false -> 0)))",
        "(val main (match unit (unit -> 1) (_ -> 0)))",
        "(val main (match (tuple 1 2) ((tuple a b) -> a) (_ -> 0)))",
        "(val main (match (some 1) ((some x) -> x) (none -> 0)))",
        "(val main (record-update (record (a 1)(b 2)) (a 3)))",
        "(val main (record-extend (record (a 1)) (b 2)))",
        "(val main (handle ask (fn (m k) (k m)) (perform ask \"q\")))",
        "(val main (local (var y 0) (set y 1) y))",
        "(type open (record (row r)))\n(val main (fn (x) (. (as open x) missing)))",
        "(val main (check-cast 1 int))",
        "(val main (try-cast 1 int))",
    ] {
        let _ = elaborate_source(src);
        let _ = elaborate_with_data(src);
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn cast_unify_round14_compose_and_decide() {
    let nested = CastEvidence::Compose(vec![
        CastEvidence::UnionCheck {
            members: vec![CoreType::Int, CoreType::F64],
        },
        CastEvidence::NumericPromote,
        CastEvidence::Compose(vec![
            CastEvidence::Identity,
            CastEvidence::TagCheck { tag: "int".into() },
        ]),
    ]);
    let _ = simplify_evidence(nested);
    let _ = compose_evidence(vec![
        CastEvidence::Identity,
        CastEvidence::TagCheck { tag: "int".into() },
    ]);
    let _ = is_runtime_checkable(&CoreType::Int);
    let _ = plan_cast_evidence(&CoreType::Int, &CoreType::F64);
    let _ = plan_cast_evidence(
        &CoreType::Union(vec![CoreType::Int, CoreType::String]),
        &CoreType::Int,
    );
    let _ = decide_subtype(&CoreType::Int, &CoreType::Number);
    assert_eq!(
        decide_subtype(&CoreType::Int, &CoreType::String),
        DecideResult::Disproved
    );

    let mut subst = Subst::new();
    let _ = unify(&CoreType::Int, &CoreType::F64, &mut subst);
    let _ = unify(
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        },
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Int), ("b".into(), CoreType::String)],
        },
        &mut subst,
    );
}
