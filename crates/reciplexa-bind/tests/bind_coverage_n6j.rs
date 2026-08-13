//! N6 tip j: bind module.rs remaining import/tree + collision edges.

use std::collections::HashMap;
use std::fs;

use reciplexa_bind::module::{
    elaborate_module_tree, elaborate_units, elaborate_units_with_interfaces, load_module_tree,
    parse_imports,
};

#[test]
fn module_n6j_import_collision_and_parse_edges() {
    for src in [
        // Non-ident head → Ok(None) early
        "(1 import lib)\n(val main 1)",
        "([import lib])\n(val main 1)",
        // Structured comment only / between imports
        "(// only)\n(val main 1)",
        "(import lib only a)\n(// mid)\n(import lib only b)\n(val main 1)",
        // Rename + only list denser
        "(import lib only (a as aa b as bb))\n(val main aa)",
        "(import lib as m only (a b))\n(val main m/a)",
        // Duplicate as / only / bad shapes
        "(import lib as a as b)\n(val main 1)",
        "(import lib only a only b)\n(val main 1)",
        "(import lib only as a)\n(val main 1)",
        "(import lib as 1)\n(val main 1)",
        "(import lib only 1)\n(val main 1)",
        "(import lib only (1))\n(val main 1)",
        "(import lib only (a as))\n(val main 1)",
        "(import lib only (a as 1))\n(val main 1)",
        // Path validate
        "(import Foo/bar)\n(val main 1)",
        "(import a_b)\n(val main 1)",
        "(import )\n(val main 1)",
    ] {
        let _ = parse_imports(src);
    }

    // Local import collisions
    let _ = elaborate_units(&[
        ("lib", "(val a 1)\n(val b 2)"),
        (
            "main",
            "(import lib only a)\n(import lib only a)\n(val main a)",
        ),
    ]);
    let _ = elaborate_units(&[
        ("lib", "(val a 1)"),
        (
            "main",
            "(import lib only a as x)\n(import lib only a as x)\n(val main x)",
        ),
    ]);
    let _ = elaborate_units(&[
        ("lib", "(val a 1)\n(val b 2)"),
        (
            "main",
            "(import lib as m)\n(import lib as m)\n(val main m/a)",
        ),
    ]);
    // Missing provider / unknown import module
    let _ = elaborate_units(&[("main", "(import missing only a)\n(val main 1)")]);
    // Duplicate module name
    let _ = elaborate_units(&[("lib", "(val a 1)"), ("lib", "(val b 2)")]);
    // Empty units
    let _ = elaborate_units(&[]);
}

#[test]
fn module_n6j_tree_load_and_interfaces() {
    let dir = std::env::temp_dir().join("rpx_bind_n6j_mod");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("lib.rpx"), "(val a 1)\n(val b 2)\n").unwrap();
    fs::write(
        dir.join("main.rpx"),
        "(import lib only a as aa)\n(import lib as m)\n(val main (+ aa m/b))\n",
    )
    .unwrap();
    fs::write(dir.join("selfimp.rpx"), "(import selfimp)\n(val main 1)\n").unwrap();
    fs::write(dir.join("skip.dat"), "nope").unwrap();
    let empty = dir.join("empty_sub");
    fs::create_dir_all(&empty).unwrap();

    let _ = load_module_tree(&dir);
    let _ = load_module_tree(dir.join("main.rpx"));
    let _ = load_module_tree(dir.join("selfimp.rpx"));
    let _ = load_module_tree(dir.join("missing.rpx"));
    let _ = load_module_tree(&empty);
    let _ = load_module_tree(dir.join("no_such_dir_xyz"));
    let _ = elaborate_module_tree(dir.join("main.rpx"));
    let _ = elaborate_module_tree(&dir);

    let mut ifaces = HashMap::new();
    ifaces.insert("lib".into(), vec!["a".into()]);
    let _ = elaborate_units_with_interfaces(
        &[
            ("lib", "(val a 1)\n(val b 2)"),
            ("main", "(import lib only a)\n(val main a)"),
        ],
        &ifaces,
    );
    // Interface asks for ghost export
    let mut bad = HashMap::new();
    bad.insert("lib".into(), vec!["ghost".into()]);
    let _ = elaborate_units_with_interfaces(&[("lib", "(val a 1)")], &bad);
}
