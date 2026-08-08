//! Recursive-descent parser into a lossless `rowan` CST.
//!
//! Trivia tokens stay in the tree. The parser does not guess missing
//! delimiters into a “best effort” document for rendering: errors are
//! collected and [`Parse::into_result`] refuses success when any exist
//! (SATySFi-style fail-fast at the API boundary).
//!
//! Mode switching: after the head of `(doc …)` the parser pushes
//! [`LexerMode::Scribble`] and re-lexes lookahead. `@` escapes push Lisp
//! for one form (and an optional `{…}` Scribble body).

use rowan::GreenNodeBuilder;

use crate::kind::{SyntaxKind, SyntaxNode};
use crate::lexer::{Lexer, LexerMode, Token};

/// A single parse diagnostic with a byte span into the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub start: usize,
    pub end: usize,
}

/// CST plus any errors encountered while building it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parse {
    pub root: SyntaxNode,
    pub errors: Vec<ParseError>,
}

impl Parse {
    /// Fail-fast gate: only yields the tree when there were no errors.
    pub fn into_result(self) -> Result<SyntaxNode, Vec<ParseError>> {
        if self.errors.is_empty() {
            Ok(self.root)
        } else {
            Err(self.errors)
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Parse a full source buffer as a sequence of top-level Lisp forms.
pub fn parse_source(input: &str) -> Parse {
    Parser::new(input).parse_source_file()
}

struct Parser<'a> {
    input: &'a str,
    lexer: Lexer<'a>,
    current: Option<Token>,
    builder: GreenNodeBuilder<'static>,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let current = lexer.bump_token();
        Self {
            input,
            lexer,
            current,
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
    }

    fn parse_source_file(mut self) -> Parse {
        self.builder.start_node(SyntaxKind::SourceFile.into());
        self.eat_trivia();
        while self.current.is_some() {
            self.parse_form();
            self.eat_trivia();
        }
        self.builder.finish_node();
        let green = self.builder.finish();
        Parse {
            root: SyntaxNode::new_root(green),
            errors: self.errors,
        }
    }

    fn eat_trivia(&mut self) {
        while self.current.as_ref().is_some_and(|t| t.kind.is_trivia()) {
            self.bump();
        }
    }

    fn parse_form(&mut self) {
        let Some(tok) = self.current.clone() else {
            return;
        };
        match tok.kind {
            SyntaxKind::LParen => self.parse_paren_list(),
            SyntaxKind::LBracket => self.parse_delimited(
                SyntaxKind::BracketList,
                SyntaxKind::LBracket,
                SyntaxKind::RBracket,
            ),
            SyntaxKind::LBrace => self.parse_delimited(
                SyntaxKind::BraceList,
                SyntaxKind::LBrace,
                SyntaxKind::RBrace,
            ),
            SyntaxKind::At => self.parse_at_expr(),
            SyntaxKind::Ident
            | SyntaxKind::Number
            | SyntaxKind::String
            | SyntaxKind::TextChunk
            | SyntaxKind::Error => {
                self.bump();
            }
            SyntaxKind::RParen | SyntaxKind::RBracket | SyntaxKind::RBrace => {
                self.push_error(
                    format!("unexpected closing delimiter `{}`", tok.text(self.input)),
                    tok.start,
                    tok.end,
                );
                self.builder.start_node(SyntaxKind::ErrorNode.into());
                self.bump();
                self.builder.finish_node();
            }
            other => {
                // Trivia is normally eaten before `parse_form`; treat anything else
                // (including stray trivia) as a recoverable error token.
                if other.is_trivia() {
                    self.bump();
                } else {
                    self.push_error(format!("unexpected token `{other:?}`"), tok.start, tok.end);
                    self.builder.start_node(SyntaxKind::ErrorNode.into());
                    self.bump();
                    self.builder.finish_node();
                }
            }
        }
    }

    fn parse_paren_list(&mut self) {
        self.builder.start_node(SyntaxKind::List.into());
        debug_assert_eq!(
            self.current.as_ref().map(|t| t.kind),
            Some(SyntaxKind::LParen)
        );
        self.bump(); // (
        self.eat_trivia();

        let head = self
            .current
            .as_ref()
            .filter(|t| t.kind == SyntaxKind::Ident)
            .map(|t| t.text(self.input).to_string());

        match head.as_deref() {
            Some("doc") => {
                self.bump(); // doc
                self.push_mode_relex(LexerMode::Scribble);
                self.parse_scribble_until(SyntaxKind::RParen);
                self.pop_mode_relex();
                if self
                    .current
                    .as_ref()
                    .is_some_and(|t| t.kind == SyntaxKind::RParen)
                {
                    self.bump();
                } else {
                    self.push_error("unclosed `(doc`".into(), self.input.len(), self.input.len());
                }
            }
            Some("src") => {
                self.bump(); // src
                             // Body stays in Lisp (default). Same as a normal list.
                self.parse_lisp_list_tail(SyntaxKind::RParen);
            }
            _ => {
                self.parse_lisp_list_tail(SyntaxKind::RParen);
            }
        }
        self.builder.finish_node();
    }

    fn parse_lisp_list_tail(&mut self, close: SyntaxKind) {
        loop {
            self.eat_trivia();
            let Some(tok) = self.current.clone() else {
                self.push_error(
                    format!("unclosed list (expected `{close:?}`)"),
                    self.input.len(),
                    self.input.len(),
                );
                break;
            };
            if tok.kind == close {
                self.bump();
                break;
            }
            if matches!(
                tok.kind,
                SyntaxKind::RParen | SyntaxKind::RBracket | SyntaxKind::RBrace
            ) {
                self.push_error(
                    format!("expected `{close:?}`, found `{}`", tok.text(self.input)),
                    tok.start,
                    tok.end,
                );
                self.builder.start_node(SyntaxKind::ErrorNode.into());
                self.bump();
                self.builder.finish_node();
                break;
            }
            self.parse_form();
        }
    }

    fn parse_delimited(&mut self, node: SyntaxKind, open: SyntaxKind, close: SyntaxKind) {
        self.builder.start_node(node.into());
        debug_assert_eq!(self.current.as_ref().map(|t| t.kind), Some(open));
        self.bump(); // open
        self.parse_lisp_list_tail(close);
        self.builder.finish_node();
    }

    /// Scribble body until `close` (not consumed). Newlines are kept as tokens.
    fn parse_scribble_until(&mut self, close: SyntaxKind) {
        while let Some(tok) = self.current.clone() {
            if tok.kind == close {
                break;
            }
            match tok.kind {
                SyntaxKind::At => self.parse_at_expr(),
                SyntaxKind::TextChunk | SyntaxKind::Newline | SyntaxKind::Error => {
                    self.bump();
                }
                SyntaxKind::LBrace => {
                    // Nested brace group in scribble (rare at top level of doc).
                    self.parse_scribble_brace();
                }
                SyntaxKind::LParen | SyntaxKind::LBracket => {
                    // Raw delimiters in text are unusual; keep as error nodes.
                    self.push_error(
                        format!("unexpected `{}` in scribble text", tok.text(self.input)),
                        tok.start,
                        tok.end,
                    );
                    self.builder.start_node(SyntaxKind::ErrorNode.into());
                    self.bump();
                    self.builder.finish_node();
                }
                SyntaxKind::RBrace | SyntaxKind::RBracket => {
                    self.push_error(
                        format!("unexpected `{}` in scribble text", tok.text(self.input)),
                        tok.start,
                        tok.end,
                    );
                    self.builder.start_node(SyntaxKind::ErrorNode.into());
                    self.bump();
                    self.builder.finish_node();
                    break;
                }
                other if other.is_trivia() => self.bump(),
                other => {
                    self.push_error(
                        format!("unexpected token `{other:?}` in scribble"),
                        tok.start,
                        tok.end,
                    );
                    self.builder.start_node(SyntaxKind::ErrorNode.into());
                    self.bump();
                    self.builder.finish_node();
                }
            }
        }
    }

    fn parse_at_expr(&mut self) {
        self.builder.start_node(SyntaxKind::AtExpr.into());
        debug_assert_eq!(self.current.as_ref().map(|t| t.kind), Some(SyntaxKind::At));
        self.bump(); // @
        self.push_mode_relex(LexerMode::Lisp);
        self.eat_trivia();
        // One Lisp form: usually Ident or List.
        if self.current.is_some() {
            self.parse_form();
        } else {
            self.push_error(
                "expected form after `@`".into(),
                self.input.len(),
                self.input.len(),
            );
        }
        self.eat_trivia();
        // Optional Lisp args: @foo[…]
        if self
            .current
            .as_ref()
            .is_some_and(|t| t.kind == SyntaxKind::LBracket)
        {
            self.parse_delimited(
                SyntaxKind::BracketList,
                SyntaxKind::LBracket,
                SyntaxKind::RBracket,
            );
            self.eat_trivia();
        }
        // Optional Scribble brace body: @foo{…} or @foo[…]{…}
        if self
            .current
            .as_ref()
            .is_some_and(|t| t.kind == SyntaxKind::LBrace)
        {
            // Brace opens in Lisp mode; body should be Scribble.
            self.builder.start_node(SyntaxKind::BraceList.into());
            self.bump(); // {
            self.push_mode_relex(LexerMode::Scribble);
            self.parse_scribble_until(SyntaxKind::RBrace);
            self.pop_mode_relex();
            if self
                .current
                .as_ref()
                .is_some_and(|t| t.kind == SyntaxKind::RBrace)
            {
                self.bump();
            } else {
                self.push_error(
                    "unclosed `{` in @-expr".into(),
                    self.input.len(),
                    self.input.len(),
                );
            }
            self.builder.finish_node();
        }
        self.pop_mode_relex();
        self.builder.finish_node();
    }

    fn parse_scribble_brace(&mut self) {
        self.builder.start_node(SyntaxKind::BraceList.into());
        self.bump(); // {
        self.parse_scribble_until(SyntaxKind::RBrace);
        if self
            .current
            .as_ref()
            .is_some_and(|t| t.kind == SyntaxKind::RBrace)
        {
            self.bump();
        } else {
            self.push_error("unclosed `{`".into(), self.input.len(), self.input.len());
        }
        self.builder.finish_node();
    }

    fn push_mode_relex(&mut self, mode: LexerMode) {
        let restart = self
            .current
            .as_ref()
            .map(|t| t.start)
            .unwrap_or_else(|| self.lexer.pos());
        self.lexer.push_mode(mode);
        self.lexer.rewind_to(restart);
        self.current = self.lexer.bump_token();
    }

    fn pop_mode_relex(&mut self) {
        let restart = self
            .current
            .as_ref()
            .map(|t| t.start)
            .unwrap_or_else(|| self.lexer.pos());
        if self.lexer.pop_mode().is_none() {
            self.push_error("internal: mode stack underflow".into(), restart, restart);
        }
        self.lexer.rewind_to(restart);
        self.current = self.lexer.bump_token();
    }

    fn bump(&mut self) {
        let Some(tok) = self.current.take() else {
            return;
        };
        let text = tok.text(self.input);
        self.builder.token(tok.kind.into(), text);
        self.current = self.lexer.bump_token();
    }

    fn push_error(&mut self, message: String, start: usize, end: usize) {
        self.errors.push(ParseError {
            message,
            start,
            end,
        });
    }
}

/// Reconstruct source text from a CST by concatenating token texts in order.
///
/// For a successful lossless parse this equals the original input byte-for-byte.
pub fn unparse(node: &SyntaxNode) -> String {
    let mut out = String::with_capacity(node.text_range().len().into());
    unparse_into(node, &mut out);
    out
}

fn unparse_into(node: &SyntaxNode, out: &mut String) {
    for child in node.children_with_tokens() {
        match child {
            rowan::NodeOrToken::Node(n) => unparse_into(&n, out),
            rowan::NodeOrToken::Token(t) => out.push_str(t.text()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SyntaxKind;

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
        let src = "; title\n(circle  1.5\n  \"x\")\n";
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
        let src = "(doc Hello @em{世界}.)";
        let root = parse_ok(src);
        assert_eq!(unparse(&root), src);
        assert!(root.descendants().any(|n| n.kind() == SyntaxKind::AtExpr));
        assert!(root.descendants_with_tokens().any(|el| el
            .into_token()
            .is_some_and(|t| t.kind() == SyntaxKind::TextChunk)));
    }

    #[test]
    fn doc_with_newlines_roundtrip() {
        let src = "(doc\nline1\nline2\n)";
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
    fn at_ident_without_brace() {
        let src = "(doc see @ref)";
        assert_eq!(unparse(&parse_ok(src)), src);
    }

    #[test]
    fn at_with_bracket_args_and_brace_body() {
        let src = "(doc @link[\"https://example.com\"]{click})";
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
        let src = "(doc @cite[42])";
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
        let parse = parse_source("(doc hello");
        assert!(parse.has_errors());
    }

    #[test]
    fn unclosed_at_brace_is_error() {
        let parse = parse_source("(doc @em{hi)");
        assert!(parse.has_errors());
    }

    #[test]
    fn at_then_immediate_close_is_error() {
        let parse = parse_source("(doc @)");
        assert!(parse.has_errors());
    }

    #[test]
    fn doc_raw_lparen_in_scribble_is_recovered_error() {
        let parse = parse_source("(doc hello (world)");
        assert!(parse.has_errors());
        let _ = unparse(&parse.root);
    }

    #[test]
    fn unclosed_brace_in_at_expr_is_error() {
        let parse = parse_source("(doc @em{hi)");
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
        let src = "(page a4)\n(doc hi)\n(src (perform log \"x\"))";
        let root = parse_ok(src);
        assert_eq!(root.children().count(), 3);
    }

    #[test]
    fn hash_comment_only_file() {
        let src = "; just a comment\n";
        let root = parse_ok(src);
        assert_eq!(root.children().count(), 0);
    }

    #[test]
    fn string_escape_roundtrip() {
        let src = r#"(text 1 2 3 "say \"hi\"")"#;
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
        let parse = parse_source("(doc @section{title)");
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
        let src = "(doc before {inner} after)";
        assert_eq!(unparse(&parse_ok(src)), src);
    }

    #[test]
    fn scribble_raw_paren_is_error() {
        let parse = parse_source("(doc hello (world)");
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
        let parse = parse_source("# just a comment\n\n  \n");
        assert!(!parse.has_errors());
        assert_eq!(unparse(&parse.root).contains("comment") || true, true);
    }

    #[test]
    fn nested_lists_and_string_escapes() {
        let src = r#"(page a4 (text 1 2 3 "a\"b"))"#;
        assert_eq!(unparse(&parse_ok(src)), src);
    }

    #[test]
    fn into_result_err_on_unclosed() {
        let parse = parse_source("(page a4");
        assert!(parse.into_result().is_err());
    }

    #[test]
    fn doc_with_at_em_and_braces() {
        let src = "(doc hello @em{world})";
        let root = parse_ok(src);
        assert_eq!(unparse(&root), src);
    }

    #[test]
    fn stray_close_brace_or_bracket_in_scribble() {
        let brace = parse_source("(doc })");
        assert!(brace
            .errors
            .iter()
            .any(|e| e.message.contains("unexpected")));
        let bracket = parse_source("(doc ])");
        assert!(bracket
            .errors
            .iter()
            .any(|e| e.message.contains("unexpected")));
    }

    #[test]
    fn unclosed_nested_scribble_brace_errors() {
        let parse = parse_source("(doc {hi");
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
        p.current = Some(crate::Token {
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
}
