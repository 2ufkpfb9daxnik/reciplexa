//! Lossless syntax substrate for reciplexa.
//!
//! This crate owns tokenization and parsing into a `rowan` CST.
//! It intentionally has no GUI, PDF, or typesetting dependencies so those
//! layers can be tested and swapped independently.

#![forbid(unsafe_code)]

pub mod kind;
pub mod lexer;
pub mod parse;

pub use kind::{SyntaxKind, SyntaxLanguage, SyntaxNode, SyntaxToken};
pub use lexer::{Lexer, LexerMode, Token};
pub use parse::{parse_source, unparse, Parse, ParseError};
