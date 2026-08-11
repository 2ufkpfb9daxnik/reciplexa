//! Integration tests moved from src/markup.rs for region coverage.

use reciplexa_syntax::kind::SyntaxKind;
use reciplexa_syntax::markup::*;
use reciplexa_syntax::parse_source;
use reciplexa_syntax::SyntaxNode;

fn markup_list(src: &str) -> SyntaxNode {
    let root = parse_source(src).into_result().unwrap();
    root.children()
        .find(|n| n.kind() == SyntaxKind::List)
        .expect("top-level list")
}

// --- validity ---

#[test]
fn plain_text_doc() {
    let parts = markup_parts(&markup_list("(markup Hello world)")).unwrap();
    // Scribble TextChunks include the space after `markup`.
    assert_eq!(parts, vec![MarkupPart::Text(" Hello world".into())]);
    assert_eq!(flatten_readable(&parts), "Hello world");
}

#[test]
fn newlines_become_spaces_when_flattened() {
    let parts = markup_parts(&markup_list("(markup\nline1\nline2\n)")).unwrap();
    assert!(parts.contains(&MarkupPart::Newline));
    assert_eq!(flatten_readable(&parts), "line1 line2");
    assert_eq!(flatten_lines(&parts), vec!["line1", "line2"]);
}

#[test]
fn at_em_structure_and_flatten() {
    let parts = markup_parts(&markup_list("(markup Hello @em{世界}.)")).unwrap();
    assert_eq!(
        parts,
        vec![
            MarkupPart::Text(" Hello ".into()),
            MarkupPart::At {
                name: "em".into(),
                bracket_args: None,
                brace_body: vec![MarkupPart::Text("世界".into())],
            },
            MarkupPart::Text(".".into()),
        ]
    );
    assert_eq!(flatten_readable(&parts), "Hello 世界.");
}

#[test]
fn at_link_and_cite() {
    let parts = markup_parts(&markup_list(
        "(markup See @link[\"https://example.com\"]{docs}. Cite @cite[42].)",
    ))
    .unwrap();
    assert_eq!(flatten_readable(&parts), "See docs. Cite 42.");
}

// --- defect ---

#[test]
fn non_doc_list_errors() {
    let err = markup_parts(&markup_list("(page a4)")).unwrap_err();
    assert!(err.message.contains("markup"));
}

#[test]
fn bare_at_ident_flattens_empty() {
    let parts = markup_parts(&markup_list("(markup see @ref)")).unwrap();
    assert_eq!(flatten_readable(&parts), "see");
}

#[test]
fn at_with_brace_only_body() {
    let parts = markup_parts(&markup_list("(markup @em{emphasis})")).unwrap();
    assert_eq!(flatten_readable(&parts), "emphasis");
}

#[test]
fn empty_doc_body() {
    let parts = markup_parts(&markup_list("(markup)")).unwrap();
    assert!(parts.is_empty());
    assert_eq!(flatten_readable(&parts), "");
}

#[test]
fn doc_part_debug() {
    let part = MarkupPart::Text("x".into());
    assert!(format!("{part:?}").contains("Text"));
}

#[test]
fn non_list_node_errors() {
    let root = parse_source("(markup hi)").into_result().unwrap();
    let err = markup_parts(&root).unwrap_err();
    assert!(err.message.contains("List"));
}

#[test]
fn wrong_head_errors() {
    let list = markup_list("(page a4)");
    let err = markup_parts(&list).unwrap_err();
    assert!(err.message.contains("markup"));
}

#[test]
fn error_node_in_body_errors() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::ErrorNode.into());
        b.token(SyntaxKind::Error.into(), "(");
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let err = markup_parts(&list).unwrap_err();
    assert!(err.message.contains("error node"), "{}", err.message);
}

#[test]
fn nested_brace_list_at_top_level() {
    let parts = markup_parts(&markup_list("(markup {nested})")).unwrap();
    assert_eq!(flatten_readable(&parts), "nested");
}

#[test]
fn flatten_lines_skips_empty_lines() {
    let parts = vec![
        MarkupPart::Text("  ".into()),
        MarkupPart::Newline,
        MarkupPart::Text("ok".into()),
    ];
    assert_eq!(flatten_lines(&parts), vec!["ok"]);
}

#[test]
fn at_with_only_bracket_args() {
    let parts = markup_parts(&markup_list("(markup prefix @unknown[args-only])")).unwrap();
    assert_eq!(flatten_readable(&parts), "prefix args-only");
}

#[test]
fn doc_walk_error_debug() {
    let err = MarkupWalkError::new("msg");
    assert!(format!("{err:?}").contains("msg"));
}

