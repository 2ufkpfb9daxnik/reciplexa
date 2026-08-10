//! Hand-written lexer with an explicit mode stack.
//!
//! Reciplexa's surface is a two-faced LISP: pure S-expressions in `src`
//! blocks and Scribble-like text (with `@`-escapes) in `markup` blocks.
//! Encoding that as `mode_stack: Vec<LexerMode>` keeps nesting explicit—
//! a finite state enum alone cannot represent nested mode switches
//! (`(markup @foo{... (src ...) ...})`) without an ad-hoc counter.
//!
//! Tokenization is incremental (`bump_token`) so the parser can drive the
//! lexer and so unit tests can assert one decision at a time.

use crate::SyntaxKind;

/// Lexical reading mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LexerMode {
    /// Pure S-expression tokenization (identifiers, numbers, strings, punct).
    Lisp,
    /// Markup / Scribble-like text chunks until `@` introduces a Lisp escape.
    Markup,
}

/// A single lexeme with byte offsets into the original source.
///
/// Offsets are UTF-8 byte indices so they align with `rowan`'s text model
/// and with lossless CST edits that rewrite numeric leaves in place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: SyntaxKind,
    pub start: usize,
    pub end: usize,
}

impl Token {
    pub fn text<'a>(&self, src: &'a str) -> &'a str {
        &src[self.start..self.end]
    }
}

/// Streaming lexer over a full source buffer.
#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    input: &'a str,
    /// Next byte index to read.
    pos: usize,
    mode_stack: Vec<LexerMode>,
}

