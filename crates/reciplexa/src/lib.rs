//! Shared `.rpx` → scene pipeline used by the CLI and GUI.
//!
//! Stages stay separable so hosts can preview without side effects, then run
//! `(src …)` effects only when exporting.

#![forbid(unsafe_code)]

pub mod cli;
pub mod pipeline;

pub use pipeline::{document_from_source, expand, lower, run_effects, typecheck, PipelineError};
