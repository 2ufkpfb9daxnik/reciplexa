//! Language-kernel conformance suite (LEX→…→EVAL path).
//!
//! Aggregates elaborate+eval, resolve_language, expand_language, and
//! typecheck_language. Document / page / circle surface checks live in
//! `reciplexa-types` (DOCUMENT SURFACE), not here.

use reciplexa_bind::resolve_language_source;
use reciplexa_core::{elaborate_source, typecheck_language_source, CoreType};
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_macro::expand_language;
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn lang_elaborate_and_eval_identity() {
    let case = ConformanceCase::new(
        "TEST-LANG-EVAL-001",
        "EVAL-001",
        "elaborate + eval identity application",
    );
    run_conformance(&case, || {
        let expr = elaborate_source("(val main ((fn (x) x) 42))").unwrap();
        let _ = expr;
        let v = eval_source("(val main ((fn (x) x) 42))").unwrap();
        assert_eq!(v, RuntimeValue::Number(42.0));
    });
}

#[test]
fn lang_resolve_language_shadowing() {
    let case = ConformanceCase::new(
        "TEST-LANG-RES-001",
        "RES-001",
        "resolve_language_source shadowing",
    );
    run_conformance(&case, || {
        let r = resolve_language_source("(val x 1) (val main (let ((x 2)) x))");
        assert!(r.is_ok(), "{:?}", r.errors);
    });
}

#[test]
fn lang_expand_language_call1() {
    let case = ConformanceCase::new(
        "TEST-LANG-MAC-001",
        "MAC-001",
        "expand_language user macro before eval",
    );
    run_conformance(&case, || {
        let src = r#"
(macro call1 ($f $x) -> ($f $x))
(val main (call1 (fn (n) n) 9))
"#;
        let expanded = expand_language(src).unwrap();
        assert!(!expanded.contains("call1") || expanded.contains("fn"));
        let v = eval_source(src).unwrap();
        assert_eq!(v, RuntimeValue::Number(9.0));
    });
}

#[test]
fn lang_typecheck_language_identity_number() {
    let case = ConformanceCase::new(
        "TEST-LANG-TYP-001",
        "TYP-001",
        "typecheck_language_source identity app is Number",
    );
    run_conformance(&case, || {
        let ty = typecheck_language_source("(val main ((fn (x) x) 1))").unwrap();
        assert_eq!(ty, CoreType::Number);
    });
}

