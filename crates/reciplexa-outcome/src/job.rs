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

    pub fn map_err<F>(self, f: impl FnOnce(E) -> F) -> JobResult<A, F> {
        match self {
            Self::Completed(a) => JobResult::Completed(a),
            Self::Failed(e) => JobResult::Failed(f(e)),
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
    fn map_err_on_failed_job() {
        let result: JobResult<(), &str> = JobResult::Failed("err");
        let mapped = result.map_err(|e| e.len());
        assert!(matches!(mapped, JobResult::Failed(3)));
    }

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

    #[test]
    fn map_preserves_non_completed_variants() {
        let cancelled: JobResult<i32, ()> =
            JobResult::Cancelled(CancellationReport::new(1, CancellationReason::Timeout));
        assert!(matches!(cancelled.map(|n| n + 1), JobResult::Cancelled(_)));

        let defect = DefectReport::new(
            1,
            DefectCode::new("test", "X"),
            "inv",
            DefectScope::Task,
            "sub",
            "msg",
        );
        let defected: JobResult<(), ()> = JobResult::Defected(Box::new(defect));
        assert!(defected.is_defected());
        assert!(matches!(defected.map(|()| 1), JobResult::Defected(_)));

        let completed: JobResult<i32, ()> = JobResult::Completed(2);
        assert_eq!(completed.map(|n| n * 3), JobResult::Completed(6));
    }

    #[test]
    fn map_err_on_all_variants() {
        let failed: JobResult<(), &str> = JobResult::Failed("ab");
        assert_eq!(failed.map_err(|e| e.len()), JobResult::Failed(2));

        let cancelled: JobResult<i32, &str> =
            JobResult::Cancelled(CancellationReport::new(1, CancellationReason::Timeout));
        assert!(matches!(
            cancelled.map_err(|e| e.len()),
            JobResult::Cancelled(_)
        ));

        let defect = DefectReport::new(
            1,
            DefectCode::new("test", "X"),
            "inv",
            DefectScope::Task,
            "sub",
            "msg",
        );
        let defected: JobResult<(), &str> = JobResult::Defected(Box::new(defect));
        assert!(matches!(
            defected.map_err(|e| e.len()),
            JobResult::Defected(_)
        ));
    }

    #[test]
    fn failed_job_is_not_completed() {
        let result: JobResult<(), &str> = JobResult::Failed("err");
        assert!(!result.is_completed());
        assert!(!result.is_defected());
    }

    #[test]
    fn map_and_map_err_on_failed_and_cancelled() {
        let failed: JobResult<i32, &str> = JobResult::Failed("x");
        assert!(matches!(failed.map(|n| n + 1), JobResult::Failed("x")));

        let failed: JobResult<i32, &str> = JobResult::Failed("x");
        assert_eq!(failed.map_err(|e| e.len()), JobResult::Failed(1));

        let cancelled: JobResult<i32, &str> =
            JobResult::Cancelled(CancellationReport::new(2, CancellationReason::Timeout));
        assert!(matches!(cancelled.map(|n| n + 1), JobResult::Cancelled(_)));
    }

    #[test]
    fn map_err_on_completed_preserves_value() {
        let completed: JobResult<i32, &str> = JobResult::Completed(7);
        assert_eq!(completed.map_err(|e| e.len()), JobResult::Completed(7));
        assert!(!JobResult::<(), ()>::Failed(()).is_completed());
    }

    #[test]
    fn job_result_debug_equality() {
        let a: JobResult<i32, &str> = JobResult::Completed(1);
        let b: JobResult<i32, &str> = JobResult::Completed(1);
        assert_eq!(a, b);
        let _ = format!("{:?}", JobResult::<(), &str>::Failed("e"));
    }
}
