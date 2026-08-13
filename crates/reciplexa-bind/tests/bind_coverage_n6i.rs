//! N6 tip i: bind resolve.rs soft continues + module parse_import edges.

use std::fs;

use reciplexa_bind::module::{
    elaborate_module_tree, elaborate_units, load_module_tree, parse_imports,
};
use reciplexa_bind::{resolve_language_source, resolve_source};

#[test]
fn resolve_n6i_malformed_let_match_data_paths() {
    for src in [
        // Empty match arm → continue
        "(val main (match 1 () (_ -> 1)))",
        "(val main (match 1 () () (_ -> 0)))",
        // Match arm not a Node
        "(val main (match 1 1 (_ -> 0)))",
        // Let: bindings Node but not List (BracketList)
        "(val main (let [x 1] x))",
        "(val main (letrec [f (fn (x) x)] (f 1)))",
        // Let: pair not List / short / non-token name
        "(val main (let ((x 1) [y 2]) x))",
        "(val main (let ((x)) x))",
        "(val main (let (((x) 1)) x))",
        "(val main (let (x 1) x))",
        "(val main (let (1) x))",
        // Let with no bindings node (Token head)
        "(val main (let x 1))",
        // Data type-param section skip + ctor declare
        "(data box ((a type)) (mk a))\n(val main (mk 1))",
        "(data box ((a type)(b type)) (pair a b))\n(val main 1)",
        // Named val sugar + BracketList params (is_param_list)
        "(val (f x) x)\n(val main (f 1))",
        "(val (f [x y]) (+ x y))\n(val main (f 1 2))",
        "(fn g [a] a)\n(val main (g 1))",
        // Nested Node params (non-Ident token skip in param walk)
        "(fn h ((x)) x)\n(val main 1)",
        "(val main (fn ((x)) x))",
        // Pattern: bind / record non-List pair / nested list / literals
        "(val main (match 1 ((bind x) -> x)))",
        "(val main (match 1 ((bind 1) -> 0) (_ -> 1)))",
        "(val main (match 1 ((record a) -> 0) (_ -> 1)))",
        "(val main (match 1 ((record (a)) -> 0) (_ -> 1)))",
        "(val main (match 1 ((tuple _) -> 0) (_ -> 1)))",
        "(val main (match 1 ((tuple true false unit) -> 0) (_ -> 1)))",
        // Invalid binders → validate_ident Err
        "(val Bad 1)",
        "(val report_title 1)",
        "(fn Bad (x) x)\n(val main 1)",
        "(data Bad (c))\n(val main 1)",
        // Path validate Err + unbound path
        "(val main Foo/bar)",
        "(val main a_b/c)",
        "(val main missing/name)",
        // Structured comment skipped inside lists
        "(val main (seq 1 (// skip) 2))",
        "(val main (list 1 (// c) 2))",
        "(// top)\n(val main 1)",
        // Var / set / local edges
        "(val main (local (var x 1) (set x 2) x))",
        "(val main (var x 1 x))",
        // Wildcard use-site early return
        "(val main (fn (_) _))",
        "(val main (match 1 (_ -> 0)))",
    ] {
        let _ = resolve_language_source(src);
    }
}

#[test]
fn resolve_n6i_source_handle_and_named_binding() {
    for src in [
        "(src (handle log (circle 1 2 3 red)))",
        "(src (handle ask (perform ask 1)))",
        "(page a4 (circle 1 2 3 red))",
        "(type title str)\n(val title \"Hi\")\n(page a4)",
        "(val _ 1)\n(page a4)",
        "(type _ int)\n(page a4)",
        // Non-ident after head breaks declare_named_binding
        "(val 1 2)\n(page a4)",
        "(type 1 int)\n(page a4)",
    ] {
        let _ = resolve_source(src);
    }
}

#[test]
fn module_n6i_import_parse_and_tree_edges() {
    for src in [
        // Non-ident list head → parse_import_list Ok(None)
        "(1 2 3)\n(val main 1)",
        "(\"x\")\n(val main 1)",
        // Nested structured comment inside import list
        "(import lib (// skip-me) only x)\n(val main x)",
        "(import lib only x (// trail))\n(val main x)",
        // Legacy only-list + rename matrix
        "(import lib only (a b))\n(val main a)",
        "(import lib only a as aa)\n(val main aa)",
        "(import lib as m only a)\n(val main m/a)",
        // Bad import shapes
        "(import)\n(val main 1)",
        "(import 1)\n(val main 1)",
        "(import lib only)\n(val main 1)",
        "(import lib as)\n(val main 1)",
        "(import lib only (a (b)))\n(val main 1)",
    ] {
        let _ = parse_imports(src);
    }

    let dir = std::env::temp_dir().join("rpx_bind_n6i_mod");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("lib.rpx"), "(val a 1)\n(val b 2)\n").unwrap();
    fs::write(
        dir.join("main.rpx"),
        "(import lib only a as aa)\n(import lib)\n(val main (+ aa b))\n",
    )
    .unwrap();
    fs::write(dir.join("skip.txt"), "nope").unwrap();
    // Nested structured comment in loaded unit
    fs::write(
        dir.join("cmt.rpx"),
        "(// file comment)\n(import lib only a)\n(val main a)\n",
    )
    .unwrap();

    let _ = load_module_tree(&dir);
    let _ = load_module_tree(dir.join("main.rpx"));
    let _ = load_module_tree(dir.join("cmt.rpx"));
    let _ = elaborate_module_tree(dir.join("main.rpx"));
    let _ = elaborate_module_tree(&dir);

    let _ = elaborate_units(&[
        ("lib", "(val a 1)\n(val b 2)"),
        (
            "main",
            "(import lib only a as aa)\n(import lib only b)\n(val main (+ aa b))",
        ),
    ]);
    // Missing export from only
    let _ = elaborate_units(&[
        ("lib", "(val a 1)"),
        ("main", "(import lib only missing)\n(val main 1)"),
    ]);
}
