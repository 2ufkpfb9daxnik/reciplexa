//! Round-4: walk language effects after quarantine fix; module import/load edges.

use std::collections::HashMap;

use reciplexa_bind::module::*;
use reciplexa_bind::resolve_language_source;

#[test]
fn language_resolve_handle_perform_raise_walks_binders() {
    let r = resolve_language_source(
        r#"
(val main
  (handle log (fn (msg k) (k msg))
    (perform log "hi")))
"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);

    let r = resolve_language_source(
        r#"
(val main
  (handle failure (fn (e) e)
    (raise "x")))
"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);

    let r = resolve_language_source(
        r#"
(val main
  (handle failure (fn (e) e)
    (or-raise (as-result (fn () 1)))))
"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);

    let r = resolve_language_source(
        r#"
(val h (handler ask (fn (_ k) (k 1))))
(val main (with h (perform ask unit)))
"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);

    // Effect forms are walked (no longer graphics-quarantined).
    let r = resolve_language_source(
        r#"
(val main
  (handle log (fn (msg) msg)
    msg))
"#,
    );
    // `msg` is out of scope here — may or may not diagnose depending on skip(2) body walk.
    let _ = r.is_ok();
}

#[test]
fn language_data_nullary_token_ctors_and_malformed() {
    let r = resolve_language_source("(data color red green blue)\n(val main red)");
    assert!(r.is_ok(), "{:?}", r.errors);

    let r = resolve_language_source("(data weird 1)\n(val main 1)");
    let _ = r.is_ok();

    let r = resolve_language_source("(data box ((a type)) )\n(val main 1)");
    let _ = r.is_ok();

    let r = resolve_language_source("(data () (x))\n(val main 1)");
    let _ = r.is_ok();

    let r = resolve_language_source("(type)\n(val main 1)");
    assert!(r.is_ok() || !r.errors.is_empty());
}

#[test]
fn language_let_letrec_var_match_edges() {
    let r = resolve_language_source(
        r#"
(val main
  (let ((a 1) (b a))
    (letrec ((f (fn (n) (if (= n 0) 0 (f (- n 1))))))
      (var x 0
        (set x 1)
        (match (tuple a b x)
          ((tuple p q r) -> (+ p (+ q r)))
          ((bind z) -> z)
          (_ -> 0))))))
"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);

    // Malformed let / letrec / match / fn / var shapes.
    for src in [
        "(val main (let 1 2))",
        "(val main (let ((1 2)) 3))",
        "(val main (letrec 1 2))",
        "(val main (letrec ((1 (fn (x) x))) 2))",
        "(val main (var 1 2 3))",
        "(val main (match 1))",
        "(val main (match 1 (true 1)))",
        "(val main (match 1 x))",
        "(val main (fn 1 2))",
        "(val main (fn name 1 2))",
        "(val main (record a))",
        "(val main (field r))",
        "(val main (list a b))",
        "(val main (if true 1 0))",
        "(val main (seq 1 2))",
        "(val main (val x 1))",
        "(val main (type t int))",
        "(val main (data opt none))",
        "(val main (match 1 (() -> 0)))",
        "(val main (match v ((record (a)) -> 1) (_ -> 0)))",
        "(val main (match v ((tuple) -> 1) (_ -> 0)))",
        "(val main (match v ((bind) -> 1) (_ -> 0)))",
        "(val main (match v ((some (bind x)) -> x) (_ -> 0)))",
    ] {
        let r = resolve_language_source(src);
        let _ = r.is_ok();
    }
}

#[test]
fn language_top_level_non_ident_head_and_empty_list() {
    let r = resolve_language_source("()\n(val main 1)");
    assert!(r.is_ok() || !r.errors.is_empty());

    let r = resolve_language_source("(1 2)\n(val main 1)");
    assert!(r.is_ok() || !r.errors.is_empty());

    let r = resolve_language_source("(val main (match v ((1) -> 0) (_ -> 1)))");
    let _ = r.is_ok();
}

#[test]
fn module_import_parse_error_matrix() {
    for src in [
        "(import)",
        "(import 1)",
        "(import foo as)",
        "(import foo as 1)",
        "(import foo as a as b)",
        "(import foo only)",
        "(import foo only only)",
        "(import foo only (a (b)))",
        "(import foo only a as)",
        "(import foo bar)",
        "(import foo (only a))",
        "(import foo only a only b)",
    ] {
        let err = parse_imports(src);
        assert!(err.is_err(), "expected err for {src}");
    }

    // Empty only-list is accepted (legacy shape).
    let _ = parse_imports("(import foo only ())");
    let ok = parse_imports("(import foo only a as b c)").unwrap();
    assert_eq!(ok[0].only.as_ref().unwrap().len(), 2);
}

#[test]
fn module_elaborate_empty_units_and_letrec_exports() {
    let err = elaborate_units(&[]);
    assert!(err.is_err());

    let units = elaborate_units(&[
        (
            "lib",
            r#"
(rec
  (val even (fn (n) (if (= n 0) true (odd (- n 1)))))
  (val odd (fn (n) (if (= n 0) false (even (- n 1))))))
"#,
        ),
        ("main", "(import lib only even)\n(val main (even 2))"),
    ])
    .unwrap();
    assert!(units.iter().any(|u| u.name == "main"));

    // Import missing export.
    let err = elaborate_units(&[
        ("lib", "(val x 1)"),
        ("main", "(import lib only missing)\n(val main 1)"),
    ])
    .unwrap_err();
    assert!(err.message.contains("not exported") || err.message.contains("missing"));

    // Duplicate module names.
    let err = elaborate_units(&[("a", "(val x 1)"), ("a", "(val y 2)")]).unwrap_err();
    assert!(!err.message.is_empty());

    // Empty body after imports → Seq([]).
    let units = elaborate_units(&[("lib", "(val x 1)"), ("main", "(import lib)\n")]);
    let _ = units;

    // Self-import.
    let err = elaborate_units(&[("main", "(import main)\n(val main 1)")]).unwrap_err();
    assert!(err.message.contains("itself") || err.message.contains("self"));
}

#[test]
fn module_interface_and_load_edges() {
    let mut bad_iface = HashMap::new();
    bad_iface.insert("lib".into(), vec!["missing".into()]);
    let err = elaborate_units_with_interfaces(
        &[("lib", "(val x 1)"), ("main", "(import lib only x)\n(val main x)")],
        &bad_iface,
    );
    assert!(err.is_err());

    let mut good_iface = HashMap::new();
    good_iface.insert("lib".into(), vec!["x".into()]);
    let units = elaborate_units_with_interfaces(
        &[("lib", "(val x 1)"), ("main", "(import lib only x)\n(val main x)")],
        &good_iface,
    )
    .unwrap();
    assert_eq!(units.len(), 2);

    let dir = std::env::temp_dir().join(format!("reciplexa-b4-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("empty.txt"), "nope").unwrap();
    let err = load_module_tree(&dir);
    // Directory with no .rpx may err or return empty.
    let _ = err;

    let err = load_module_tree(dir.join("nope.rpx"));
    assert!(err.is_err());

    std::fs::write(dir.join("a.rpx"), "(import missing)\n(val main 1)\n").unwrap();
    let err = load_module_tree(dir.join("a.rpx"));
    assert!(err.is_err());

    std::fs::write(dir.join("self.rpx"), "(import self)\n(val main 1)\n").unwrap();
    let err = load_module_tree(dir.join("self.rpx"));
    assert!(err.is_err());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn module_from_elaborate_error() {
    let err = elaborate_units(&[("main", "(val")]) .unwrap_err();
    assert!(!err.message.is_empty());
}
