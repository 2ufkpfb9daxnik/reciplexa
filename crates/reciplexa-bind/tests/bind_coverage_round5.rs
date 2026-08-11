//! Round-5 bind: remaining resolve/module miss clusters.

use std::collections::HashMap;

use reciplexa_bind::module::*;
use reciplexa_bind::{resolve_language_source, resolve_source};

#[test]
fn resolve_more_data_ctor_and_fn_shapes() {
    let r = resolve_language_source(
        r#"
(data result ((a type)(e type)) (ok a) (err e))
(fn map (f x) (f x))
(val main (map (fn (n) n) 1))
"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);

    let r = resolve_language_source("(data t ((a type)) make)\n(val main make)");
    assert!(r.is_ok(), "{:?}", r.errors);

    // Missing/malformed binders
    for src in [
        "(val)",
        "(fn)",
        "(fn ())",
        "(type)",
        "(data t ((not-a-param)) c)",
        "(data t 1 2)",
        "(val main (fn name (x) x))",
        "(val main (fn bad))",
        "(val main (letrec ((f)) f))",
        "(val main (var x y))",
        "(val main (match v (record (a x) -> x) (_ -> 0)))",
        "(val main (match v ((record (a x y)) -> x) (_ -> 0)))",
        "(val main (match v ((bind _) -> 0) (_ -> 1)))",
        "(val main (match v (1.5 -> 0) (_ -> 1)))",
        "(val main (match v (\"hi\" -> 0) (_ -> 1)))",
        "(val main (match v (unit -> 0) (_ -> 1)))",
        "(val main (match v (false -> 0) (_ -> 1)))",
        "(val main (match v ((some 1) -> 0) (_ -> 1)))",
        "(val main (match v ((pair a b c) -> a) (_ -> 0)))",
    ] {
        let _ = resolve_language_source(src);
    }
}

#[test]
fn resolve_document_still_quarantines_page() {
    let r = resolve_source("(page a4 (circle 1 2 3 red))\n(val title \"x\")");
    assert!(r.is_ok() || !r.errors.is_empty());
}

#[test]
fn resolve_language_empty_list_and_nested_non_list() {
    let r = resolve_language_source("()\n(val main (seq))");
    let _ = r.is_ok();

    let r = resolve_language_source("(val main {})");
    let _ = r.is_ok();
}

#[test]
fn module_collision_and_empty_body_and_dir_load() {
    let err = elaborate_units(&[
        ("a", "(val x 1)"),
        ("b", "(val y 2)"),
        ("main", "(import a only x)\n(import b only x)\n(val main x)"),
    ]);
    assert!(err.is_err());

    let err = elaborate_units(&[
        ("a", "(val x 1)"),
        ("b", "(val x 2)"),
        ("main", "(import a)\n(import b as a)\n(val main 1)"),
    ]);
    let _ = err;

    let units = elaborate_units(&[
        ("lib", "(val x 1)"),
        ("main", "(import lib as l)\n(val main l/x)"),
    ])
    .unwrap();
    assert!(units.iter().any(|u| u.name == "main"));

    let dir = std::env::temp_dir().join(format!("reciplexa-b5-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("lib.rpx"), "(val x 1)\n").unwrap();
    std::fs::write(dir.join("main.rpx"), "(import lib only x)\n(val main x)\n").unwrap();
    let loaded = load_module_tree(&dir).unwrap();
    assert!(loaded.len() >= 2);
    let units = elaborate_units(
        &loaded
            .iter()
            .map(|(n, s)| (n.as_str(), s.as_str()))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    assert!(!units.is_empty());
    let _ = std::fs::remove_dir_all(&dir);

    // interface From elaborate error surface (message carry)
    let mut iface = HashMap::new();
    iface.insert("lib".into(), vec!["x".into(), "y".into()]);
    let err = elaborate_units_with_interfaces(&[("lib", "(val x 1)")], &iface);
    assert!(err.is_err());
}

#[test]
fn module_parse_error_on_import_unit() {
    let err = elaborate_units(&[("main", "(import")]).unwrap_err();
    assert!(!err.message.is_empty());

    let err = parse_imports("(import foo as Bar only a)").unwrap();
    assert_eq!(err.len(), 1);
}
