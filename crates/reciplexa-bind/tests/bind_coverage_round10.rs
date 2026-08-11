//! Round-10 bind: import split residuals + resolve soft-path asserts.

use std::fs;

use reciplexa_bind::module::*;
use reciplexa_bind::resolve_language_source;

#[test]
fn parse_imports_alias_only_and_invalid_paths() {
    for src in [
        "(import graphics/shapes as g only circle)\n(val main (g/circle 1 2 3))",
        "(import graphics/shapes only circle as c)\n(val main (c 1 2 3))",
        "(import graphics/shapes as g only circle as c)\n(val main (c 1 2 3))",
        "(import graphics/shapes)\n(val main graphics/shapes/circle)",
        "(; skip ;)\n(import graphics/shapes only circle)\n(val main 1)",
        "// line\n(import graphics/shapes)\n(val main 1)",
        "(import)\n(val main 1)",
        "(import 1)\n(val main 1)",
        "(import graphics/shapes as)\n(val main 1)",
        "(import graphics/shapes only)\n(val main 1)",
        "(import graphics/shapes as g as h)\n(val main 1)",
        "(import graphics/shapes only circle only rect)\n(val main 1)",
        "(import BadPath)\n(val main 1)",
        "(import graphics//shapes)\n(val main 1)",
        "(import -bad)\n(val main 1)",
        "(import graphics/shapes only (circle))\n(val main 1)",
    ] {
        let _ = parse_imports(src);
        let _ = resolve_language_source(src);
    }
}

#[test]
fn elaborate_units_collision_and_multi_import() {
    let err = elaborate_units(&[
        ("lib", "(val x 1)"),
        ("other", "(val x 2)"),
        (
            "main",
            "(import lib only x)\n(import other only x)\n(val main x)",
        ),
    ]);
    assert!(err.is_err(), "bare name collision should fail: {err:?}");

    let ok = elaborate_units(&[
        ("lib", "(val x 1)\n(val y 2)"),
        (
            "main",
            "(import lib only x)\n(import lib only y)\n(val main (+ x y))",
        ),
    ]);
    assert!(ok.is_ok(), "same module twice ok: {ok:?}");

    let ok = elaborate_units(&[
        ("lib", "(val x 1)"),
        (
            "main",
            "(import lib as a)\n(import lib as b)\n(val main a/x)",
        ),
    ]);
    assert!(ok.is_ok(), "dual alias ok: {ok:?}");

    let err = elaborate_units(&[
        ("lib", "(val x 1)"),
        ("other", "(val y 2)"),
        (
            "main",
            "(import lib as g)\n(import other as g)\n(val main 1)",
        ),
    ]);
    assert!(err.is_err(), "alias collision: {err:?}");
}

#[test]
fn load_module_tree_utf8_and_nested_imports() {
    let dir = std::env::temp_dir().join(format!("reciplexa-b10-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.rpx"), "(import b)\n(val main b/x)\n").unwrap();
    fs::write(dir.join("b.rpx"), "(val x 1)\n").unwrap();
    fs::write(dir.join("readme.md"), "skip\n").unwrap();
    let units = load_module_tree(dir.join("a.rpx")).unwrap();
    assert_eq!(units[0].0, "a");
    assert!(units.iter().any(|(n, _)| n == "b"));
    let elab = elaborate_module_tree(dir.join("a.rpx")).unwrap();
    assert!(elab.iter().any(|u| u.name == "a"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn resolve_soft_skips_and_pattern_shapes() {
    for src in [
        "(data t ((a type)) (mk a))\n(val main (mk 1))",
        "(data t (1))\n(val main 1)",
        "(data t ((mk)))\n(val main 1)",
        "(data t (mk (1)))\n(val main 1)",
        "(val (f) 1)\n(val main (f))",
        "(val (f x y) (+ x y))\n(val main (f 1 2))",
        "(fn f (x) x)\n(val main (f 1))",
        "(fn f x x)\n(val main 1)",
        "(type t int)\n(val main 1)",
        "(type t (fn int int))\n(val main 1)",
        "(val main (let ((x 1) (y 2)) (+ x y)))",
        "(val main (letrec ((f (fn (n) (if (= n 0) 0 (f (- n 1))))) (f 1)))",
        "(val main (letrec (bad) 1))",
        "(val main (letrec ((1 2)) 1))",
        "(val main (letrec (((f))) 1))",
        "(val main (match 1 (1 -> 1) (_ -> 0)))",
        "(val main (match v ((cons h t) -> h) (_ -> 0)))",
        "(val main (match v ((some x) -> x) ((none) -> 0)))",
        "(val main (match v ((record (a x)) -> x)))",
        "(val main (match v ((tuple a b) -> a)))",
        "(val main (match v ((bind x) -> x)))",
        "(val main (match v (_ -> 0) (1 -> 1)))",
        "(val main (var 1 0 1))",
        "(val main (var x))\n",
        "(; structured comment with (inner form) ;)\n(val main 1)",
        "\"string at top\"\n(val main 1)",
        "42\n(val main 1)",
        "(val main a/b/c)",
        "(val main (fn (x) (fn (y) (+ x y))))",
    ] {
        let _ = resolve_language_source(src);
    }
}