fn green_list(build: impl FnOnce(&mut rowan::GreenNodeBuilder<'_>)) -> SyntaxNode {
    let mut b = rowan::GreenNodeBuilder::new();
    b.start_node(SyntaxKind::List.into());
    build(&mut b);
    b.finish_node();
    SyntaxNode::new_root(b.finish())
}

#[test]
fn whitespace_and_comment_after_body_started() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.token(SyntaxKind::TextChunk.into(), "hi");
        b.token(SyntaxKind::Whitespace.into(), " ");
        b.token(SyntaxKind::Comment.into(), ";c");
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let parts = markup_parts(&list).unwrap();
    assert!(parts
        .iter()
        .any(|p| matches!(p, MarkupPart::Text(t) if t == " ")));
}

#[test]
fn unexpected_ident_token_in_doc_body() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.token(SyntaxKind::Ident.into(), "nope");
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let err = markup_parts(&list).unwrap_err();
    assert!(err.message.contains("unexpected token"));
}

#[test]
fn node_before_doc_head_errors() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.start_node(SyntaxKind::BraceList.into());
        b.token(SyntaxKind::LBrace.into(), "{");
        b.token(SyntaxKind::RBrace.into(), "}");
        b.finish_node();
        b.token(SyntaxKind::Ident.into(), "markup");
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let err = markup_parts(&list).unwrap_err();
    assert!(err.message.contains("before body"));
}

#[test]
fn unexpected_list_node_in_doc_body() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::List.into());
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::RParen.into(), ")");
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let err = markup_parts(&list).unwrap_err();
    assert!(err.message.contains("unexpected node"));
}

#[test]
fn empty_parens_without_doc_head() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let err = markup_parts(&list).unwrap_err();
    assert!(err.message.contains("expected head `markup`"));
}

#[test]
fn at_empty_parens_without_ident_errors() {
    // Empty `@(…)` embedding is rejected.
    let err = markup_parts(&markup_list("(markup @())")).unwrap_err();
    assert!(
        err.message.contains("empty") || err.message.contains("identifier"),
        "{}",
        err.message
    );
}

#[test]
fn at_embed_paren_form_is_embed_part() {
    // SYN §17.6: `@(space 20mm)` is arbitrary code embedding.
    let parts = markup_parts(&markup_list("(markup @(space 20mm))")).unwrap();
    assert!(
        parts
            .iter()
            .any(|p| matches!(p, MarkupPart::Embed { source } if source.contains("space"))),
        "{parts:?}"
    );
}

#[test]
fn at_with_paren_only_body() {
    // SYN-001: `@name(...)` scribble body (paren form of `@name{...}`).
    let parts = markup_parts(&markup_list("(markup @em(emphasis))")).unwrap();
    assert_eq!(flatten_readable(&parts), "emphasis");
}

#[test]
fn at_without_ident_errors() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::AtExpr.into());
        b.token(SyntaxKind::At.into(), "@");
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let err = markup_parts(&list).unwrap_err();
    assert!(err.message.contains("identifier after `@`"));
}

#[test]
fn at_ignores_unknown_nested_nodes() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::AtExpr.into());
        b.token(SyntaxKind::At.into(), "@");
        b.token(SyntaxKind::Ident.into(), "em");
        b.start_node(SyntaxKind::ErrorNode.into());
        b.token(SyntaxKind::Error.into(), "?");
        b.finish_node();
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let parts = markup_parts(&list).unwrap();
    assert!(matches!(parts[0], MarkupPart::At { ref name, .. } if name == "em"));
}

#[test]
fn brace_with_nested_at_expr() {
    let parts = markup_parts(&markup_list("(markup {@em{x} y})")).unwrap();
    assert_eq!(flatten_readable(&parts), "x y");
}

#[test]
fn brace_whitespace_token_is_kept() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::BraceList.into());
        b.token(SyntaxKind::LBrace.into(), "{");
        b.token(SyntaxKind::Whitespace.into(), " ");
        b.token(SyntaxKind::TextChunk.into(), "x");
        b.token(SyntaxKind::RBrace.into(), "}");
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let parts = markup_parts(&list).unwrap();
    assert!(parts
        .iter()
        .any(|p| matches!(p, MarkupPart::Text(t) if t == " ")));
}

#[test]
fn unexpected_token_and_node_in_brace() {
    let bad_tok = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::BraceList.into());
        b.token(SyntaxKind::LBrace.into(), "{");
        b.token(SyntaxKind::Ident.into(), "bad");
        b.token(SyntaxKind::RBrace.into(), "}");
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    assert!(markup_parts(&bad_tok)
        .unwrap_err()
        .message
        .contains("scribble brace"));

    let bad_node = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::BraceList.into());
        b.token(SyntaxKind::LBrace.into(), "{");
        b.start_node(SyntaxKind::List.into());
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::RParen.into(), ")");
        b.finish_node();
        b.token(SyntaxKind::RBrace.into(), "}");
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    assert!(markup_parts(&bad_node)
        .unwrap_err()
        .message
        .contains("unexpected node"));
}

