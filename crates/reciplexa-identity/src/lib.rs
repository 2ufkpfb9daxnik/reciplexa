//! Stable identity types used across compiler, document model, and GUI.
//!
//! Identities are opaque newtypes — array indices, line numbers, and memory
//! addresses must not be substituted for them (`implementation-agent-prompt.md`).

#![forbid(unsafe_code)]

pub mod binding;
pub mod document;
pub mod package;
pub mod syntax;
