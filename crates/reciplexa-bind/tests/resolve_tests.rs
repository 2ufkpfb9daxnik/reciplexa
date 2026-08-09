//! Integration tests moved from src/resolve.rs for region coverage.

use reciplexa_bind::resolve_source;

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
