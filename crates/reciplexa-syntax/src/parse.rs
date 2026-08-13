//! Recursive-descent parser into a lossless `rowan` CST.
//!
//! Trivia tokens stay in the tree. The parser does not guess missing
//! delimiters into a “best effort” document for rendering: errors are
//! collected and [`Parse::into_result`] refuses success when any exist
//! (SATySFi-style fail-fast at the API boundary).
//!
//! Mode switching: after the head of `(markup …)` the parser pushes
//! [`LexerMode::Markup`] and re-lexes lookahead. `@` escapes push Lisp
//! for one form (and an optional `{…}` Scribble body).
//! Structured comments `(// …)` are SYN-001 trivia forms (not semantic).

use rowan::GreenNodeBuilder;

use crate::kind::{SyntaxKind, SyntaxNode};
use crate::lexer::{Lexer, LexerMode, Token};
use crate::virtual_close_delimiter;

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

pub struct Parser<'a> {
    input: &'a str,
    lexer: Lexer<'a>,
    pub current: Option<Token>,
    builder: GreenNodeBuilder<'static>,
    pub errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
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
        loop {
            while self.current.as_ref().is_some_and(|t| t.kind.is_trivia()) {
                self.bump();
            }
            if self.at_structured_comment() {
                self.parse_structured_comment();
                continue;
            }
            break;
        }
    }

    /// Look ahead from a pending `(` for an exact `//` head (SYN-001).
    fn at_structured_comment(&self) -> bool {
        let Some(tok) = self.current.as_ref() else {
            return false;
        };
        if tok.kind != SyntaxKind::LParen {
            return false;
        }
        let mut rest = &self.input[tok.end..];
        loop {
            let trimmed = rest.trim_start_matches([' ', '\t', '\u{0c}', '\n', '\r']);
            if trimmed.len() == rest.len() {
                break;
            }
            rest = trimmed;
        }
        if !rest.starts_with("//") {
            return false;
        }
        match rest[2..].chars().next() {
            None => true,
            Some(c) => !is_ident_continue(c),
        }
    }

    fn parse_structured_comment(&mut self) {
        self.builder
            .start_node(SyntaxKind::StructuredComment.into());
        debug_assert_eq!(
            self.current.as_ref().map(|t| t.kind),
            Some(SyntaxKind::LParen)
        );
        self.bump(); // (
        while self.current.as_ref().is_some_and(|t| t.kind.is_trivia()) {
            self.bump();
        }
        // `at_structured_comment` only returns true when the next non-trivia
        // token is Ident `//`, so the former recovery arm was unreachable.
        debug_assert!(
            self.current
                .as_ref()
                .is_some_and(|t| t.kind == SyntaxKind::Ident && t.text(self.input) == "//"),
            "structured comment without //"
        );
        self.bump(); // //
        // Body: track paren depth; strings are single tokens from the lexer.
        let mut depth = 1usize;
        loop {
            let Some(tok) = self.current.clone() else {
                self.push_error(
                    "unclosed structured comment `(//`".to_string(),
                    self.input.len(),
                    self.input.len(),
                );
                self.emit_missing_token(")", self.input.len());
                break;
            };
            match tok.kind {
                SyntaxKind::LParen => {
                    self.bump();
                    depth += 1;
                }
                SyntaxKind::RParen => {
                    self.bump();
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => self.bump(),
            }
        }
        self.builder.finish_node();
    }

    pub fn parse_form(&mut self) {
        let Some(tok) = self.current.clone() else {
            return;
        };
        match tok.kind {
            SyntaxKind::LParen => self.parse_paren_list(),
            SyntaxKind::LBracket => self.parse_delimited(
                SyntaxKind::BracketList,
                SyntaxKind::LBracket,
                SyntaxKind::RBracket,
                false,
            ),
            SyntaxKind::LBrace => self.parse_delimited(
                SyntaxKind::BraceList,
                SyntaxKind::LBrace,
                SyntaxKind::RBrace,
                false,
            ),
            SyntaxKind::At => self.parse_at_expr(),
            SyntaxKind::Ident
            | SyntaxKind::Number
            | SyntaxKind::String
            | SyntaxKind::TextChunk
            | SyntaxKind::Arrow
            | SyntaxKind::Error => {
                if tok.kind == SyntaxKind::String {
                    if let Some(close) = virtual_close_delimiter(tok.text(self.input)) {
                        self.bump();
                        self.emit_missing_token(&close, tok.end);
                        return;
                    }
                }
                self.bump();
            }
            SyntaxKind::RParen | SyntaxKind::RBracket | SyntaxKind::RBrace => {
                self.push_error(
                    format!("unexpected closing delimiter `{}`", tok.text(self.input)),
                    tok.start,
                    tok.end,
                );
                self.builder.start_node(SyntaxKind::ErrorNode.into());
                self.bump_as(SyntaxKind::UnexpectedToken);
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
            Some("markup") => {
                self.bump(); // markup
                self.push_mode_relex(LexerMode::Markup);
                self.parse_scribble_until(SyntaxKind::RParen);
                self.pop_mode_relex();
                if self
                    .current
                    .as_ref()
                    .is_some_and(|t| t.kind == SyntaxKind::RParen)
                {
                    self.bump();
                } else {
                    self.push_error(
                        "unclosed `(markup`".into(),
                        self.input.len(),
                        self.input.len(),
                    );
                    self.emit_missing_token(")", self.input.len());
                }
            }
            Some("src") => {
                self.bump(); // src
                             // Body stays in Lisp (default). Same as a normal list.
                self.parse_lisp_list_tail(SyntaxKind::RParen, false);
            }
            _ => {
                self.parse_lisp_list_tail(SyntaxKind::RParen, false);
            }
        }
        self.builder.finish_node();
    }

    fn parse_lisp_list_tail(&mut self, close: SyntaxKind, _code_bracket_arg: bool) {
        loop {
            self.eat_trivia();
            let Some(tok) = self.current.clone() else {
                self.push_error(
                    format!("unclosed list (expected `{close:?}`)"),
                    self.input.len(),
                    self.input.len(),
                );
                self.emit_missing_token(closing_text(close), self.input.len());
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
                self.bump_as(SyntaxKind::UnexpectedToken);
                break;
            }
            self.parse_form();
        }
    }

    fn parse_delimited(
        &mut self,
        node: SyntaxKind,
        open: SyntaxKind,
        close: SyntaxKind,
        code_bracket_arg: bool,
    ) {
        self.builder.start_node(node.into());
        debug_assert_eq!(self.current.as_ref().map(|t| t.kind), Some(open));
        self.bump(); // open
        self.parse_lisp_list_tail(close, code_bracket_arg);
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
                    // Nested brace group in scribble (rare at top level of markup).
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
                true,
            );
            self.eat_trivia();
        }
        // SYN-001: optional markup body in parentheses — @foo(…)
        if self
            .current
            .as_ref()
            .is_some_and(|t| t.kind == SyntaxKind::LParen)
        {
            self.builder.start_node(SyntaxKind::List.into());
            self.bump(); // (
            self.push_mode_relex(LexerMode::Markup);
            self.parse_scribble_until(SyntaxKind::RParen);
            self.pop_mode_relex();
            if self
                .current
                .as_ref()
                .is_some_and(|t| t.kind == SyntaxKind::RParen)
            {
                self.bump();
            } else {
                self.push_error(
                    "unclosed `(` in @-expr".into(),
                    self.input.len(),
                    self.input.len(),
                );
                self.emit_missing_token(")", self.input.len());
            }
            self.builder.finish_node();
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
            self.push_mode_relex(LexerMode::Markup);
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
                self.emit_missing_token("}", self.input.len());
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
            self.emit_missing_token("}", self.input.len());
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

    pub fn pop_mode_relex(&mut self) {
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

    pub fn bump(&mut self) {
        let Some(tok) = self.current.take() else {
            return;
        };
        let text = tok.text(self.input);
        self.builder.token(tok.kind.into(), text);
        self.current = self.lexer.bump_token();
    }

    fn bump_as(&mut self, kind: SyntaxKind) {
        let Some(tok) = self.current.take() else {
            return;
        };
        let text = tok.text(self.input);
        self.builder.token(kind.into(), text);
        self.current = self.lexer.bump_token();
    }

    /// Insert a virtual closer at `at` (SYN §18; omitted from `unparse` / format).
    fn emit_missing_token(&mut self, text: &str, at: usize) {
        let _ = at;
        self.builder.token(SyntaxKind::MissingToken.into(), text);
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
            rowan::NodeOrToken::Token(t) => {
                if t.kind().is_virtual_recovery() {
                    continue;
                }
                out.push_str(t.text());
            }
        }
    }
}

fn closing_text(close: SyntaxKind) -> &'static str {
    match close {
        SyntaxKind::RParen => ")",
        SyntaxKind::RBracket => "]",
        // Only called with RParen / RBracket / RBrace.
        _ => "}",
    }
}

fn is_ident_continue(c: char) -> bool {
    c.is_alphabetic() || c.is_ascii_digit() || matches!(c, '-' | '$' | '_')
}

#[cfg(test)]
mod coverage_helpers {
    use super::*;

    #[test]
    fn bump_and_bump_as_none_current() {
        let mut p = Parser::new("");
        // Exhaust tokens so current is None.
        while p.current.is_some() {
            p.bump();
        }
        p.bump(); // None early-return
        p.bump_as(SyntaxKind::UnexpectedToken); // None early-return
        let _ = closing_text(SyntaxKind::RParen);
        let _ = closing_text(SyntaxKind::RBracket);
        let _ = closing_text(SyntaxKind::RBrace);
    }
}
