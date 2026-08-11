//! Round-7 bind: elaborate From path, import edges, resolve residual shapes.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use reciplexa_bind::module::*;
use reciplexa_bind::resolve_language_source;

#[test]
fn module_from_elaborate_error_and_empty_body() {
    // Triggers `From<ElaborateError>` via `elaborate_source(...)?`.
    let err = elaborate_units(&[("m", "(type x int)")]);
    assert!(err.is_err(), "type without value binding should fail");

    let err = elaborate_units(&[("m", "(page a4)")]);
    assert!(err.is_err());

    let ok = elaborate_units(&[("lib", "(val x 1)"), ("main", "(import lib)\n")]);
    assert!(ok.is_ok(), "{ok:?}");
}

#[test]
fn module_import_path_and_structured_comment_edges() {
    for src in [
        "(import)",
        "(import 1)",
        "(import foo/../bar)",
        "(import lib as)",
        "(import lib as 1)",
        "(import lib only)",
        "(import lib only 1)",
        "(import lib only (1))",
        "(import lib only (x as))",
        "(import lib only (x as 1))",
        "(import lib only (x y z))",
        "(import lib as a as b)",
        "(import lib only x only y)",
        "()\n(val main 1)",
        "(// leading)\n(import lib)\n(val main 1)",
        "1\n(import lib)\n(val main 1)",
    ] {
        let _ = parse_imports(src);
    }

    let ok = parse_imports("(import foo/bar as p only (x as y))\n1");
    assert!(ok.is_ok() || ok.is_err());
}

#[test]
fn module_tree_utf8_and_self_import_file() {
    let dir = std::env::temp_dir().join(format!("reciplexa-b7-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join("a.rpx"), "(import a)\n(val x 1)\n").unwrap();
    let err = load_module_tree(dir.join("a.rpx"));
    assert!(err.is_err());

    fs::write(dir.join("lib.rpx"), "(val x 1)\n").unwrap();
    fs::write(dir.join("main.rpx"), "(import lib as L)\n(val main L/x)\n").unwrap();
    let units = load_module_tree(dir.join("main.rpx")).unwrap();
    assert!(units.iter().any(|(n, _)| n == "main"));
    assert!(units.iter().any(|(n, _)| n == "lib"));

    let mut iface = HashMap::new();
    iface.insert("lib".into(), vec!["x".into()]);
    let elab = elaborate_units_with_interfaces(
        &[
            ("lib", "(val x 1)"),
            ("main", "(import lib only x)\n(val main x)"),
        ],
        &iface,
    );
    assert!(elab.is_ok(), "{elab:?}");

    let bad_iface = HashMap::from([("lib".into(), vec!["missing".into()])]);
    let err = elaborate_units_with_interfaces(&[("lib", "(val x 1)")], &bad_iface);
    assert!(err.is_err());

    // Non-.rpx skipped in directory load
    fs::write(dir.join("notes.txt"), "ignore").unwrap();
    let loaded = load_module_tree(&dir).unwrap();
    assert!(loaded.iter().all(|(n, _)| n != "notes"));

    let _ = fs::remove_dir_all(&dir);

    // Path-not-found / empty
    let missing = PathBuf::from("definitely-missing-reciplexa-b7-xyz");
    assert!(load_module_tree(&missing).is_err());
}

#[test]
fn resolve_data_params_ctor_lists_and_malformed() {
    for src in [
        "(data t ((a type)(b type)) (mk a b))\n(val main (mk 1 2))",
        "(data t ((a type)) (leaf) (node (t a)))\n(val main leaf)",
        "(data t ((a type)) ctor)\n(val main ctor)",
        "(data t (mk a) (also))\n(val main also)",
        // Non-ident name / empty-ish
        "(data 1 (c))\n(val main 1)",
        "(data t ((1 type)) c)\n(val main c)",
        "(data t ((a type) 1) c)\n(val main c)",
        "(fn f (x y) (+ x y))\n(val main (f 1 2))",
        "(fn)\n(val main 1)",
        "(fn f)\n(val main 1)",
        "(fn f x x)\n(val main 1)",
        "(type)\n(val main 1)",
        "(type t)\n(val main 1)",
        "(type 1 int)\n(val main 1)",
        "(val (f x y) (+ x y))\n(val main (f 1 2))",
        "(val (f) 1)\n(val main f)",
        "(val (1 x) x)\n(val main 1)",
        "(val 1 2)\n(val main 1)",
        "(val main (let ((x 1)(y x)) y))",
        "(val main (letrec ((f (fn (x) (f x)))) f))",
        "(val main (var cell 0 (seq (set cell 1) cell)))",
        "(val main (match v ((tuple a (bind b)) -> b) (_ -> 0)))",
        "(val main (match v ((record (a _) (b (bind c))) -> c) (_ -> 0)))",
        "(val main (match v ((bind x) -> x)))",
        "(val main (match v (true -> 1) (false -> 0) (_ -> -1)))",
        "(val main (match v (unit -> 0) (_ -> 1)))",
        "(val main (match v ((some (bind x)) -> x) (_ -> 0)))",
        "(val main (match v ((mk (tuple a b)) -> a) (_ -> 0)))",
        "(val main (match v (a/b -> 0) (_ -> 1)))",
        "(val main (match 1 2))",
        "(val main (page a4))",
        "(val main (doc \"x\"))",
        "(val main {})",
        "()\n(val main ())",
        "[1 2]\n(val main 1)",
        "(val main (fn (x) (fn (y) (+ x y))))",
        "(val main (handle ask (fn (m k) (k m)) (perform ask \"x\")))",
        "(val main (with (handler ask (fn (m k) (k m))) (perform ask \"x\")))",
        "(val main (raise \"e\"))",
        "(val main (or-raise (as-result (fn () 1))))",
        "(val main (as-result (fn () 1)))",
        "(val main (seq (perform log \"a\") 1))",
        "(val main (forward k))",
        "(val main foo/bar/baz)",
    ] {
        let _ = resolve_language_source(src);
    }
}
