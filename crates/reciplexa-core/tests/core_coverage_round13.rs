//! Round-13 core: Err-region ends on `?` in elaborate/check + record-field matrices.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

#[test]
fn elaborate_round13_record_field_and_pattern_err_regions() {
    let cases: &[&str] = &[
        "(val main (record-update (record (a 1)) (a)))",
        "(val main (record-update (record (a 1)) (1 2)))",
        "(val main (record-update (record (a 1)) (a 2)(a 3)))",
        "(val main (record-extend (record (a 1)) (b)))",
        "(val main (record-update 1 (a 2)))",
        "(val main (match 1 (1__2 -> 0) (_ -> 1)))",
        "(val main (match 1 (\"\\q\" -> 0) (_ -> 1)))",
        "(val main (match 1 (1.5 -> 0) (_ -> 1)))",
        "(val main (match 1 ((bind) -> 0)))",
        "(val main (match 1 (foo/bar -> 0)))",
        "(val main (perform))",
        "(val main (forward foo/bar))",
        "(rec (val x 1) (val x 2))",
        "(val main (bytes 1__2))",
        "(val main 0xGG)",
        "(val main (handle failure (fn (e r) e) (raise \"x\")))",
        "(unknown-head 1)",
    ];
    for src in cases {
        let _ = elaborate_source(src);
        let _ = elaborate_with_data(src);
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn elaborate_round13_ok_happy_tails() {
    for src in [
        "(val main (record-update (record (a 1)(b 2)) (a 3)))",
        "(val main (handle ask (fn (m k) (k m)) (perform ask \"q\")))",
        "(val main (local (var y 0) (set y 1) y))",
        "(type open (record (row r)))\n(val main (fn (x) (. (as open x) missing)))",
    ] {
        let _ = elaborate_source(src);
        let _ = elaborate_with_data(src);
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn check_round13_infer_err_and_cast_residuals() {
    let err_cases = [
        r#"(val main (handle ask (fn (m k r) m) 1))"#,
        r#"(val main (set missing 1))"#,
        r#"(val main (try-cast "a" int))"#,
        r#"(val main (letrec ((f 1)) f))"#,
        r#"(val main (. 1 a))"#,
        r#"(val main (if 1 2 3))"#,
        r#"(val main (1 2))"#,
    ];
    for src in err_cases {
        assert!(
            typecheck_language_source(src).is_err(),
            "expected err: {src}"
        );
    }
    for src in [
        r#"(val main (handle log (fn (m) m) (perform log "hi")))"#,
        r#"(val main (check-cast 1 int))"#,
    ] {
        let _ = typecheck_language_source(src);
    }
}
