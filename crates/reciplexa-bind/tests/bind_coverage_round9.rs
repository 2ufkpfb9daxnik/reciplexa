//! Round-9 bind: load_module_tree IO edges + interface link residuals.

use std::collections::HashMap;
use std::fs;

use reciplexa_bind::module::*;
use reciplexa_bind::resolve_language_source;

#[test]
fn load_module_tree_path_not_found_and_empty_dir() {
    let missing = std::env::temp_dir().join(format!(
        "reciplexa-b9-missing-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let err = load_module_tree(&missing).unwrap_err();
    assert!(err.message.contains("not found") || err.message.contains("failed"));

    let dir = std::env::temp_dir().join(format!("reciplexa-b9-empty-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let err = load_module_tree(&dir).unwrap_err();
    assert!(err.message.contains("no `.rpx`") || err.message.contains("failed"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn load_module_tree_skips_non_rpx_and_self_import() {
    let dir = std::env::temp_dir().join(format!("reciplexa-b9-mix-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("ok.rpx"), "(val x 1)\n").unwrap();
    fs::write(dir.join("notes.txt"), "not a module\n").unwrap();
    fs::write(dir.join("locked.rpx"), "(import locked)\n(val main 1)\n").unwrap();
    let units = load_module_tree(&dir).unwrap();
    assert!(units.iter().any(|(n, _)| n == "ok"));
    let err = load_module_tree(dir.join("locked.rpx")).unwrap_err();
    assert!(err.message.contains("cannot import itself"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn elaborate_units_with_interfaces_and_alias_only() {
    let mut ifaces = HashMap::new();
    ifaces.insert("lib".into(), vec!["x".into()]);
    let ok = elaborate_units_with_interfaces(
        &[
            ("lib", "(val x 1)\n(val hidden 2)"),
            ("main", "(import lib only x)\n(val main x)"),
        ],
        &ifaces,
    );
    assert!(ok.is_ok());

    let err = elaborate_units_with_interfaces(
        &[
            ("lib", "(val x 1)"),
            ("main", "(import lib only missing)\n(val main 1)"),
        ],
        &ifaces,
    );
    assert!(err.is_err());

    let err = elaborate_units_with_interfaces(&[("lib", "(val x 1)")], &{
        let mut m = HashMap::new();
        m.insert("lib".into(), vec!["ghost".into()]);
        m
    });
    assert!(err.is_err());

    let ok = elaborate_units(&[
        ("lib", "(val x 1)\n(val y 2)"),
        ("main", "(import lib as g)\n(val main g/x)"),
    ]);
    assert!(ok.is_ok(), "alias import: {ok:?}");

    let ok = elaborate_units(&[
        ("lib", "(val x 1)"),
        ("main", "(import lib)\n(val main lib/x)"),
    ]);
    assert!(ok.is_ok(), "bare import: {ok:?}");
}

#[test]
fn resolve_round9_more_outliers() {
    for src in [
        "(data t ((a type)) ((mk a)) )\n(val main 1)",
        "(data t (mk a b c))\n(val main (mk 1 2 3))",
        "(fn)\n(val main 1)",
        "(fn 1 (x) x)\n(val main 1)",
        "(type)\n(val main 1)",
        "(type 1)\n(val main 1)",
        "(val)\n(val main 1)",
        "(val 1 2)\n(val main 1)",
        "(val (1 x) x)\n(val main 1)",
        "(val main (match v ((tuple a) -> a) (_ -> 0)))",
        "(val main (match v ((record (a x) (b y)) -> x) (_ -> 0)))",
        "(val main (match v ((bind x) -> x)))",
        "(val main (letrec () 1))",
        "(val main (let () 1))",
        "(val main (local () 1))",
        "// line comment\n(val main 1)",
        "(; structured ;)\n(val main 1)",
        "(val main a/b)",
        "(import missing)\n(val main 1)",
    ] {
        let _ = resolve_language_source(src);
        let _ = parse_imports(src);
    }
}
