//! Round-11 bind: resolve import/elaborate residual soft paths.

use std::fs;

use reciplexa_bind::module::*;
use reciplexa_bind::resolve_language_source;

#[test]
fn resolve_round11_malformed_and_comment_skips() {
    for src in [
        "(; only comment)\n(val main 1)",
        "// line comment\n(val main 1)",
        "(import graphics/shapes)\n(val main 1)",
        "(import graphics/shapes only circle)\n(val main 1)",
        "(import graphics/shapes as g)\n(val main g/circle)",
        "(import graphics/shapes only circle as c)\n(val main (c 1 2 3))",
        "(import graphics/shapes as g only circle)\n(val main (g/circle 1 2 3))",
        "(import graphics/shapes as g only circle as c)\n(val main (c 1 2 3))",
        "(import Bad)\n(val main 1)",
        "(import graphics//shapes)\n(val main 1)",
        "(import -bad)\n(val main 1)",
        "(import)\n(val main 1)",
        "(import 1)\n(val main 1)",
        "(import graphics/shapes as)\n(val main 1)",
        "(import graphics/shapes only)\n(val main 1)",
        "(import graphics/shapes as g as h)\n(val main 1)",
        "(import graphics/shapes only circle only rect)\n(val main 1)",
        "(import graphics/shapes only (circle))\n(val main 1)",
        "(val main (foo/bar unknown))",
        "(val main (match 1 (1__2 -> 0) (_ -> 1)))",
        "(val main (match 1 (\"\\q\" -> 0) (_ -> 1)))",
    ] {
        let _ = parse_imports(src);
        let _ = resolve_language_source(src);
    }
}

#[test]
fn resolve_round11_temp_tree_imports() {
    let dir = std::env::temp_dir().join("rpx_bind_cov11");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let lib = dir.join("lib.rpx");
    fs::write(&lib, "(val circle 1)\n(val rect 2)").unwrap();
    let main = dir.join("main.rpx");
    fs::write(&main, "(import lib only circle)\n(val main circle)").unwrap();
    let _ = resolve_language_source(&fs::read_to_string(&main).unwrap());
}

#[test]
fn elaborate_units_round11_collision_matrix() {
    let err = elaborate_units(&[
        ("lib", "(val x 1)"),
        ("other", "(val x 2)"),
        (
            "main",
            "(import lib only x)\n(import other only x)\n(val main x)",
        ),
    ]);
    assert!(err.is_err());

    for units in [
        vec![
            ("lib", "(val x 1)\n(val y 2)"),
            (
                "main",
                "(import lib only x)\n(import lib only y)\n(val main (+ x y))",
            ),
        ],
        vec![
            ("lib", "(val x 1)"),
            (
                "main",
                "(import lib as a)\n(import lib as b)\n(val main a/x)",
            ),
        ],
    ] {
        assert!(elaborate_units(&units).is_ok());
    }
}
