//! Lossless syntax substrate for reciplexa.
//!
//! This crate owns tokenization and parsing into a `rowan` CST.
//! It intentionally has no GUI, PDF, or typesetting dependencies so those
//! layers can be tested and swapped independently.

#![forbid(unsafe_code)]

pub mod edit;
pub mod ident;
pub mod identity;
pub mod kind;
pub mod lexer;
pub mod markup;
pub mod number_lit;
pub mod parse;
pub mod reserved;
pub mod string_lit;

pub use edit::{format_drag_number, replace_token_text, token_at_offset};
pub use ident::{
    coalesce_slash_paths, is_operator_ident, is_wildcard_ident, normalize_ident, validate_ident,
    SlashAtom,
};
pub use identity::{build_identity_map, preserve_identity_on_reparse, SyntaxIdentityMap};
pub use kind::{SyntaxElement, SyntaxKind, SyntaxLanguage, SyntaxNode, SyntaxToken};
pub use lexer::{Lexer, LexerMode, Token};
pub use markup::{flatten_lines, flatten_readable, markup_parts, MarkupPart, MarkupWalkError};
pub use number_lit::parse_number_literal;
pub use parse::{parse_source, unparse, Parse, ParseError};
pub use reserved::is_reserved_special_form;
pub use string_lit::{decode_string_literal, encode_string_literal};
