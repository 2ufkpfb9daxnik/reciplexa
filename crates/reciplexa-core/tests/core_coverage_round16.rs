//! Round-16 core: residual elaborate/check clusters from llvm miss map.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

#[test]
fn elaborate_round16_quarantine_and_type_singletons() {
    let cases = [
        "(page a4 (circle 1 2 3))",
        "(doc \"title\")",
        "(type orphan int)\n(val main 1)",
        "(type t int)\n(val main 1)",
        "(type t (singleton 1.5))\n(val main 1)",
        "(type t (singleton \"hi\"))\n(val main 1)",
        "(type t (dynamic int string))\n(val main 1)",
        "(type t (not int string))\n(val main 1)",
        "(type t (diff int))\n(val main 1)",
        "(type t (effects ask))\n(val main 1)",
        "(type t (int))\n(val main 1)",
        "(val main 1.5)",
        "(val main 1.0e2)",
        "(val main (unicode 1.5))",
        "(val main (unicode (+ 1.0 2.0)))",
        "(val main foo/bar)",
        "(rec)\n(val main 1)",
        "(rec (data))\n(val main 1)",
    ];
    for src in cases {
        let _ = elaborate_source(src);
        let _ = elaborate_with_data(src);
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn elaborate_round16_pattern_literal_and_bind_errs() {
    let cases = [
        "(val main (match 1 (1 x -> 0) (_ -> 1)))",
        "(val main (match 1 (\"x\" x -> 0) (_ -> 1)))",
        "(val main (match 1 (1.5 x -> 0) (_ -> 1)))",
        "(val main (match true (true x -> 0) (_ -> 1)))",
        "(val main (match false (false x -> 0) (_ -> 1)))",
        "(val main (match unit (unit x -> 0) (_ -> 1)))",
        "(val main (match 1 ((bind 1 2) -> 0) (_ -> 1)))",
        "(val main (match 1 ((bind) -> 0)))",
        "(val main (match 1 (_ x -> 0) (_ -> 1)))",
        "(val main (match 1 (true -> 0) (_ -> 1)))",
        "(val main (match 1 (1 -> 0) (2 ->)))",
    ];
    for src in cases {
        let _ = elaborate_source(src);
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn check_round16_open_record_extend_update_and_match() {
    let ok = [
        r#"(type open (record (row r)))
(val main (fn (x) (. (as open x) missing)))"#,
        r#"(val main (record-extend (record (a 1)) (b 2)))"#,
        r#"(val main (record-update (record (a 1)(b 2)) (a 3)))"#,
        r#"(data opt (none) (some x))
(val main (match (some 1) (none -> 0) (some x -> x)))"#,
        r#"(data opt (none) (some x))
(val main (match (some 1) (none x -> 0) (some y -> y)))"#,
        r#"(data box ((a type)) (mk a))
(val mk (fn (x) x))
(val main (mk 1))"#,
        r#"(val main (if (number? 1) 1 0))"#,
        r#"(val main (let ((x (dynamic 1))) (if (number? x) x 0)))"#,
        r#"(val main (seq 1 2))"#,
        r#"(val main (handle log (fn (msg) msg) (perform log "hi")))"#,
        r#"(val h (handler ask (fn (_ k) (k 1))))
(val main (with h (perform ask unit)))"#,
    ];
    for src in ok {
        let _ = typecheck_language_source(src);
        let _ = elaborate_source(src);
    }

    let err = [
        r#"(val main (record-extend (record (a 1)) (a 2)))"#,
        r#"(val main (record-update (record (a 1)) (b 2)))"#,
        r#"(val main (. 1 a))"#,
    ];
    for src in err {
        assert!(
            typecheck_language_source(src).is_err() || elaborate_source(src).is_err(),
            "{src}"
        );
    }
}

#[test]
fn check_round16_cast_handle_forward_and_raise() {
    let cases = [
        r#"(val main (try-cast 1 int))"#,
        r#"(val main (check-cast "a" string))"#,
        r#"(val main (check-cast 1 string))"#,
        r#"(val main (as-result (fn () 1)))"#,
        r#"(val main (or-raise (as-result (fn () 1))))"#,
        r#"(val main (handle failure (fn (e) e) (raise "x")))"#,
        r#"(val main
  (handle log (fn (msg) msg)
    (handle log (fn (msg k) (forward k))
      (perform log "outer"))))"#,
        r#"(val main (if false (raise "x") 1))"#,
    ];
    for src in cases {
        let _ = typecheck_language_source(src);
    }
}
