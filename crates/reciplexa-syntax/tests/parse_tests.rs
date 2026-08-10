//! Integration tests moved from src/parse.rs for region coverage.

use reciplexa_syntax::kind::SyntaxKind;
use reciplexa_syntax::parse::*;
use reciplexa_syntax::SyntaxNode;
use reciplexa_syntax::Token;

fn parse_ok(src: &str) -> SyntaxNode {
    parse_source(src)
        .into_result()
        .unwrap_or_else(|e| panic!("parse errors for {src:?}: {e:?}"))
}

// --- validity ---

#[test]
fn empty_source_is_empty_file() {
    let root = parse_ok("");
    assert_eq!(root.kind(), SyntaxKind::SourceFile);
    assert_eq!(root.children_with_tokens().count(), 0);
    assert_eq!(unparse(&root), "");
}

#[test]
fn roundtrip_preserves_whitespace_comments_and_newlines() {
    let src = "(// title)\n(circle  1.5\n  \"x\")\n";
    let root = parse_ok(src);
    assert_eq!(unparse(&root), src);
}

#[test]
fn parses_nested_lists() {
    let src = "(a (b c) d)";
    let root = parse_ok(src);
    assert_eq!(unparse(&root), src);
    let lists: Vec<_> = root
        .descendants()
        .filter(|n| n.kind() == SyntaxKind::List)
        .collect();
    assert_eq!(lists.len(), 2);
}

#[test]
fn parses_bracket_and_brace_lists() {
    let src = "(foo [1 2] {a})";
    let root = parse_ok(src);
    assert_eq!(unparse(&root), src);
    assert!(root
        .descendants()
        .any(|n| n.kind() == SyntaxKind::BracketList));
    assert!(root
        .descendants()
        .any(|n| n.kind() == SyntaxKind::BraceList));
}

#[test]
fn japanese_and_symbols_roundtrip() {
    let src = "(円 -> 10)";
    assert_eq!(unparse(&parse_ok(src)), src);
}

#[test]
fn doc_scribble_roundtrip_with_text_and_at() {
    let src = "(markup Hello @em{世界}.)";
    let root = parse_ok(src);
    assert_eq!(unparse(&root), src);
    assert!(root.descendants().any(|n| n.kind() == SyntaxKind::AtExpr));
    assert!(root.descendants_with_tokens().any(|el| el
        .into_token()
        .is_some_and(|t| t.kind() == SyntaxKind::TextChunk)));
}

#[test]
fn doc_with_newlines_roundtrip() {
    let src = "(markup\nline1\nline2\n)";
    assert_eq!(unparse(&parse_ok(src)), src);
}

#[test]
fn src_block_stays_lisp() {
    let src = "(src (define x 1))";
    let root = parse_ok(src);
    assert_eq!(unparse(&root), src);
    assert_eq!(
        root.descendants()
            .filter(|n| n.kind() == SyntaxKind::List)
            .count(),
        2
    );
}

#[test]
fn at_with_paren_markup_body_roundtrip() {
    let src = "(markup @heading(進捗) @strong(型付き))";
    let root = parse_ok(src);
    assert_eq!(unparse(&root), src);
    assert!(root.descendants().any(|n| n.kind() == SyntaxKind::AtExpr));
}

#[test]
fn at_with_bracket_args_and_paren_body() {
    let src = r#"(markup @link["https://example.com"](docs))"#;
    assert_eq!(unparse(&parse_ok(src)), src);
}

#[test]
fn at_ident_without_brace() {
    let src = "(markup see @ref)";
    assert_eq!(unparse(&parse_ok(src)), src);
}

#[test]
fn at_with_bracket_args_and_brace_body() {
    let src = "(markup @link[\"https://example.com\"]{click})";
    let root = parse_ok(src);
    assert_eq!(unparse(&root), src);
    assert!(root
        .descendants()
        .any(|n| n.kind() == SyntaxKind::BracketList));
    assert!(root
        .descendants()
        .any(|n| n.kind() == SyntaxKind::BraceList));
}

#[test]
fn at_with_bracket_args_only() {
    let src = "(markup @cite[42])";
    assert_eq!(unparse(&parse_ok(src)), src);
}

// --- defect ---

#[test]
fn unclosed_paren_is_error_and_into_result_fails() {
    let parse = parse_source("(a");
    assert!(parse.has_errors());
    assert!(parse.into_result().is_err());
}