#[test]
fn lang_handle_shallow_v0() {
    let case = ConformanceCase::new(
        "TEST-LANG-EFF-001",
        "EFF-001",
        "shallow handle catches perform",
    );
    run_conformance(&case, || {
        let v =
            eval_source(r#"(val main (handle log (fn (msg) msg) (perform log "ok")))"#).unwrap();
        assert_eq!(v, RuntimeValue::String("ok".into()));
    });
}

#[test]
fn lang_data_match() {
    let case = ConformanceCase::new("TEST-LANG-DAT-001", "DAT-001", "data/match elaborates to 1");
    run_conformance(&case, || {
        let src = include_str!("../../../examples/lang_match.rpx");
        let v = eval_source(src).unwrap();
        assert_eq!(v, RuntimeValue::Number(1.0));
        let ty = typecheck_language_source(src).unwrap();
        assert_eq!(ty, CoreType::Number);
    });
}

#[test]
fn lang_data_match_non_exhaustive() {
    let case = ConformanceCase::new(
        "TEST-LANG-DAT-002",
        "DAT-001",
        "non-exhaustive match is a static error",
    );
    run_conformance(&case, || {
        let src = r#"
(data option (none) (some x))
(val main (match (some 1) (some x -> x)))
"#;
        let err = elaborate_source(src).unwrap_err();
        assert!(err.message.contains("non-exhaustive"), "{}", err.message);
        let err = typecheck_language_source(src).unwrap_err();
        assert!(err.message.contains("non-exhaustive"), "{}", err.message);
    });
}

#[test]
fn lang_literal_tuple_multipayload_patterns() {
    let case = ConformanceCase::new(
        "TEST-LANG-DAT-003",
        "DAT-001",
        "literal, tuple, and multi-payload patterns",
    );
    run_conformance(&case, || {
        let v = eval_source(
            r#"
(val main (match 0 (0 -> "zero") (_ -> "other")))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::String("zero".into()));

        let v = eval_source(
            r#"
(val main
  (match (tuple 1 2)
    (tuple a b -> (+ a b))))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::Number(3.0));

        let v = eval_source(
            r#"
(data pair (pair x y))
(val main (match (pair 4 5) (pair a b -> (* a b))))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::Number(20.0));
    });
}

#[test]
fn lang_letrec() {
    let case = ConformanceCase::new("TEST-LANG-BND-001", "BND-001", "letrec self-call");
    run_conformance(&case, || {
        let src = include_str!("../../../examples/lang_letrec.rpx");
        let v = eval_source(src).unwrap();
        assert_eq!(v, RuntimeValue::Number(7.0));
    });
}

#[test]
fn lang_toplevel_rec_and_local_var() {
    let case = ConformanceCase::new(
        "TEST-LANG-BND-rec-local",
        "BND-001",
        "top-level rec and local var decls",
    );
    run_conformance(&case, || {
        let v = eval_source(
            r#"
(rec
  (val even?
    (fn (n)
      (if (= n 0) true (odd? (- n 1)))))
  (val odd?
    (fn (n)
      (if (= n 0) false (even? (- n 1))))))
(val main (even? 3))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::Bool(false));

        let v = eval_source(
            r#"
(val main
  (local
    (var n 1)
    (rec
      (val bump (fn () (set n (+ n 1)))))
    (seq (bump) n)))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::Number(2.0));
    });
}

#[test]
fn lang_primitives() {
    let case = ConformanceCase::new("TEST-LANG-KER-001", "KER-001", "numeric primitives");
    run_conformance(&case, || {
        let src = include_str!("../../../examples/lang_prim.rpx");
        let v = eval_source(src).unwrap();
        assert_eq!(v, RuntimeValue::Number(10.0));
    });
}

#[test]
fn lang_deep_resume() {
    let case = ConformanceCase::new(
        "TEST-LANG-EFF-001-deep",
        "EFF-001",
        "deep one-shot resume continues body",
    );
    run_conformance(&case, || {
        let v =
            eval_source(r#"(val main (handle ask (fn (_ k) (k 41)) (seq (perform ask 0) 99)))"#)
                .unwrap();
        assert_eq!(v, RuntimeValue::Number(99.0));
    });
}

#[test]
fn lang_with_and_handler_value() {
    let case = ConformanceCase::new(
        "TEST-LANG-EFF-with",
        "EFF-001",
        "first-class handler + with sugar",
    );
    run_conformance(&case, || {
        let v = eval_source(
            r#"
(val h (handler log (fn (msg) msg)))
(val main (with h (perform log "via-with")))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::String("via-with".into()));

        let v2 = eval_source(
            r#"
(val main
  (with (handler ask (fn (_ k) (k 7)))
    (seq (perform ask 0) 42)))
"#,
        )
        .unwrap();
        assert_eq!(v2, RuntimeValue::Number(42.0));
    });
}

#[test]
fn lang_var_set() {
    let case = ConformanceCase::new("TEST-LANG-BND-var", "BND-001", "var/set cell");
    run_conformance(&case, || {
        let v = eval_source(
            r#"
(val main
  (var count 0
    (set count 3)
    count))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::Number(3.0));
    });
}

#[test]
fn lang_surface_type_decls_and_dynamic() {
    let case = ConformanceCase::new(
        "TEST-LANG-TYP-type",
        "TYP-001",
        "surface (type name Ty) and (dynamic)",
    );
    run_conformance(&case, || {
        let (expr, data) = reciplexa_core::elaborate_with_data(
            r#"
(type title str)
(type blob (dynamic))
(type either (union int str))
(val title "ok")
(val main title)
"#,
        )
        .unwrap();
        assert!(matches!(
            data.type_aliases.get("title"),
            Some(CoreType::String)
        ));
        assert!(matches!(
            data.type_aliases.get("blob"),
            Some(CoreType::Dynamic)
        ));
        assert!(matches!(
            data.type_aliases.get("either"),
            Some(CoreType::Union(_))
        ));
        let _ = expr;
        let v = eval_source(
            r#"
(type title str)
(val title "ok")
(val main title)
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::String("ok".into()));
        let ty = typecheck_language_source(
            r#"
(type title str)
(val title "ok")
(val main title)
"#,
        )
        .unwrap();
        assert_eq!(ty, CoreType::String);
    });
}
