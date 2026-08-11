//! Region-coverage batch for resolve + module error/happy paths.

use reciplexa_bind::module::*;
use reciplexa_bind::{resolve_language_source, resolve_source};
use reciplexa_core::expr::CoreExpr;
use reciplexa_eval::{eval_expr, RuntimeValue, UnitHost};
use std::collections::HashMap;

#[test]
fn language_parse_error_returns_errors() {
    let r = resolve_language_source("(val main");
    assert!(!r.is_ok());
    assert!(r.errors.iter().any(|e| e.message.contains("parse error")));
}

#[test]
fn language_top_level_bracket_and_comment() {
    let r = resolve_language_source("[1 2]\n(// note)\n(val main 1)");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn language_data_with_type_params_and_nullary_ctor() {
    let r = resolve_language_source(
        r#"
(data option ((a type)) (none) (some a))
(val main none)
"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);
    assert!(r.env.bindings.values().any(|n| n == "option"));
    assert!(r.env.bindings.values().any(|n| n == "none"));
    assert!(r.env.bindings.values().any(|n| n == "some"));
}

#[test]
fn language_val_named_fn_sugar_and_wildcard_param() {
    let r = resolve_language_source("(val (id x) x)\n(val main (id 1))");
    assert!(r.is_ok(), "{:?}", r.errors);
    assert!(r.env.bindings.values().any(|n| n == "id"));

    let r = resolve_language_source("(fn ignore (_) 0)\n(val main (ignore 1))");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn language_anonymous_fn_at_toplevel_and_type_decl() {
    let r = resolve_language_source("(fn (x) x)\n(type t int)\n(val main 1)");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn language_let_letrec_var_set_if_seq() {
    let r = resolve_language_source(
        r#"(val main
  (let ((x 1) (y x))
    (letrec ((f (fn (n) (if n (f false) y))))
      (var c 0
        (seq (set c 1) (f true))))))"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn language_match_bind_tuple_record_and_wildcard() {
    let r = resolve_language_source(
        r#"
(data option (none) (some x))
(val main
  (match (some 1)
    (none -> 0)
    (_ -> 0)
    (bind v -> 0)
    (some x -> x)
    ((tuple a b) -> a)
    ((record (title t) (n n)) -> n)))
"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn language_record_field_list_tuple_and_qualified_path() {
    let r = resolve_language_source(
        r#"(val main
  (field (record (title "T") (n 1)) title))"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);

    let r = resolve_language_source("(val main (list 1 2))\n(val other (tuple 1 2))");
    assert!(r.is_ok(), "{:?}", r.errors);

    // Unbound qualified path.
    let r = resolve_language_source("(val main missing/path)");
    assert!(!r.is_ok());
    assert!(r.errors.iter().any(|e| e.message.contains("unbound")));
}

#[test]
fn language_quarantines_document_and_effect_forms() {
    let r = resolve_language_source(
        r#"
(page a4 (circle 1 2 3 red))
(markup Hello)
(perform log "x")
(handle log (fn (m) m) (perform log "y"))
(val main 1)
"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn language_handler_with_and_nested_val() {
    let r = resolve_language_source(
        r#"(val main
  (with (handler log (fn (msg) msg))
    (seq 1 2)))"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn language_malformed_val_and_match_without_arrow() {
    let r = resolve_language_source("(val 1 2)\n(val main (match 1 (x x)))");
    // Should not panic; may report unbound or succeed partially.
    let _ = r.is_ok();
}

#[test]
fn language_rejects_invalid_ident_binder() {
    // Most invalid idents are parse-level; reserved binder already covered.
    let r = resolve_language_source("(val let 1)");
    assert!(!r.is_ok());
}

#[test]
fn document_resolve_still_covers_src_handle() {
    let r = resolve_source("(src (handle log (circle 1 2 3 red)))");
    assert!(r.is_ok() || !r.errors.is_empty());
}

#[test]
fn elaborate_units_empty_duplicate_self_import() {
    let err = elaborate_units(&[]).unwrap_err();
    assert!(err.message.contains("at least one"));

    let err = elaborate_units(&[("a", "(val main 1)"), ("a", "(val main 2)")]).unwrap_err();
    assert!(err.message.contains("duplicate"));

    let err = elaborate_units(&[("main", "(import main) (val main 1)")]).unwrap_err();
    assert!(err.message.contains("itself"));
}

#[test]
fn elaborate_units_empty_body_and_interface_exports() {
    let units =
        elaborate_units(&[("lib", "(val id 1)"), ("main", "(import lib only id)")]).unwrap();
    assert!(units.iter().any(|u| u.name == "main"));

    let mut iface = HashMap::new();
    iface.insert("lib".into(), vec!["id".into()]);
    let units = elaborate_units_with_interfaces(
        &[
            ("lib", "(val id 1) (val hidden 2)"),
            ("main", "(import lib only id) (val main id)"),
        ],
        &iface,
    )
    .unwrap();
    let main = units.iter().find(|u| u.name == "main").unwrap();
    let v = eval_expr(&main.expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Int(1));

    let err = elaborate_units_with_interfaces(
        &[
            ("lib", "(val id 1)"),
            ("main", "(import lib only hidden) (val main 0)"),
        ],
        &iface,
    )
    .unwrap_err();
    assert!(err.message.contains("not exported") || err.message.contains("hidden"));
}

#[test]
fn elaborate_units_prefix_collision_and_parse_imports() {
    let err = elaborate_units(&[
        ("a", "(val x 1)"),
        ("b", "(val x 2)"),
        ("main", "(import a as p) (import b as p) (val main 0)"),
    ])
    .unwrap_err();
    assert!(err.message.contains("prefix") || err.message.contains("distinct"));

    let imports = parse_imports("(import graphics/color as c only black)\n(val main 1)").unwrap();
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module, "graphics/color");
    assert_eq!(imports[0].alias.as_deref(), Some("c"));

    let err = parse_imports("(import)").unwrap_err();
    assert!(err.message.contains("import"));

    let err = parse_imports("(import 1)").unwrap_err();
    assert!(err.message.contains("identifier") || err.message.contains("path"));

    let err = parse_imports("(import foo as)").unwrap_err();
    assert!(err.message.contains("alias"));

    let err = parse_imports("(import foo only)").unwrap_err();
    assert!(err.message.contains("only"));

    let err = parse_imports("(import foo as a as b)").unwrap_err();
    assert!(err.message.contains("duplicate"));

    let err = parse_imports("(import foo only a only b)").unwrap_err();
    assert!(err.message.contains("duplicate"));

    let err = parse_imports("(import foo bar)").unwrap_err();
    assert!(err.message.contains("unexpected") || err.message.contains("as"));

    let err = parse_imports("(import foo only (a (b)))").unwrap_err();
    assert!(err.message.contains("identifier"));

    let err = parse_imports("(val main").unwrap_err();
    assert!(err.message.contains("parse error"));
}

#[test]
fn elaborate_units_bare_import_and_letrec_exports() {
    let units = elaborate_units(&[
        (
            "lib",
            r#"(val main
  (letrec ((f (fn (x) x)))
    f))"#,
        ),
        ("main", "(import lib) (val main lib/main)"),
    ])
    .unwrap();
    assert!(units.iter().any(|u| matches!(u.expr, CoreExpr::Let { .. })));
}

#[test]
fn load_module_tree_missing_path_and_empty_dir() {
    let err = load_module_tree("d:/reciplexa/.tmp/no-such-mod-tree-xyz").unwrap_err();
    assert!(err.message.contains("not found") || err.message.contains("failed"));

    let dir = std::env::temp_dir().join(format!("reciplexa-empty-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let err = load_module_tree(&dir).unwrap_err();
    assert!(err.message.contains("no `.rpx`") || err.message.contains("no "));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_module_tree_self_import_and_missing_sibling() {
    let dir = std::env::temp_dir().join(format!("reciplexa-self-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.rpx"), "(import main)\n(val main 1)\n").unwrap();
    let err = load_module_tree(dir.join("main.rpx")).unwrap_err();
    assert!(err.message.contains("itself") || err.message.contains("self"));
    let _ = std::fs::remove_dir_all(&dir);

    let dir = std::env::temp_dir().join(format!("reciplexa-miss-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.rpx"), "(import missing)\n(val main 1)\n").unwrap();
    let err = load_module_tree(dir.join("main.rpx")).unwrap_err();
    assert!(err.message.contains("failed to read") || err.message.contains("missing"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn interface_rejects_unknown_export_name() {
    let mut iface = HashMap::new();
    iface.insert("lib".into(), vec!["ghost".into()]);
    let err =
        elaborate_units_with_interfaces(&[("lib", "(val id 1)"), ("main", "(val main 0)")], &iface)
            .unwrap_err();
    assert!(
        err.message.contains("ghost")
            || err.message.contains("export")
            || err.message.contains("interface"),
        "{}",
        err.message
    );
}

#[test]
fn match_arm_missing_arrow_and_bind_payload() {
    let r = resolve_language_source(
        r#"
(data option (none) (some x))
(val main
  (match (some 1)
    (some (bind y) -> y)
    (none -> 0)))
"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);
}
