//! Lossless syntax substrate for reciplexa.
//!
//! This crate owns tokenization and parsing into a `rowan` CST.
//! It intentionally has no GUI, PDF, or typesetting dependencies so those
//! layers can be tested and swapped independently.

#![forbid(unsafe_code)]

pub mod doc;
pub mod edit;
pub mod kind;
pub mod lexer;
pub mod parse;

pub use doc::{doc_parts, flatten_lines, flatten_readable, DocPart, DocWalkError};
pub use edit::{format_drag_number, replace_token_text, token_at_offset};
pub use kind::{SyntaxElement, SyntaxKind, SyntaxLanguage, SyntaxNode, SyntaxToken};
pub use lexer::{Lexer, LexerMode, Token};
pub use parse::{parse_source, unparse, Parse, ParseError};
