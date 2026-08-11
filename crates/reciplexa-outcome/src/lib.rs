//! Typed outcomes separating failure, cancellation, and defect.
//!
//! See `specification.md` ERR-001 and CON-001 for the normative classification.

#![forbid(unsafe_code)]

pub mod cancellation;
pub mod classify;
pub mod defect;
pub mod failure;
pub mod job;
pub mod subject;

pub use cancellation::{CancellationReason, CancellationReport, CleanupStatus};
pub use classify::{
    classify_foreign_adapter, classify_resource_exhaustion, foreign_adapter_defect,
    resource_exhausted_failure, unhandled_failure_report, ForeignAdapterClass,
    ResourceExhaustionClass,
};
pub use defect::{DefectCode, DefectReport, DefectScope, RecoveryStatus, RuntimeTrustStatus};
pub use failure::{FailureCode, FailureReport};
pub use job::JobResult;
pub use subject::{ApplicationOutcome, SubjectOutcome};