#[test]
fn unexpected_close_paren_is_error() {
    let parse = parse_source(")");
    assert!(parse.has_errors());
    let err = &parse.errors[0];
    assert!(err.message.contains("unexpected"));
}

#[test]
fn mismatched_closer_is_error() {
    let parse = parse_source("(a]");
    assert!(parse.has_errors());
    assert!(parse.into_result().is_err());
}

#[test]
fn error_tree_still_unparses_without_panic() {
    let parse = parse_source("(a");
    let _ = unparse(&parse.root);
}

#[test]
fn crlf_roundtrip() {
    let src = "(a)\r\n(b)\r\n";
    assert_eq!(unparse(&parse_ok(src)), src);
}

#[test]
fn unclosed_doc_is_error() {
    let parse = parse_source("(markup hello");
    assert!(parse.has_errors());
}

#[test]
fn unclosed_at_brace_is_error() {
    let parse = parse_source("(markup @em{hi)");
    assert!(parse.has_errors());
}

#[test]
fn at_then_immediate_close_is_error() {
    let parse = parse_source("(markup @)");
    assert!(parse.has_errors());
}

#[test]
fn doc_raw_lparen_in_scribble_is_recovered_error() {
    let parse = parse_source("(markup hello (world)");
    assert!(parse.has_errors());
    let _ = unparse(&parse.root);
}

#[test]
fn unclosed_brace_in_at_expr_is_error() {
    let parse = parse_source("(markup @em{hi)");
    assert!(parse.has_errors());
    assert!(parse.errors.iter().any(|e| e.message.contains("unclosed")));
}

#[test]
fn error_recovery_still_yields_source_file_root() {
    let parse = parse_source("(a (b]");
    assert!(parse.has_errors());
    assert_eq!(parse.root.kind(), SyntaxKind::SourceFile);
}

#[test]
fn multiple_top_level_forms() {
    let src = "(page a4)\n(markup hi)\n(src (perform log \"x\"))";
    let root = parse_ok(src);
    assert_eq!(root.children().count(), 3);
}

#[test]
fn structured_comment_only_file() {
    let src = "(// just a comment)\n";
    let root = parse_ok(src);
    assert_eq!(root.children().count(), 1);
    assert_eq!(
        root.children().next().unwrap().kind(),
        SyntaxKind::StructuredComment
    );
}

#[test]
fn structured_comment_roundtrip_and_nesting() {
    let src = "(// outer (// inner) still)\n(page a4)";
    let root = parse_ok(src);
    assert_eq!(unparse(&root), src);
    assert_eq!(root.children().count(), 2);
}

#[test]
fn string_literal_roundtrip() {
    let src = r#"(text 1 2 3 "say hi")"#;
    assert_eq!(unparse(&parse_ok(src)), src);
}

#[test]
fn color_ident_in_list() {
    let src = "(circle 1 2 3 red)";
    assert_eq!(unparse(&parse_ok(src)), src);
}

#[test]
fn nested_unclosed_list_recovers_root() {
    let parse = parse_source("(page a4 (circle 1 2 (rect 1 2 3 4))");
    assert!(parse.has_errors());
    assert_eq!(parse.root.kind(), SyntaxKind::SourceFile);
    let _ = unparse(&parse.root);
}

#[test]
fn doc_unclosed_at_brace_recovers() {
    let parse = parse_source("(markup @section{title)");
    assert!(parse.has_errors());
    assert_eq!(parse.root.kind(), SyntaxKind::SourceFile);
}

#[test]
fn stray_bracket_in_list_is_error() {
    let parse = parse_source("(page a4 (circle 1 2 3])");
    assert!(parse.has_errors());
    let _ = unparse(&parse.root);
}

#[test]
fn src_form_with_unclosed_list_errors() {
    let parse = parse_source("(src (perform log \"x\")\n(page a4)");
    assert!(parse.has_errors());
}

#[test]
fn parse_has_errors_and_into_result_ok() {
    let ok = parse_source("(page a4)");
    assert!(!ok.has_errors());
    assert!(ok.into_result().is_ok());
}

#[test]
fn top_level_bracket_and_brace_forms() {
    assert_eq!(unparse(&parse_ok("[1 2 3]")), "[1 2 3]");
    assert_eq!(unparse(&parse_ok("{a b}")), "{a b}");
}

