//! Round-12 core: OpenRecord / pattern / cast compose / unicode residual leaves.

use reciplexa_core::cast::{
    compose_evidence, is_runtime_checkable, plan_cast_evidence, simplify_evidence, CastEvidence,
};
use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::elaborate_source;
use reciplexa_core::ty::CoreType;

#[test]
fn open_record_get_unify_and_update_extend() {
    // Open row: missing field introduces fresh vars via unify on OpenRecord.
    let cases_ok = [
        r#"(type open (record (row r)))
(val main (fn (x)
  (let ((y (as open x)))
    (. y missing))))"#,
        r#"(val main (record-update (record (a 1) (b 2)) (a 3)))"#,
        r#"(val main (record-extend (record (a 1)) (b 2)))"#,
        r#"(val main (. (record (a 1) (b "x")) a))"#,
        r#"(type o (record (a int) (row r)))
(val main (fn (x) (. (as o x) a)))"#,
    ];
    for src in cases_ok {
        let _ = typecheck_language_source(src);
        let _ = elaborate_source(src);
    }
    let cases_err = [
        r#"(val main (record-update 1 (a 2)))"#,
        r#"(val main (record-update (record (a 1)) (b 2)))"#,
        r#"(val main (record-extend 1 (a 2)))"#,
        r#"(val main (record-extend (record (a 1)) (a 2)))"#,
        r#"(val main (. 1 a))"#,
    ];
    for src in cases_err {
        assert!(
            typecheck_language_source(src).is_err() || elaborate_source(src).is_err(),
            "{src}"
        );
    }
}

#[test]
fn pattern_literal_and_bool_unit_residuals() {
    let cases = [
        "(val main (match 1 (1 2 -> 0) (_ -> 1)))", // literal with args
        "(val main (match true (true extra -> 0) (_ -> 1)))",
        "(val main (match false (false x -> 0) (_ -> 1)))",
        "(val main (match unit (unit x -> 0) (_ -> 1)))",
        "(val main (match 1 (\"x\" -> 0) (_ -> 1)))",
        "(val main (match 1 (_ x -> 0)))", // wildcard with args
        "(val main (match 1 ((bind) -> 0) (_ -> 1)))",
        "(val main (match 1 ((bind 1) -> 0) (_ -> 1)))",
        "(val main (match (tuple 1 2) ((tuple 1) -> 0) (_ -> 1)))",
        "(val main (match 1 (true -> 0) (_ -> 1)))",
        "(val main (match true (true -> 1) (false -> 0)))",
        "(val main (match unit (unit -> 1) (_ -> 0)))",
        "(val main (unicode 0x41))",
        "(val main (unicode (+ 30 5)))",
        "(val main (unicode))",
        "(val main (unicode 1 2))",
        "(val main (unicode \"x\"))",
        "(val main (unicode (fn (x) x)))",
    ];
    for src in cases {
        let _ = elaborate_source(src);
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn cast_compose_nested_identity_and_checkable_tails() {
    let nested = CastEvidence::Compose(vec![
        CastEvidence::Identity,
        CastEvidence::Compose(vec![
            CastEvidence::Identity,
            CastEvidence::TagCheck { tag: "int".into() },
            CastEvidence::Identity,
        ]),
        CastEvidence::Identity,
    ]);
    assert_eq!(
        simplify_evidence(nested),
        CastEvidence::TagCheck { tag: "int".into() }
    );
    assert_eq!(compose_evidence(vec![]), CastEvidence::Identity);
    assert!(is_runtime_checkable(&CoreType::Fun {
        args: vec![CoreType::Int],
        ret: Box::new(CoreType::Int),
        effects: Default::default(),
    }));
    assert!(!is_runtime_checkable(&CoreType::Fun {
        args: vec![CoreType::Int],
        ret: Box::new(CoreType::Int),
        effects: reciplexa_core::ty::EffectRow {
            ops: vec!["".into()],
        },
    }));
    let _ = plan_cast_evidence(&CoreType::Any, &CoreType::Int);
    let _ = plan_cast_evidence(
        &CoreType::OpenRecord {
            fields: vec![("a".into(), CoreType::Int)],
            row: Box::new(CoreType::Any),
        },
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        },
    );
}

#[test]
fn handle_with_match_happy_path_region_ends() {
    let srcs = [
        r#"(val main (handle ask (fn (m k) (k m)) (perform ask "q")))"#,
        r#"(val main (with (handler ask (fn (m k) (k m))) (perform ask "q")))"#,
        r#"(data opt (none) (some x))
(val main (match (some 1) (none -> 0) (some x -> x)))"#,
        r#"(val main (match (record (a 1)) ((record (a x)) -> x)))"#,
        r#"(val main (if true 1 0))"#,
        r#"(val main (if false 1 0))"#,
        r#"(val main (seq 1 2 3))"#,
        r#"(val main (as-result (fn () (raise "e"))))"#,
        r#"(val main (or-raise (as-result (fn () 1))))"#,
        r#"(val main (try-cast 1 int))"#,
        r#"(val main (check-cast 1 number))"#,
    ];
    for src in srcs {
        let _ = typecheck_language_source(src);
    }
}
