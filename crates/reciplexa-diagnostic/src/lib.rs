//! Machine-readable diagnostics (`specification.md` DIAG-001).
//!
//! Display strings are derived from structured message templates — they are
//! not the canonical diagnostic representation.

#![forbid(unsafe_code)]

pub mod code;
pub mod collector;
pub mod failure_bundle;
pub mod kind;
pub mod message;
pub mod origin;
pub mod parse_adapter;
pub mod render;
pub mod severity;

#[cfg(feature = "syntax")]
pub mod syntax_bridge;

pub use code::DiagnosticCode;
pub use collector::DiagnosticCollector;
pub use failure_bundle::{
    primary_failure_diagnostic, suppressed_cleanup_diagnostic, FailureDiagnosticBundle,
};
pub use kind::{Diagnostic, DiagnosticCategory, DiagnosticId, DiagnosticLifecycleStage};
pub use message::DiagnosticMessage;
pub use origin::{DiagnosticOrigin, SourceOrigin};
pub use parse_adapter::{push_parse_errors, ParseDiagnosticInput};
pub use render::{render_diagnostic_line, render_diagnostics};
pub use severity::DiagnosticSeverity;
#[cfg(feature = "syntax")]
pub use syntax_bridge::push_syntax_parse_errors;
