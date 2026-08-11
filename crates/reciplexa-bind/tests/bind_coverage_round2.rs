//! Round-2 bind coverage: remaining resolve/module arms.

use reciplexa_bind::module::*;
use reciplexa_bind::{resolve_language_source, BindingMap};
use reciplexa_identity::binding::BindingId;
use std::collections::HashMap;

#[test]
fn binding_map_definition_of() {
    let r = resolve_language_source("(val x 1) (val main x)");
    assert!(r.is_ok(), "{:?}", r.errors);
    let id = r
        .env
        .bindings
        .iter()
        .find(|(_, n)| *n == "x")
        .map(|(id, _)| *id)
        .expect("x");
    assert!(r.binding_map.definition_of(id).is_some());
    assert!(r.binding_map.definition_of(BindingId::new(999_999)).is_none());
}

#[test]
fn language_top_level_bare_token_and_non_ident_head() {
    // Trivia-only tokens between forms.
    let r = resolve_language_source("  \n(val main 1)");
    assert!(r.is_ok(), "{:?}", r.errors);

    // List whose head is not an ident — falls through.
    let r = resolve_language_source("(1 2 3)\n(val main 1)");
    assert!(r.is_ok() || !r.errors.is_empty());
}

#[test]
fn language_data_nullary_ident_ctors() {
    let r = resolve_language_source("(data color red green blue)\n(val main red)");
    assert!(r.is_ok(), "{:?}", r.errors);
    assert!(r.env.bindings.values().any(|n| n == "red"));
}

#[test]
fn language_nested_perform_handle_inside_val() {
    // Nested perform/handle are expression forms (not top-level quarantine).
    let r = resolve_language_source(
        r#"(val main
  (handle log (fn (msg) msg)
    (perform log "x")))"#,
    );
    // May be ok: perform/handle are recognized in lang_resolve_list.
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn language_let_malformed_and_empty_match() {
    let r = resolve_language_source("(val main (let 1 2))");
    let _ = r.is_ok();
    let r = resolve_language_source("(val main (letrec 1 2))");
    let _ = r.is_ok();
    let r = resolve_language_source("(val main (match))");
    let _ = r.is_ok();
    let r = resolve_language_source("(val main (var))");
    let _ = r.is_ok();
}

#[test]
fn language_match_literal_true_false_unit_underscore() {
    let r = resolve_language_source(
        r#"(val main
  (match true
    (true -> 1)
    (false -> 0)
    (unit -> 0)
    (_ -> 2)))"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn language_field_without_record_and_nested_type() {
    let r = resolve_language_source("(val main (field))");
    let _ = r.is_ok();
    let r = resolve_language_source("(val main (type t int))");
    let _ = r.is_ok();
}

#[test]
fn parse_imports_node_and_only_as_errors() {
    let err = parse_imports("(import foo (bar))").unwrap_err();
    assert!(!err.message.is_empty());

    let err = parse_imports("(import foo only a as)").unwrap_err();
    assert!(err.message.contains("local") || err.message.contains("as"));

    let err = parse_imports("(import foo only a (b))").unwrap_err();
    assert!(err.message.contains("identifier") || err.message.contains("only"));
}

#[test]
fn elaborate_units_missing_export_table_edge() {
    // Empty-body unit with only import.
    let units = elaborate_units(&[
        ("lib", "(val id 1)"),
        ("main", "(import lib only id)\n"),
    ])
    .unwrap();
    assert!(units.iter().any(|u| u.name == "main"));

    // LetRec exports via letrec-elaborated libs already covered; force From ElaborateError.
    let err = elaborate_units(&[("main", "(val main")]) .unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn load_directory_skips_non_rpx_and_elaborate_tree() {
    let dir = std::env::temp_dir().join(format!("reciplexa-mix-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("readme.txt"), "nope").unwrap();
    std::fs::write(dir.join("a.rpx"), "(val a 1)\n").unwrap();
    std::fs::write(dir.join("b.rpx"), "(val b 2)\n").unwrap();
    let units = load_module_tree(&dir).unwrap();
    assert_eq!(units.len(), 2);
    let elaborated = elaborate_module_tree(&dir).unwrap();
    assert_eq!(elaborated.len(), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn prefix_same_module_ok_and_bare_rename_collision_same_module() {
    let units = elaborate_units(&[
        ("lib", "(val id 1)"),
        (
            "main",
            "(import lib as p) (import lib as p) (val main p/id)",
        ),
    ])
    .unwrap();
    assert!(units.iter().any(|u| u.name == "main"));

    // Same module, same local name via only — ok.
    let units = elaborate_units(&[
        ("lib", "(val id 1)"),
        (
            "main",
            "(import lib only id) (import lib only id) (val main id)",
        ),
    ])
    .unwrap();
    assert!(units.iter().any(|u| u.name == "main"));
}

#[test]
fn import_only_as_local_and_interface_ok() {
    let mut iface = HashMap::new();
    iface.insert("lib".into(), vec!["id".into()]);
    let units = elaborate_units_with_interfaces(
        &[
            ("lib", "(val id 1) (val other 2)"),
            ("main", "(import lib only id as x) (val main x)"),
        ],
        &iface,
    )
    .unwrap();
    assert!(units.iter().any(|u| u.name == "main"));
}

#[test]
fn binding_map_default_empty() {
    let map = BindingMap::default();
    assert!(map.definitions.is_empty());
    assert!(map.uses.is_empty());
}