#[test]
fn standalone_ident_number_string_forms() {
    let root = parse_ok("foo\n42\n\"hi\"");
    // Top-level atoms are tokens under SourceFile (not nested nodes).
    let atoms: Vec<_> = root
        .children_with_tokens()
        .filter_map(|el| el.into_token())
        .filter(|t| {
            !matches!(
                t.kind(),
                SyntaxKind::Whitespace | SyntaxKind::Newline | SyntaxKind::Comment
            )
        })
        .collect();
    assert_eq!(atoms.len(), 3);
    assert_eq!(atoms[0].text(), "foo");
    assert_eq!(atoms[1].text(), "42");
    assert_eq!(atoms[2].text(), "\"hi\"");
    assert_eq!(unparse(&root), "foo\n42\n\"hi\"");
}

#[test]
fn bare_at_without_form_is_parse_error() {
    let parse = parse_source("@");
    assert!(parse.has_errors());
}

#[test]
fn nested_scribble_brace_in_doc() {
    let src = "(markup before {inner} after)";
    assert_eq!(unparse(&parse_ok(src)), src);
}

#[test]
fn scribble_raw_paren_is_error() {
    let parse = parse_source("(markup hello (world)");
    assert!(parse.errors.iter().any(|e| e.message.contains("scribble")));
}

#[test]
fn unclosed_bracket_list_errors() {
    let parse = parse_source("[1 2");
    assert!(parse.has_errors());
}

#[test]
fn parse_error_carries_span() {
    let parse = parse_source(")");
    let err = &parse.errors[0];
    assert!(err.start <= err.end);
    assert!(!err.message.is_empty());
}

#[test]
fn unclosed_brace_list_errors() {
    let parse = parse_source("{a b");
    assert!(parse.has_errors());
}

#[test]
fn mismatched_brace_closer_errors() {
    let parse = parse_source("{a)");
    assert!(parse.has_errors());
}

#[test]
fn comment_and_whitespace_only_trivia() {
    let parse = parse_source("(// just a comment)\n\n  \n");
    assert!(!parse.has_errors());
    assert!(unparse(&parse.root).contains("comment"));
}

#[test]
fn nested_lists_and_string_literals() {
    let src = r#"(page a4 (text 1 2 3 "a\\b"))"#;
    assert_eq!(unparse(&parse_ok(src)), src);
}

#[test]
fn into_result_err_on_unclosed() {
    let parse = parse_source("(page a4");
    assert!(parse.into_result().is_err());
}

#[test]
fn doc_with_at_em_and_braces() {
    let src = "(markup hello @em{world})";
    let root = parse_ok(src);
    assert_eq!(unparse(&root), src);
}

#[test]
fn stray_close_brace_or_bracket_in_scribble() {
    let brace = parse_source("(markup })");
    assert!(brace
        .errors
        .iter()
        .any(|e| e.message.contains("unexpected")));
    let bracket = parse_source("(markup ])");
    assert!(bracket
        .errors
        .iter()
        .any(|e| e.message.contains("unexpected")));
}

#[test]
fn unclosed_nested_scribble_brace_errors() {
    let parse = parse_source("(markup {hi");
    assert!(parse.errors.iter().any(|e| e.message.contains("unclosed")));
}

#[test]
fn defensive_parser_edges_via_private_api() {
    // EOF: parse_form / bump no-ops.
    let mut p = Parser::new("");
    p.parse_form();
    p.bump();
    p.pop_mode_relex();
    assert!(
        p.errors.iter().any(|e| e.message.contains("underflow")),
        "{:?}",
        p.errors
    );

    // Trivia still sitting in `current` (normally eaten first).
    let mut p = Parser::new(" ");
    assert!(p.current.as_ref().is_some_and(|t| t.kind.is_trivia()));
    p.parse_form();
    assert!(p.current.is_none());

    // Non-token kind injected to exercise the unexpected-token recovery arm.
    let mut p = Parser::new("x");
    p.current = Some(Token {
        kind: SyntaxKind::List,
        start: 0,
        end: 0,
    });
    p.parse_form();
    assert!(
        p.errors.iter().any(|e| e.message.contains("unexpected")),
        "{:?}",
        p.errors
    );
}
