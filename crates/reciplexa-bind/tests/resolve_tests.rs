//! Integration tests moved from src/resolve.rs for region coverage.

use reciplexa_bind::{resolve_language_source, resolve_source};

#[test]
fn resolves_builtin_colors() {
    let r = resolve_source("(page a4 (circle 1 2 3 red))");
    assert!(r.is_ok());
}

#[test]
fn unknown_ident_is_error() {
    let r = resolve_source("(page a4 (circle 1 2 3 puce))");
    assert!(!r.is_ok());
    assert!(r.errors[0].message.contains("puce"));
}

#[test]
fn parse_error_prevents_binding() {
    let r = resolve_source("(page a4 (rect");
    assert!(!r.is_ok());
    assert!(r.errors.iter().all(|e| e.message.contains("parse error")));
    assert!(r.env.bindings.is_empty() || r.env.builtin_colors.len() >= 6);
}

#[test]
fn src_handle_scope_allows_inner_names() {
    let r = resolve_source("(src (handle inner (circle 1 2 3 red)))");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn surface_keywords_are_not_unbound_errors() {
    let r = resolve_source("(page a4 (group (translate 1 2 (rect 0 0 1 1 red))))");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn empty_source_is_ok_with_builtins() {
    let r = resolve_source("");
    assert!(r.is_ok());
    assert!(r.env.builtin_colors.contains_key("red"));
    assert!(r.env.builtin_colors.contains_key("a4"));
}

#[test]
fn all_paper_and_color_builtins_resolve() {
    for color in ["black", "white", "red", "green", "blue"] {
        let src = format!("(page a4 (circle 0 0 1 {color}))");
        assert!(resolve_source(&src).is_ok(), "{color}");
    }
    for paper in ["a4", "letter"] {
        let src = format!("(page {paper})");
        assert!(resolve_source(&src).is_ok(), "{paper}");
    }
}

#[test]
fn doc_and_src_forms_resolve() {
    assert!(resolve_source("(markup Hello)").is_ok());
    assert!(resolve_source("(src (perform log \"x\"))\n(page a4)").is_ok());
}

#[test]
fn markup_at_commands_are_not_unbound_idents() {
    assert!(resolve_source("(markup @heading(Hi))").is_ok());
    assert!(resolve_source("(page a4 (circle 1 2 3 red))").is_ok());
    let unbound = resolve_source("(page a4 (circle 1 2 3 puce))");
    assert!(!unbound.is_ok());
    assert!(unbound.errors[0].message.contains("puce"));
}

#[test]
fn structured_comment_top_level_is_skipped() {
    let r = resolve_source("(// note with weird)\n(page a4 (circle 1 2 3 red))");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn nested_unknown_ident_reports_error() {
    let r = resolve_source("(page a4 (group (circle 0 0 1 mauve)))");
    assert!(!r.is_ok());
    assert!(r.errors.iter().any(|e| e.message.contains("mauve")));
}

#[test]
fn resolve_error_carries_span() {
    let r = resolve_source("(page a4 (circle 1 2 3 puce))");
    assert!(!r.errors.is_empty());
    let range = r.errors[0].range;
    // Unbound ident should point at a non-empty half-open span.
    assert!(range.end().0 >= range.start().0);
    assert!(!range.is_empty());
}

#[test]
fn bracket_and_brace_top_level_forms() {
    // Non-list forms are skipped by resolve_form; should not panic.
    let r = resolve_source("[1 2 3]\n{a b}");
    assert!(r.is_ok() || !r.errors.is_empty());
}

#[test]
fn empty_and_headless_lists_are_skipped() {
    let r = resolve_source("()\n(123)\n(src ())\n(src [1])");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn nested_handle_lists_inside_src() {
    // First child of `handle` is a List (not a name token), so skip(1) still
    // walks subsequent lists through resolve_src_form.
    let r = resolve_source("(src (handle (page a4) (circle 1 2 3 red)))");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn list_starting_with_delimiter_has_no_head() {
    // Head scan breaks on non-trivia non-lparen before any Ident.
    let r = resolve_source("(() )");
    assert!(r.is_ok() || !r.errors.is_empty());
}
#[test]
fn resolve_expr_on_bracket_child_of_group() {
    // BracketList child exercises the non-List arm of resolve_expr.
    let r = resolve_source("(group [1 2] (circle 1 2 3 red))");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn type_and_val_declare_binding_names() {
    let r = resolve_source(
        r#"(type title str)
(val title "Hello")
(page a4 (circle 1 2 3 red))"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);
    assert!(r.env.bindings.values().any(|n| n == "title"));
}

#[test]
fn top_level_perform_handle_resolve() {
    assert!(resolve_source(r#"(perform log "x")(handle log (perform log "y"))(page a4)"#).is_ok());
}

#[test]
fn language_shadowing_let_over_val_resolves() {
    let r = resolve_language_source("(val x 1) (val main (let ((x 2)) x))");
    assert!(r.is_ok(), "{:?}", r.errors);
    assert!(r.env.bindings.values().any(|n| n == "x"));
    assert!(r.env.bindings.values().any(|n| n == "main"));
    assert!(r.env.builtin_colors.is_empty());
}

#[test]
fn language_unbound_identifier_errors_with_span() {
    let r = resolve_language_source("(val main y)");
    assert!(!r.is_ok());
    assert!(
        r.errors
            .iter()
            .any(|e| e.message.contains("unbound identifier `y`")),
        "{:?}",
        r.errors
    );
    let err = r
        .errors
        .iter()
        .find(|e| e.message.contains("`y`"))
        .expect("y error");
    let start: u32 = err.range.start().into();
    let end: u32 = err.range.end().into();
    assert!(end > start, "expected non-empty span for unbound y");
}

#[test]
fn language_fn_param_binding_resolves() {
    let r = resolve_language_source("(val main ((fn (x) x) 1))");
    assert!(r.is_ok(), "{:?}", r.errors);
}

#[test]
fn language_named_fn_and_type_declare() {
    let r = resolve_language_source("(fn id (x) x)\n(type T Num)\n(val main (id 1))");
    assert!(r.is_ok(), "{:?}", r.errors);
    assert!(r.env.bindings.values().any(|n| n == "id"));
    assert!(r.env.bindings.values().any(|n| n == "T"));
}

#[test]
fn language_skips_document_forms_without_color_builtins() {
    let r = resolve_language_source("(page a4 (circle 1 2 3 red))\n(val main 1)");
    assert!(r.is_ok(), "{:?}", r.errors);
    assert!(r.env.builtin_colors.is_empty());
}

#[test]
fn language_data_and_match_resolve() {
    let r = resolve_language_source(
        r#"
(data Option (None) (Some x))
(val main (match (Some 1) (None -> 0) (Some x -> x)))
"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);
    assert!(r.env.bindings.values().any(|n| n == "None"));
    assert!(r.env.bindings.values().any(|n| n == "Some"));
}

#[test]
fn language_binding_map_use_site_to_declaration() {
    use reciplexa_source::offset::ByteOffset;
    use reciplexa_source::range::TextRange;

    let src = "(val x 1) (val main x)";
    let r = resolve_language_source(src);
    assert!(r.is_ok(), "{:?}", r.errors);
    let decl_id = r
        .env
        .bindings
        .iter()
        .find(|(_, name)| *name == "x")
        .map(|(id, _)| *id)
        .expect("x declaration");
    // Last `x` is the use-site in `(val main x)`.
    let use_start = src.rfind('x').expect("use site");
    let range = TextRange::try_new(
        ByteOffset::new(use_start as u32),
        ByteOffset::new((use_start + 1) as u32),
    )
    .unwrap();
    assert_eq!(
        r.binding_map.binding_at(range),
        Some(decl_id),
        "use-site should map to declaration BindingId; map={:?}",
        r.binding_map.uses
    );
    assert_eq!(r.env.bindings.get(&decl_id).map(String::as_str), Some("x"));
}

#[test]
fn rejects_reserved_special_form_as_val_binder() {
    let r = resolve_language_source("(val if 1)");
    assert!(!r.is_ok());
    assert!(
        r.errors
            .iter()
            .any(|e| e.message.contains("reserved special-form") && e.message.contains("`if`")),
        "{:?}",
        r.errors
    );
}

#[test]
fn rejects_reserved_special_form_as_fn_name() {
    let r = resolve_language_source("(fn match (x) x)");
    assert!(!r.is_ok());
    assert!(
        r.errors
            .iter()
            .any(|e| e.message.contains("reserved special-form")),
        "{:?}",
        r.errors
    );
}

#[test]
fn reserved_table_covers_syn_core_forms() {
    for name in [
        "markup",
        "fn",
        "val",
        "type",
        "type-alias",
        "local",
        "rec",
        "let",
        "letrec",
        "if",
        "seq",
        "var",
        "set",
        "handle",
        "with",
    ] {
        assert!(
            reciplexa_bind::is_reserved_special_form(name),
            "{name} should be reserved"
        );
    }
    assert!(!reciplexa_bind::is_reserved_special_form("report"));
}
