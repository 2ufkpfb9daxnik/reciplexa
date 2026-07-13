//! Lossless syntax substrate for reciplexa.
//!
//! This crate owns tokenization and (later) parsing into a `rowan` CST.
//! It intentionally has no GUI, PDF, or typesetting dependencies so those
//! layers can be tested and swapped independently.

#![forbid(unsafe_code)]

pub mod kind;
pub mod lexer;

pub use kind::{SyntaxKind, SyntaxLanguage};
pub use lexer::{Lexer, LexerMode, Token};
