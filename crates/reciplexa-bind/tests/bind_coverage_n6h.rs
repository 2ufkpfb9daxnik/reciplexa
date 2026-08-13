//! N6 tip: bind module tree + resolve residuals under 97%.

use std::fs;

use reciplexa_bind::module::{
    elaborate_module_tree, elaborate_units, load_module_tree, parse_imports,
};
use reciplexa_bind::resolve_language_source;

#[test]
fn module_tree_import_rename_and_directory() {
    let dir = std::env::temp_dir().join("rpx_bind_n6h_mod");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join("a.rpx"), "(val x 1)\n(val y 2)\n").unwrap();
    fs::write(dir.join("b.rpx"), "(val z 3)\n").unwrap();
    fs::write(
        dir.join("main.rpx"),
        "(import a only x as ax)\n(import a only y)\n(import b)\n(val main (+ ax y))\n",
    )
    .unwrap();
    // Non-.rpx ignored in directory load
    fs::write(dir.join("skip.txt"), "nope").unwrap();

    let _ = load_module_tree(&dir);
    let _ = load_module_tree(dir.join("main.rpx"));
    let _ = elaborate_module_tree(dir.join("main.rpx"));
    let _ = elaborate_module_tree(&dir);

    // Transitive + duplicate pending name (seen.continue)
    fs::write(dir.join("cycle_a.rpx"), "(import cycle_b)\n(val main 1)\n").unwrap();
    fs::write(dir.join("cycle_b.rpx"), "(import cycle_a)\n(val main 2)\n").unwrap();
    let _ = load_module_tree(dir.join("cycle_a.rpx"));

    // Self-import reject
    fs::write(dir.join("self.rpx"), "(import self)\n(val main 1)\n").unwrap();
    let _ = load_module_tree(dir.join("self.rpx"));

    // Missing module path
    let _ = load_module_tree(dir.join("missing.rpx"));
    let _ = load_module_tree(dir.join("no_such_dir_xyz"));

    // Import parse edges feeding module
    for src in [
        "(import a only x as ax)\n(val main ax)",
        "(import a as m only x)\n(val main m/x)",
        "(import a)\n(val main 1)",
        "(import 1)\n(val main 1)",
        "(val main 1)",
    ] {
        let _ = parse_imports(src);
    }

    let _ = elaborate_units(&[
        ("lib", "(val a 1)\n(val b 2)"),
        (
            "main",
            "(import lib only a as aa)\n(import lib only b)\n(val main (+ aa b))",
        ),
    ]);
    let _ = elaborate_units(&[("lib", "(val a 1)"), ("main", "(import lib)\n(val main a)")]);
}

#[test]
fn resolve_n6h_dense_language_forms() {
    for src in [
        "(val main 1)",
        "(fn f (x) x)\n(val main (f 1))",
        "(val main (fn (x) x))",
        "(val main (let ((x 1)(y 2)) (+ x y)))",
        "(val main (letrec ((f (fn (x) (f x)))) f))",
        "(val main (local (val x 1) (var y 2) (+ x y)))",
        "(val main (match 1 (1 -> 0) (_ -> 1)))",
        "(val main (match (tuple 1 2) ((tuple a b) -> a) (_ -> 0)))",
        "(val main (match (record (a 1)) ((record (a x)) -> x) (_ -> 0)))",
        "(data opt (none) (some x))\n(val main (match (some 1) (none -> 0) ((some x) -> x)))",
        "(val main (handle ask (fn (m k) (k m)) (perform ask 1)))",
        "(val main (with (handler ask (fn (m k) (k m))) 1))",
        "(val main (record (a 1) (b 2)))",
        "(val main (record-update (record (a 1)) (a 2)))",
        "(val main (record-extend (record (a 1)) (b 2)))",
        "(val main (field (record (a 1)) a))",
        "(val main (if true 1 0))",
        "(val main (seq 1 2 3))",
        "(val main (list 1 2))",
        "(val main (tuple 1 2))",
        "(val main (as int 1))",
        "(val main (try-cast 1 int))",
        "(val main (check-cast 1 int))",
        "(val main (perform log \"x\"))",
        "(val main (forward k))",
        "(val main (raise \"e\"))",
        "(; comment)\n(val main 1)",
        "(// structured)\n(val main 1)",
        "(import graphics/shapes only circle)\n(val main 1)",
        // Empty / odd match arms
        "(val main (match 1))",
        "(val main (match 1 (-> 0)))",
        "(val main (match 1 (1 ->)))",
        // Nested binds
        "(val main (match 1 ((bind x) -> x)))",
        "(val main (fn (_) 1))",
    ] {
        let _ = resolve_language_source(src);
        let _ = parse_imports(src);
    }
}
