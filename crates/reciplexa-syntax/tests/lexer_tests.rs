//! Integration tests moved from src/lexer.rs for region coverage.

use reciplexa_syntax::kind::SyntaxKind;
use reciplexa_syntax::lexer::*;

fn kinds(src: &str) -> Vec<SyntaxKind> {
    Lexer::new(src)
        .tokenize_all()
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

fn texts(src: &str) -> Vec<&str> {
    let mut lex = Lexer::new(src);
    let tokens = lex.tokenize_all();
    tokens.iter().map(|t| t.text(src)).collect()
}

// --- validity: mode stack ---

#[test]
fn new_lexer_starts_in_lisp_with_single_frame() {
    let lex = Lexer::new("");
    assert_eq!(lex.mode(), LexerMode::Lisp);
    assert_eq!(lex.mode_stack(), &[LexerMode::Lisp]);
    assert!(lex.is_eof());
}

#[test]
fn push_and_pop_modes_nest_explicitly() {
    let mut lex = Lexer::new("");
    lex.push_mode(LexerMode::Markup);
    assert_eq!(lex.mode(), LexerMode::Markup);
    lex.push_mode(LexerMode::Lisp);
    assert_eq!(lex.mode(), LexerMode::Lisp);
    assert_eq!(
        lex.mode_stack(),
        &[LexerMode::Lisp, LexerMode::Markup, LexerMode::Lisp]
    );
    assert_eq!(lex.pop_mode(), Some(LexerMode::Lisp));
    assert_eq!(lex.mode(), LexerMode::Markup);
    assert_eq!(lex.pop_mode(), Some(LexerMode::Markup));
    assert_eq!(lex.mode(), LexerMode::Lisp);
}

// --- defect: mode stack ---

#[test]
fn pop_refuses_to_remove_last_mode_frame() {
    let mut lex = Lexer::new("");
    assert_eq!(lex.pop_mode(), None);
    assert_eq!(lex.mode(), LexerMode::Lisp);
    assert_eq!(lex.mode_stack().len(), 1);
}

// --- validity: lisp tokens ---

#[test]
fn lexes_empty_list_with_trivia() {
    let src = "( )\n";
    assert_eq!(
        kinds(src),
        vec![
            SyntaxKind::LParen,
            SyntaxKind::Whitespace,
            SyntaxKind::RParen,
            SyntaxKind::Newline,
        ]
    );
    assert_eq!(texts(src), vec!["(", " ", ")", "\n"]);
}

#[test]
fn lexes_ident_number_string_and_at() {
    let src = r#"(circle 1.5 "x" @)"#;
    assert_eq!(
        kinds(src),
        vec![
            SyntaxKind::LParen,
            SyntaxKind::Ident,
            SyntaxKind::Whitespace,
            SyntaxKind::Number,
            SyntaxKind::Whitespace,
            SyntaxKind::String,
            SyntaxKind::Whitespace,
            SyntaxKind::At,
            SyntaxKind::RParen,
        ]
    );
}

#[test]
fn semicolon_is_error_not_line_comment() {
    // SYN-001 rejects `;` line comments; bare `;` is an error token.
    let src = "; hello\nfoo";
    assert_eq!(
        kinds(src),
        vec![
            SyntaxKind::Error,
            SyntaxKind::Whitespace,
            SyntaxKind::Ident,
            SyntaxKind::Newline,
            SyntaxKind::Ident,
        ]
    );
}

#[test]
fn lexes_signed_and_symbolic_idents() {
    assert_eq!(kinds("+"), vec![SyntaxKind::Ident]);
    assert_eq!(kinds("+10"), vec![SyntaxKind::Number]);
    assert_eq!(kinds("-3.14"), vec![SyntaxKind::Number]);
    assert_eq!(kinds("-"), vec![SyntaxKind::Ident]);
}

#[test]
fn arrow_is_reserved_token_not_ident() {
    // DAT-001 / MAC-001: `->` is SyntaxKind::Arrow, never Ident.
    assert_eq!(kinds("->"), vec![SyntaxKind::Arrow]);
    assert_eq!(texts("->"), vec!["->"]);
    assert_eq!(
        kinds("(some item -> item)"),
        vec![
            SyntaxKind::LParen,
            SyntaxKind::Ident,
            SyntaxKind::Whitespace,
            SyntaxKind::Ident,
            SyntaxKind::Whitespace,
            SyntaxKind::Arrow,
            SyntaxKind::Whitespace,
            SyntaxKind::Ident,
            SyntaxKind::RParen,
        ]
    );
}

#[test]
fn lexes_comparison_and_division_ops_as_single_idents() {
    // SYN-001 §5: fixed operator identifiers, each one token.
    for op in ["/", ">", "<=", ">=", "!="] {
        assert_eq!(kinds(op), vec![SyntaxKind::Ident], "op `{op}`");
        assert_eq!(texts(op), vec![op], "op `{op}`");
    }
    assert_eq!(texts("(<= x y)"), vec!["(", "<=", " ", "x", " ", "y", ")"]);
}

#[test]
fn lexes_brackets_and_braces() {
    assert_eq!(
        kinds("{[ ]}"),
        vec![
            SyntaxKind::LBrace,
            SyntaxKind::LBracket,
            SyntaxKind::Whitespace,
            SyntaxKind::RBracket,
            SyntaxKind::RBrace,
        ]
    );
}

#[test]
fn japanese_ident_is_accepted_in_lisp_mode() {
    // Alphabetic in Unicode — needed early for Japanese report workflow.
    assert_eq!(kinds("円"), vec![SyntaxKind::Ident]);
    assert_eq!(texts("円"), vec!["円"]);
}

// --- validity: scribble ---

#[test]
fn scribble_emits_closing_paren_as_delimiter() {
    let mut lex = Lexer::new("hi)");
    lex.push_mode(LexerMode::Markup);
    assert_eq!(
        lex.tokenize_all()
            .into_iter()
            .map(|t| t.kind)
            .collect::<Vec<_>>(),
        vec![SyntaxKind::TextChunk, SyntaxKind::RParen]
    );
}

#[test]
fn scribble_emits_text_chunks_and_at() {
    let mut lex = Lexer::new("hello @world");
    lex.push_mode(LexerMode::Markup);
    assert_eq!(
        lex.tokenize_all()
            .into_iter()
            .map(|t| t.kind)
            .collect::<Vec<_>>(),
        vec![SyntaxKind::TextChunk, SyntaxKind::At, SyntaxKind::TextChunk]
    );
}

#[test]
fn parser_driven_mode_switch_after_at_lexes_ident() {
    // Mode changes are external (parser-owned). After `@`, the driver
    // pushes Lisp so the escape name tokenizes as Ident, not TextChunk.
    let src = "hi @circle more";
    let mut lex = Lexer::new(src);
    lex.push_mode(LexerMode::Markup);
    assert_eq!(lex.bump_token().unwrap().kind, SyntaxKind::TextChunk);
    assert_eq!(lex.bump_token().unwrap().kind, SyntaxKind::At);
    lex.push_mode(LexerMode::Lisp);
    let ident = lex.bump_token().unwrap();
    assert_eq!(ident.kind, SyntaxKind::Ident);
    assert_eq!(ident.text(src), "circle");
    assert_eq!(lex.pop_mode(), Some(LexerMode::Lisp));
    assert_eq!(lex.bump_token().unwrap().kind, SyntaxKind::TextChunk);
}

#[test]
fn scribble_preserves_newlines_separately() {
    let mut lex = Lexer::new("a\nb");
    lex.push_mode(LexerMode::Markup);
    let tokens = lex.tokenize_all();
    assert_eq!(
        tokens.iter().map(|t| t.kind).collect::<Vec<_>>(),
        vec![
            SyntaxKind::TextChunk,
            SyntaxKind::Newline,
            SyntaxKind::TextChunk
        ]
    );
    assert_eq!(tokens[0].text("a\nb"), "a");
    assert_eq!(tokens[2].text("a\nb"), "b");
}

// --- defect: robust lexing ---

#[test]
fn unterminated_string_is_error_token_not_panic() {
    assert_eq!(kinds("\"abc"), vec![SyntaxKind::Error]);
}

#[test]
fn unknown_glyph_is_single_error_token() {
    assert_eq!(kinds("§"), vec![SyntaxKind::Error]);
    assert_eq!(texts("§"), vec!["§"]);
}

#[test]
fn crlf_counts_as_one_newline_token() {
    assert_eq!(kinds("\r\n"), vec![SyntaxKind::Newline]);
    assert_eq!(texts("\r\n"), vec!["\r\n"]);
}

#[test]
fn tokenize_all_on_empty_is_empty() {
    assert!(kinds("").is_empty());
}

#[test]
fn string_backslash_is_literal_char() {
    // SYN-001 §8.1: no escapes — `"a\nb"` is a, backslash, n, b.
    let src = "\"a\\nb\"";
    let mut lex = Lexer::new(src);
    let tok = lex.bump_token().unwrap();
    assert_eq!(tok.kind, SyntaxKind::String);
    assert_eq!(tok.text(src), src);
}

#[test]
fn quote_after_backslash_terminates_string() {
    // Without escapes, `"` always ends the string even after `\`.
    let src = r#""a\"b""#;
    let mut lex = Lexer::new(src);
    let tok = lex.bump_token().unwrap();
    assert_eq!(tok.kind, SyntaxKind::String);
    assert_eq!(tok.text(src), "\"a\\\"");
}

#[test]
fn lexer_input_accessor() {
    let lex = Lexer::new("abc");
    assert_eq!(lex.input(), "abc");
}

#[test]
fn scribble_lexes_brackets_and_crlf() {
    let mut lex = Lexer::new("[\r\n]");
    lex.push_mode(LexerMode::Markup);
    let kinds: Vec<_> = lex.tokenize_all().into_iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            SyntaxKind::LBracket,
            SyntaxKind::Newline,
            SyntaxKind::RBracket
        ]
    );
}

#[test]
fn scribble_bare_cr_is_newline() {
    let mut lex = Lexer::new("a\rb");
    lex.push_mode(LexerMode::Markup);
    let kinds: Vec<_> = lex.tokenize_all().into_iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            SyntaxKind::TextChunk,
            SyntaxKind::Newline,
            SyntaxKind::TextChunk
        ]
    );
}

#[test]
fn number_with_trailing_dot_stops_before_dot() {
    // `1.` — digit run ends at the dot when no fractional digit follows.
    assert_eq!(kinds("1."), vec![SyntaxKind::Number, SyntaxKind::Error]);
}

#[test]
fn lone_cr_is_newline_trivia() {
    let mut lx = Lexer::new("\ra");
    let tok = lx.bump_token().unwrap();
    assert_eq!(tok.kind, SyntaxKind::Newline);
}

#[test]
fn string_backslash_at_eof_yields_error() {
    // Opening quote then backslash at EOF.
    let mut lx = Lexer::new("\"\\");
    let tok = lx.bump_token().unwrap();
    assert_eq!(tok.kind, SyntaxKind::Error);
}
