//! Job-level results with explicit defect path.

use crate::cancellation::CancellationReport;
use crate::defect::DefectReport;

/// Result of a fault-bounded job (`specification.md` ERR-001 §24).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobResult<A, E> {
    Completed(A),
    Failed(E),
    Cancelled(CancellationReport),
    Defected(Box<DefectReport>),
}

impl<A, E> JobResult<A, E> {
    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed(_))
    }

    pub fn is_defected(&self) -> bool {
        matches!(self, Self::Defected(_))
    }

    pub fn map<B>(self, f: impl FnOnce(A) -> B) -> JobResult<B, E> {
        match self {
            Self::Completed(a) => JobResult::Completed(f(a)),
            Self::Failed(e) => JobResult::Failed(e),
            Self::Cancelled(c) => JobResult::Cancelled(c),
            Self::Defected(d) => JobResult::Defected(d),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cancellation::{CancellationReason, CancellationReport};
    use crate::defect::{DefectCode, DefectReport, DefectScope};

    #[test]
    fn completed_job_is_not_defected() {
        let result: JobResult<(), ()> = JobResult::Completed(());
        assert!(result.is_completed());
        assert!(!result.is_defected());
    }

    #[test]
    fn defected_job_is_flagged() {
        let report = DefectReport::new(
            1,
            DefectCode::new("test", "X"),
            "invariant",
            DefectScope::Task,
            "subsystem",
            "msg",
        );
        let result: JobResult<(), ()> = JobResult::Defected(Box::new(report));
        assert!(result.is_defected());
    }

    #[test]
    fn cancelled_job_carries_report() {
        let report = CancellationReport::new(9, CancellationReason::Timeout);
        let result: JobResult<i32, ()> = JobResult::Cancelled(report.clone());
        assert!(matches!(result, JobResult::Cancelled(r) if r == report));
    }
}
