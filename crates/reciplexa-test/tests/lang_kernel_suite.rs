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
(macro call1 (f x) (f x))
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