#[test]
fn error_node_in_scribble_brace() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::BraceList.into());
        b.token(SyntaxKind::LBrace.into(), "{");
        b.start_node(SyntaxKind::ErrorNode.into());
        b.token(SyntaxKind::Error.into(), "(");
        b.finish_node();
        b.token(SyntaxKind::RBrace.into(), "}");
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let err = markup_parts(&list).unwrap_err();
    assert!(err.message.contains("scribble body contains an error node"));
}

#[test]
fn nested_brace_lists_unwrap() {
    let parts = markup_parts(&markup_list("(markup {{inner}})")).unwrap();
    assert_eq!(flatten_readable(&parts), "inner");
}

#[test]
fn short_bracket_args_use_full_text() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::AtExpr.into());
        b.token(SyntaxKind::At.into(), "@");
        b.token(SyntaxKind::Ident.into(), "cite");
        b.start_node(SyntaxKind::BracketList.into());
        b.token(SyntaxKind::LBracket.into(), "[");
        b.finish_node();
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let parts = markup_parts(&list).unwrap();
    assert_eq!(flatten_readable(&parts), "[");
}

#[test]
fn leading_trivia_before_doc_head_is_skipped() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Whitespace.into(), " ");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.token(SyntaxKind::TextChunk.into(), "x");
        b.token(SyntaxKind::RParen.into(), ")");
    });
    assert_eq!(flatten_readable(&markup_parts(&list).unwrap()), "x");
}

#[test]
fn scribble_comment_trivia_is_skipped() {
    let parts = markup_parts(&markup_list("(markup {hi;c x})")).unwrap();
    assert!(!parts.is_empty());
}

#[test]
fn at_expr_brace_question_propagates_nested_error() {
    // Synthetic AtExpr with BraceList containing ErrorNode → `?` on walk_scribble_container.
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::AtExpr.into());
        b.token(SyntaxKind::At.into(), "@");
        b.token(SyntaxKind::Ident.into(), "em");
        b.start_node(SyntaxKind::BraceList.into());
        b.token(SyntaxKind::LBrace.into(), "{");
        b.start_node(SyntaxKind::ErrorNode.into());
        b.token(SyntaxKind::Error.into(), "!");
        b.finish_node();
        b.token(SyntaxKind::RBrace.into(), "}");
        b.finish_node();
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let err = markup_parts(&list).unwrap_err();
    assert!(err.message.contains("error node"));
}

#[test]
fn scribble_nested_at_question_propagates() {
    // BraceList containing AtExpr without ident → `?` on walk_at_expr.
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::BraceList.into());
        b.token(SyntaxKind::LBrace.into(), "{");
        b.start_node(SyntaxKind::AtExpr.into());
        b.token(SyntaxKind::At.into(), "@");
        b.finish_node();
        b.token(SyntaxKind::RBrace.into(), "}");
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let err = markup_parts(&list).unwrap_err();
    assert!(err.message.contains("identifier after `@`"));
}

#[test]
fn nested_brace_list_question_propagates() {
    // Outer brace contains nested brace with ErrorNode → `?` on nested walk_scribble_container.
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::BraceList.into());
        b.token(SyntaxKind::LBrace.into(), "{");
        b.start_node(SyntaxKind::BraceList.into());
        b.token(SyntaxKind::LBrace.into(), "{");
        b.start_node(SyntaxKind::ErrorNode.into());
        b.token(SyntaxKind::Error.into(), "!");
        b.finish_node();
        b.token(SyntaxKind::RBrace.into(), "}");
        b.finish_node();
        b.token(SyntaxKind::RBrace.into(), "}");
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let err = markup_parts(&list).unwrap_err();
    assert!(err.message.contains("error node"));
}

#[test]
fn at_expr_ignores_non_ident_tokens() {
    // Non-ident token after `@` hits the Ident-false arm then fails for missing name.
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::AtExpr.into());
        b.token(SyntaxKind::At.into(), "@");
        b.token(SyntaxKind::Number.into(), "1");
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let err = markup_parts(&list).unwrap_err();
    assert!(err.message.contains("identifier after `@`"));
}

#[test]
fn brace_comment_trivia_else_arm() {
    let list = green_list(|b| {
        b.token(SyntaxKind::LParen.into(), "(");
        b.token(SyntaxKind::Ident.into(), "markup");
        b.start_node(SyntaxKind::BraceList.into());
        b.token(SyntaxKind::LBrace.into(), "{");
        b.token(SyntaxKind::Comment.into(), ";c");
        b.token(SyntaxKind::TextChunk.into(), "x");
        b.token(SyntaxKind::RBrace.into(), "}");
        b.finish_node();
        b.token(SyntaxKind::RParen.into(), ")");
    });
    let parts = markup_parts(&list).unwrap();
    assert!(parts
        .iter()
        .any(|p| matches!(p, MarkupPart::Text(t) if t == "x")));
}
