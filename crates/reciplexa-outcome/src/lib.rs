//! Typed outcomes separating failure, cancellation, and defect.
//!
//! See `specification.md` ERR-001 and CON-001 for the normative classification.

#![forbid(unsafe_code)]

pub mod cancellation;
pub mod defect;
pub mod job;
pub mod subject;

pub use cancellation::{CancellationReason, CancellationReport, CleanupStatus};
pub use defect::{DefectCode, DefectReport, DefectScope, RecoveryStatus, RuntimeTrustStatus};
pub use job::JobResult;
pub use subject::{ApplicationOutcome, SubjectOutcome};