impl<'a> Lexer<'a> {
    /// Start in [`LexerMode::Lisp`].
    ///
    /// Files are S-expression rooted (`(src …)` / `(markup …)`); Markup mode is
    /// entered only when the parser recognizes a `markup` form and pushes mode.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            mode_stack: vec![LexerMode::Lisp],
        }
    }

    pub fn input(&self) -> &'a str {
        self.input
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Current mode; the stack is never empty by construction.
    pub fn mode(&self) -> LexerMode {
        *self
            .mode_stack
            .last()
            .expect("lexer mode stack is never empty")
    }

    pub fn mode_stack(&self) -> &[LexerMode] {
        &self.mode_stack
    }

    /// Rewind the read head to a byte offset (must be a char boundary).
    ///
    /// Used when the parser switches lexer modes: the one-token lookahead was
    /// produced under the old mode and must be re-lexed under the new one.
    pub fn rewind_to(&mut self, byte_pos: usize) {
        debug_assert!(self.input.is_char_boundary(byte_pos));
        self.pos = byte_pos.min(self.input.len());
    }

    pub fn push_mode(&mut self, mode: LexerMode) {
        self.mode_stack.push(mode);
    }

    /// Pop a mode. Refuses to pop the last frame so the lexer always has a
    /// defined mode (defect inputs that over-pop should not crash hosts).
    pub fn pop_mode(&mut self) -> Option<LexerMode> {
        if self.mode_stack.len() <= 1 {
            return None;
        }
        self.mode_stack.pop()
    }

    /// Lex the next token, or `None` at EOF.
    pub fn bump_token(&mut self) -> Option<Token> {
        if self.is_eof() {
            return None;
        }
        match self.mode() {
            LexerMode::Lisp => Some(self.bump_lisp()),
            LexerMode::Markup => Some(self.bump_scribble()),
        }
    }

    /// Collect all remaining tokens (convenience for tests and tooling).
    pub fn tokenize_all(&mut self) -> Vec<Token> {
        let mut out = Vec::new();
        while let Some(tok) = self.bump_token() {
            out.push(tok);
        }
        out
    }

    fn bump_lisp(&mut self) -> Token {
        let start = self.pos;
        let ch = self.peek_char().expect("caller checked EOF");

        // Newline is its own trivia kind so editors can reason about lines
        // without re-scanning whitespace blobs.
        if ch == '\n' {
            self.advance_char();
            return self.finish(SyntaxKind::Newline, start);
        }
        if ch == '\r' {
            self.advance_char();
            if self.peek_char() == Some('\n') {
                self.advance_char();
            }
            return self.finish(SyntaxKind::Newline, start);
        }

        if ch.is_whitespace() {
            self.advance_char();
            while let Some(c) = self.peek_char() {
                if c == '\n' || c == '\r' || !c.is_whitespace() {
                    break;
                }
                self.advance_char();
            }
            return self.finish(SyntaxKind::Whitespace, start);
        }

        // SYN-001: `;` line comments are not part of the language.
        // Structured comments are `(// …)` forms, recognized by the parser.

        let kind = match ch {
            '(' => {
                self.advance_char();
                SyntaxKind::LParen
            }
            ')' => {
                self.advance_char();
                SyntaxKind::RParen
            }
            '{' => {
                self.advance_char();
                SyntaxKind::LBrace
            }
            '}' => {
                self.advance_char();
                SyntaxKind::RBrace
            }
            '[' => {
                self.advance_char();
                SyntaxKind::LBracket
            }
            ']' => {
                self.advance_char();
                SyntaxKind::RBracket
            }
            '@' => {
                self.advance_char();
                SyntaxKind::At
            }
            '"' => return self.bump_string(start),
            c if is_ident_start(c) => return self.bump_ident_or_number(start),
            c if c.is_ascii_digit() => return self.bump_number(start),
            _ => {
                self.advance_char();
                SyntaxKind::Error
            }
        };
        self.finish(kind, start)
    }

    fn bump_scribble(&mut self) -> Token {
        let start = self.pos;
        let ch = self.peek_char().expect("caller checked EOF");

        // Delimiters stay visible so `(markup …)` can close and `@foo{…}` can nest.
        let punct = match ch {
            '@' => Some(SyntaxKind::At),
            '(' => Some(SyntaxKind::LParen),
            ')' => Some(SyntaxKind::RParen),
            '{' => Some(SyntaxKind::LBrace),
            '}' => Some(SyntaxKind::RBrace),
            '[' => Some(SyntaxKind::LBracket),
            ']' => Some(SyntaxKind::RBracket),
            _ => None,
        };
        if let Some(kind) = punct {
            self.advance_char();
            return self.finish(kind, start);
        }

        if ch == '\n' {
            self.advance_char();
            return self.finish(SyntaxKind::Newline, start);
        }
        if ch == '\r' {
            self.advance_char();
            if self.peek_char() == Some('\n') {
                self.advance_char();
            }
            return self.finish(SyntaxKind::Newline, start);
        }

        // TextChunk runs until a delimiter, newline, or EOF.
        self.advance_char();
        while let Some(c) = self.peek_char() {
            if matches!(c, '@' | '(' | ')' | '{' | '}' | '[' | ']' | '\n' | '\r') {
                break;
            }
            self.advance_char();
        }
        self.finish(SyntaxKind::TextChunk, start)
    }

    fn bump_string(&mut self, start: usize) -> Token {
        self.advance_char(); // opening quote
        loop {
            match self.peek_char() {
                None => return self.finish(SyntaxKind::Error, start),
                Some('"') => {
                    self.advance_char();
                    return self.finish(SyntaxKind::String, start);
                }
                Some('\\') => {
                    self.advance_char();
                    if self.peek_char().is_some() {
                        self.advance_char();
                    }
                }
                Some(_) => {
                    self.advance_char();
                }
            }
        }
    }

    fn bump_ident_or_number(&mut self, start: usize) -> Token {
        // Leading `+` / `-` may start a number if a digit follows.
        if matches!(self.peek_char(), Some('+') | Some('-')) {
            let sign = self.peek_char();
            let next = self.peek_char_at(sign.unwrap().len_utf8());
            if next.is_some_and(|c| c.is_ascii_digit()) {
                return self.bump_number(start);
            }
            // DAT-001 / MAC-001: bare `->` is a reserved Arrow token, not Ident.
            if sign == Some('-') && next == Some('>') {
                self.advance_char(); // `-`
                self.advance_char(); // `>`
                return self.finish(SyntaxKind::Arrow, start);
            }
        }
        self.advance_char();
        while let Some(c) = self.peek_char() {
            if !is_ident_continue(c) {
                break;
            }
            self.advance_char();
        }
        self.finish(SyntaxKind::Ident, start)
    }

    fn bump_number(&mut self, start: usize) -> Token {
        if matches!(self.peek_char(), Some('+') | Some('-')) {
            self.advance_char();
        }
        let mut seen_digit = false;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                seen_digit = true;
                self.advance_char();
            } else if c == '.' && seen_digit {
                // Single fractional dot; further dots end the token and will
                // be lexed separately (often as Error or Ident later).
                let after = self.peek_char_at('.'.len_utf8());
                if after.is_some_and(|d| d.is_ascii_digit()) {
                    self.advance_char();
                    while let Some(d) = self.peek_char() {
                        if !d.is_ascii_digit() {
                            break;
                        }
                        self.advance_char();
                    }
                }
                break;
            } else {
                break;
            }
        }
        // Callers only enter `bump_number` with a digit or a signed digit, so
        // `seen_digit` is always true; keep the guard for API safety.
        debug_assert!(seen_digit);
        self.finish(SyntaxKind::Number, start)
    }

    fn finish(&self, kind: SyntaxKind, start: usize) -> Token {
        Token {
            kind,
            start,
            end: self.pos,
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn peek_char_at(&self, offset_from_pos: usize) -> Option<char> {
        let i = self.pos + offset_from_pos;
        if i >= self.input.len() {
            return None;
        }
        self.input[i..].chars().next()
    }

    fn advance_char(&mut self) {
        let c = self
            .peek_char()
            .expect("advance_char requires a pending character");
        self.pos += c.len_utf8();
    }
}

fn is_ident_start(c: char) -> bool {
    // Lisp-ish: letters, symbols common in Scheme-like dialects, underscore.
    // Digits alone start numbers; `+`/`-` handled in bump_ident_or_number.
    c.is_alphabetic()
        || matches!(
            c,
            '_' | '+' | '-' | '*' | '/' | '%' | '=' | '<' | '>' | '!' | '?' | '$'
        )
}

fn is_ident_continue(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}
