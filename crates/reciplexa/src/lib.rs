//! Shared `.rpx` → scene pipeline used by the CLI and GUI.
//!
//! Stages stay separable so hosts can preview without side effects, then run
//! `(src …)` effects only when exporting.

#![forbid(unsafe_code)]

pub mod cli;
pub mod document_pipeline;
pub mod pipeline;

pub use pipeline::{
    document_for_export, document_from_source, document_from_source_with_snapshot, expand, lower,
    run_effects, PipelineDocument, PipelineError,
};
