//! Private-path tests moved from src/lib.rs for region coverage.

use reciplexa_effect::*;
use reciplexa_syntax::parse_source;

#[test]
fn list_head_ident_edges() {
    let root = parse_source("[1]\n()\n(123)\n(src [1])\n( \"x\")")
        .into_result()
        .unwrap();
    let mut forms = root.children();
    let bracket = forms.next().unwrap();
    assert!(list_head_ident(&bracket).is_none());
    let empty = forms.next().unwrap();
    assert!(list_head_ident(&empty).is_none());
    let numbered = forms.next().unwrap();
    assert!(list_head_ident(&numbered).is_none());
    let src = forms.next().unwrap();
    assert_eq!(list_head_ident(&src).as_deref(), Some("src"));
    // String token as first atom (not Ident) → Ident-false arm then break.
    let string_headed = forms.next().unwrap();
    assert!(list_head_ident(&string_headed).is_none());
    // Non-list children of src are skipped by run_src_forms / collect.
    let mut h = TestHandler::default();
    assert!(run_src_forms(&mut h, &src).unwrap().is_empty());
    assert!(collect_performs("(src [1] (noop))").unwrap().is_empty());
}

#[test]
fn parse_perform_node_rejects_non_perform() {
    let root = parse_source("(page a4)").into_result().unwrap();
    let page = root.children().next().unwrap();
    let err = parse_perform_node(&page).unwrap_err();
    assert!(err.message.contains("perform"));
}

#[test]
fn handle_skips_non_list_nodes() {
    // BracketList body child hits the `_` arm in run_handle.
    let mut h = TestHandler::default();
    let vals = run_source_effects(
        &mut h,
        r#"(src (handle write-path [1] (perform log "ok")))"#,
    )
    .unwrap();
    assert_eq!(h.logs, vec!["ok"]);
    assert_eq!(vals.len(), 1);
}

#[test]
fn parse_errors_propagate_through_collect_and_run() {
    assert!(collect_performs("(src (perform log").is_err());
    let mut h = TestHandler::default();
    assert!(run_source_effects(&mut h, "(src (perform log").is_err());
}

#[test]
fn headless_src_form_errors() {
    let root = parse_source("(src ())").into_result().unwrap();
    let src = root.children().next().unwrap();
    let mut h = TestHandler::default();
    let err = run_src_forms(&mut h, &src).unwrap_err();
    assert!(err.message.contains("head"));
}

#[test]
fn handle_body_perform_errors_propagate() {
    let mut h = TestHandler::default();
    // Unknown op inside handle body → run_src_form Err via `?`.
    let err = run_source_effects(&mut h, "(src (handle log (nope)))").unwrap_err();
    assert!(err.message.contains("unsupported") || err.message.contains("nope"));
}

#[test]
fn nested_perform_collect_errors() {
    // Malformed nested perform should fail collect_performs_in_list `?`.
    assert!(collect_performs("(src (perform))").is_err());
    // Nested under handle → outer `collect_performs_in_list` `?` Err arm.
    assert!(collect_performs("(src (handle log (perform)))").is_err());
}
