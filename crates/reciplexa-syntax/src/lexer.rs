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

        // SYN §1.2: UTF-8 BOM at byte offset 0 is lossless trivia (no semantic effect).
        if start == 0 && self.input.starts_with('\u{feff}') {
            self.advance_char();
            return self.finish(SyntaxKind::Bom, start);
        }

        // SYN §1.5: shebang at byte offset 0 is lossless trivia (no semantic effect).
        if start == 0 && self.input.starts_with("#!") {
            while let Some(ch) = self.peek_char() {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                self.advance_char();
            }
            return self.finish(SyntaxKind::Shebang, start);
        }

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
            '.' => return self.bump_ellipsis(start),
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
        // SYN §8.2: `"` opens a short string; `"""`+ is a variable-length delimiter.
        // `""` is the empty string (delimiter length 2 is reserved for that).
        let mut open = 0usize;
        while self.peek_char() == Some('"') {
            self.advance_char();
            open += 1;
        }
        if open == 0 {
            return self.finish(SyntaxKind::Error, start);
        }
        if open == 2 {
            return self.finish(SyntaxKind::String, start);
        }
        if open == 1 {
            // Short one-line string: first `"` terminates (§8.1: no escapes).
            // Unclosed at newline/EOF becomes a partial String (§18.4 virtual close).
            loop {
                match self.peek_char() {
                    None => return self.finish(SyntaxKind::String, start),
                    Some('"') => {
                        self.advance_char();
                        return self.finish(SyntaxKind::String, start);
                    }
                    Some('\n') | Some('\r') => {
                        return self.finish(SyntaxKind::String, start);
                    }
                    Some(_) => {
                        self.advance_char();
                    }
                }
            }
        }
        // Multi-quote (n ≥ 3): close on the first run of ≥ n quotes (consume n).
        loop {
            match self.peek_char() {
                None => return self.finish(SyntaxKind::String, start),
                Some('"') => {
                    let mut n = 0usize;
                    while self.peek_char() == Some('"') {
                        self.advance_char();
                        n += 1;
                    }
                    if n >= open {
                        // Extra quotes beyond `open` remain for the next token.
                        let extra = n - open;
                        if extra > 0 {
                            self.pos -= extra; // rewind leftover `"` bytes (ASCII)
                        }
                        return self.finish(SyntaxKind::String, start);
                    }
                    // Fewer than `open`: already consumed as content.
                }
                Some(_) => {
                    self.advance_char();
                }
            }
        }
    }

    /// MAC-001: `...` (template splice) and `...+` (pattern rest) as Ident atoms.
    fn bump_ellipsis(&mut self, start: usize) -> Token {
        // Require three dots; optional trailing `+` for pattern rest.
        self.advance_char(); // first `.`
        if self.peek_char() != Some('.') || self.peek_char_at('.'.len_utf8()) != Some('.') {
            return self.finish(SyntaxKind::Error, start);
        }
        self.advance_char();
        self.advance_char();
        if self.peek_char() == Some('+') {
            self.advance_char();
        }
        self.finish(SyntaxKind::Ident, start)
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

        // SYN §5: freestanding operators are the fixed list only (no `++`, `<>`, …).
        if let Some(tok) = self.try_bump_operator(start) {
            return tok;
        }

        // SYN §3.5: bare `_` is the wildcard; `_foo` is rejected.
        if self.peek_char() == Some('_') {
            self.advance_char();
            if self.peek_char().is_some_and(is_ident_continue) {
                while self.peek_char().is_some_and(is_ident_continue) {
                    self.advance_char();
                }
                return self.finish(SyntaxKind::Error, start);
            }
            return self.finish(SyntaxKind::Ident, start);
        }

        self.advance_char();
        while let Some(c) = self.peek_char() {
            if !is_ident_continue(c) {
                break;
            }
            self.advance_char();
        }
        // Optional single trailing `?` / `!` (SYN §3.6).
        if matches!(self.peek_char(), Some('?') | Some('!')) {
            let mark = self.peek_char().unwrap();
            let after = self.peek_char_at(mark.len_utf8());
            if !after.is_some_and(is_ident_continue) && !matches!(after, Some('?') | Some('!')) {
                self.advance_char();
            }
        }

        let text = &self.input[start..self.pos];
        // Underscore-in-name and leading/trailing/double `-` are hard lex errors.
        // Uppercase-start is left as Ident so elaborate/resolve can diagnose clearly.
        if text.contains('_')
            || (text.starts_with('-') && text != "-")
            || text.ends_with('-')
            || text.contains("--")
        {
            return Token {
                kind: SyntaxKind::Error,
                start,
                end: self.pos,
            };
        }
        self.finish(SyntaxKind::Ident, start)
    }

    /// Match a SYN §5 operator as a single Ident; does not glue into letters.
    fn try_bump_operator(&mut self, start: usize) -> Option<Token> {
        let c = self.peek_char()?;
        let two = match c {
            '<' | '>' | '!' => {
                let n = self.peek_char_at(c.len_utf8());
                match (c, n) {
                    ('<', Some('=')) => Some("<="),
                    ('>', Some('=')) => Some(">="),
                    ('!', Some('=')) => Some("!="),
                    _ => None,
                }
            }
            _ => None,
        };
        if let Some(op) = two {
            // Ensure not followed by ident-continue (operators are freestanding).
            let len = op.len();
            let after = self.peek_char_at(len);
            if after.is_some_and(|ch| is_ident_continue(ch) || ch == '?' || ch == '!') {
                return None;
            }
            for _ in 0..len {
                self.advance_char();
            }
            return Some(self.finish(SyntaxKind::Ident, start));
        }

        if matches!(c, '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!') {
            let after = self.peek_char_at(c.len_utf8());
            // Structured comments use Ident `//` (SYN); keep as one token.
            if c == '/' && after == Some('/') {
                self.advance_char();
                self.advance_char();
                return Some(self.finish(SyntaxKind::Ident, start));
            }
            // Freestanding `/` is division (SYN §5); path segments are separate Idents.
            // `-` before a letter is leading-hyphen reject (fall through).
            if c == '-' && after.is_some_and(|ch| ch.is_alphabetic()) {
                return None;
            }
            // Other ops are always freestanding — do not glue into `+x`.
            self.advance_char();
            return Some(self.finish(SyntaxKind::Ident, start));
        }
        None
    }

    /// SYN §7 numeric literals: decimal / `0b`/`0o`/`0x` / `_` / scientific `e`.
    ///
    /// Unit suffixes (§12) are **not** glued: `40mm` → Number `40` + Ident `mm`
    /// so package apply can treat the suffix as a constructor name.
    fn bump_number(&mut self, start: usize) -> Token {
        if matches!(self.peek_char(), Some('+') | Some('-')) {
            self.advance_char();
        }

        // Radix forms: 0b… / 0o… / 0x…
        if self.peek_char() == Some('0') {
            let next = self.peek_char_at('0'.len_utf8());
            if matches!(next, Some('b') | Some('o') | Some('x')) {
                self.advance_char(); // 0
                let radix_ch = self.peek_char().expect("checked");
                self.advance_char(); // b|o|x
                let radix = match radix_ch {
                    'b' => 2,
                    'o' => 8,
                    'x' => 16,
                    _ => unreachable!(),
                };
                if !self.consume_digits_with_sep(|c| c.is_digit(radix)) {
                    return self.finish(SyntaxKind::Error, start);
                }
                return self.finish(SyntaxKind::Number, start);
            }
        }

        // Decimal integer / float / scientific.
        if !self.consume_decimal_number() {
            // Consume a maximal digit/sep/dot/e run so `007` is one Error token.
            while let Some(c) = self.peek_char() {
                if c.is_ascii_digit()
                    || c == '_'
                    || c == '.'
                    || c == 'e'
                    || ((c == '+' || c == '-')
                        && self
                            .input
                            .get(start..self.pos)
                            .is_some_and(|s| s.ends_with('e')))
                {
                    self.advance_char();
                } else {
                    break;
                }
            }
            return self.finish(SyntaxKind::Error, start);
        }
        self.finish(SyntaxKind::Number, start)
    }

    /// Consume one-or-more digits with optional `_` separators between digits.
    /// Returns false if no digit was consumed or separators are illegal.
    fn consume_digits_with_sep(&mut self, is_digit: impl Fn(char) -> bool) -> bool {
        let mut saw_digit = false;
        let mut prev_underscore = false;
        loop {
            match self.peek_char() {
                Some('_') => {
                    if !saw_digit || prev_underscore {
                        return false;
                    }
                    // Trailing `_` before non-digit end is illegal — peek ahead.
                    let after = self.peek_char_at('_'.len_utf8());
                    if !after.is_some_and(&is_digit) {
                        return false;
                    }
                    self.advance_char();
                    prev_underscore = true;
                }
                Some(c) if is_digit(c) => {
                    self.advance_char();
                    saw_digit = true;
                    prev_underscore = false;
                }
                _ => break,
            }
        }
        saw_digit && !prev_underscore
    }

    /// Decimal mantissa + optional fraction + optional `e` exponent (SYN §7.5).
    /// Also enforces no unnecessary leading zeros on the integer part (§7.2).
    fn consume_decimal_number(&mut self) -> bool {
        let int_start = self.pos;
        if !self.consume_digits_with_sep(|c| c.is_ascii_digit()) {
            return false;
        }
        let int_text = &self.input[int_start..self.pos];
        let int_digits: String = int_text.chars().filter(|c| c.is_ascii_digit()).collect();
        if int_digits.len() > 1 && int_digits.starts_with('0') {
            return false;
        }

        // Fraction: `.` + digits (both sides required — `1.` / `.5` rejected).
        if self.peek_char() == Some('.') {
            let after = self.peek_char_at('.'.len_utf8());
            if !after.is_some_and(|d| d.is_ascii_digit()) {
                // Leave `.` for the next token (often Error / ellipsis).
                return true;
            }
            self.advance_char();
            if !self.consume_digits_with_sep(|c| c.is_ascii_digit()) {
                return false;
            }
        }

        // Exponent: `e` [sign] digits (lowercase `e` only).
        if self.peek_char() == Some('e') {
            self.advance_char();
            if matches!(self.peek_char(), Some('+') | Some('-')) {
                self.advance_char();
            }
            if !self.consume_digits_with_sep(|c| c.is_ascii_digit()) {
                return false;
            }
        }
        true
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
    // Letters, `$` (macro params), `_` (wildcard), and SYN §5 operator starters.
    c.is_alphabetic()
        || matches!(
            c,
            '_' | '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '?' | '$'
        )
}

fn is_ident_continue(c: char) -> bool {
    // Include `_` so `report_title` is one Error token (SYN §3.5), not three.
    // Kebab `-` only; `/` is path separator / division op (SYN §4–5), not continue.
    c.is_alphabetic() || c.is_ascii_digit() || matches!(c, '-' | '$' | '_')
}
