//! Round-8 bind: tip remaining resolve/module load edges.

use std::fs;

use reciplexa_bind::module::*;
use reciplexa_bind::resolve_language_source;

#[test]
fn resolve_force_non_list_children_and_var_edges() {
    for src in [
        // Non-list expression node walk (brace group)
        "{}\n(val main 1)",
        "(val main {x})",
        // var / letrec malformed that still resolve atoms
        "(val main (var 1 2 3))",
        "(val main (var x))",
        "(val main (letrec [((f (fn (x) x)))] f))",
        "(val main (letrec ((f (fn (x) x)) 1) f))",
        "(val main (let ((1 2)) 3))",
        "(val main (match v))",
        "(val main (match v a))",
        "(val main (match v (1 -> 0) (\"x\" -> 1) (1.0 -> 2) (true -> 3) (false -> 4) (unit -> 5) (_ -> 6)))",
        "(val main (match v ((tuple) -> 0) (_ -> 1)))",
        "(val main (match v ((record) -> 0) (_ -> 1)))",
        "(val main (match v ((record a) -> 0) (_ -> 1)))",
        "(val main (match v ((bind) -> 0) (_ -> 1)))",
        "(val main (match v ((some _) -> 0) ((none) -> 1) (_ -> 2)))",
        "(val main (fn (x _) x))",
        "(val (_ x) x)\n(val main 1)",
        "(data t ((a type)) (mk (a) b))\n(val main 1)",
        "(fn f (x) x)\n(val main f)",
        // qualified path use/def
        "(val main a/b/c)",
        "(val a/b 1)\n(val main a/b)",
    ] {
        let _ = resolve_language_source(src);
    }
}

#[test]
fn module_import_validate_and_comment_token_paths() {
    for src in [
        "(import lib only (x as y) (z))\n1",
        "(import graphics/color)\n1",
        "(import lib as L only x)\n1",
        "(import lib only x as y)\n1",
        "   \n(import lib)\n(val main 1)",
        "(import lib)\n",
        "1 2 3",
        "(import)",
        "(import ())",
        "(import lib as as)",
    ] {
        let _ = parse_imports(src);
        let _ = elaborate_units(&[("lib", "(val x 1)"), ("main", src)]);
    }

    let dir = std::env::temp_dir().join(format!("reciplexa-b8-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.rpx"), "(val x 1)\n").unwrap();
    fs::write(dir.join("b.rpx"), "(import a as A)\n(val main A/x)\n").unwrap();
    let _ = elaborate_module_tree(dir.join("b.rpx"));
    let _ = load_module_tree(&dir);
    let _ = fs::remove_dir_all(&dir);
}
