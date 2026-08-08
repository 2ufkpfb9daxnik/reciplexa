//! Application and subject outcomes.

use crate::cancellation::CancellationReport;
use crate::defect::DefectReport;

/// Outcome of the program under test, separate from test harness outcome
/// (`specification.md` TEST-001 §40).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectOutcome<A, E> {
    Success(A),
    Failure(E),
    Cancelled,
    Defect(Box<DefectReport>),
}

impl<A, E> SubjectOutcome<A, E> {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failure(_))
    }

    pub fn map<B>(self, f: impl FnOnce(A) -> B) -> SubjectOutcome<B, E> {
        match self {
            Self::Success(a) => SubjectOutcome::Success(f(a)),
            Self::Failure(e) => SubjectOutcome::Failure(e),
            Self::Cancelled => SubjectOutcome::Cancelled,
            Self::Defect(d) => SubjectOutcome::Defect(d),
        }
    }

    pub fn map_err<F>(self, f: impl FnOnce(E) -> F) -> SubjectOutcome<A, F> {
        match self {
            Self::Success(a) => SubjectOutcome::Success(a),
            Self::Failure(e) => SubjectOutcome::Failure(f(e)),
            Self::Cancelled => SubjectOutcome::Cancelled,
            Self::Defect(d) => SubjectOutcome::Defect(d),
        }
    }
}

/// Root scope outcome after shutdown (`specification.md` CON-001 §61).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationOutcome<A, E> {
    Completed(A),
    Requested(ExitIntent),
    Failed(E),
    Cancelled(CancellationReport),
    Defected(Box<DefectReport>),
    Aborted(InfrastructureAbort),
}

impl<A, E> ApplicationOutcome<A, E> {
    pub fn is_defected(&self) -> bool {
        matches!(self, Self::Defected(_))
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed(_))
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled(_))
    }
}

/// User-visible exit intent — not an OS exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitIntent {
    pub category: ExitCategory,
    pub user_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCategory {
    Success,
    NoChanges,
    UserCancelledOperation,
    RestartRequested,
    Custom,
}

/// Infrastructure could not determine a trustworthy result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfrastructureAbort {
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_map_preserves_failure() {
        let outcome: SubjectOutcome<i32, &str> = SubjectOutcome::Failure("err");
        let mapped = outcome.map_err(|e| e.len());
        assert!(matches!(mapped, SubjectOutcome::Failure(3)));
    }

    #[test]
    fn subject_success_is_distinct_from_test_pass() {
        let subject: SubjectOutcome<i32, ()> = SubjectOutcome::Success(42);
        assert!(subject.is_success());
    }

    #[test]
    fn application_completed_is_success_path() {
        let outcome: ApplicationOutcome<(), ()> = ApplicationOutcome::Completed(());
        assert!(matches!(outcome, ApplicationOutcome::Completed(())));
    }

    #[test]
    fn subject_cancelled_and_defect_branches() {
        use crate::defect::{DefectCode, DefectReport, DefectScope};

        let cancelled: SubjectOutcome<i32, ()> = SubjectOutcome::Cancelled;
        assert!(!cancelled.is_success());
        assert!(!cancelled.is_failure());

        let defect = DefectReport::new(
            1,
            DefectCode::new("test", "X"),
            "broken",
            DefectScope::Task,
            "sub",
            "safe",
        );
        let defected: SubjectOutcome<(), ()> = SubjectOutcome::Defect(Box::new(defect));
        assert!(!defected.is_success());

        let mapped: SubjectOutcome<i32, ()> = SubjectOutcome::Success(7).map(|n| n * 2);
        assert!(matches!(mapped, SubjectOutcome::Success(14)));
    }

    #[test]
    fn application_outcome_variants_are_distinct() {
        use crate::cancellation::{CancellationReason, CancellationReport};
        use crate::defect::{DefectCode, DefectReport, DefectScope};

        let cancelled: ApplicationOutcome<(), ()> =
            ApplicationOutcome::Cancelled(CancellationReport::new(1, CancellationReason::Timeout));
        assert!(cancelled.is_cancelled());

        let defected: ApplicationOutcome<(), ()> =
            ApplicationOutcome::Defected(Box::new(DefectReport::new(
                1,
                DefectCode::new("t", "D"),
                "inv",
                DefectScope::Process,
                "s",
                "m",
            )));
        assert!(defected.is_defected());

        let requested: ApplicationOutcome<(), ()> = ApplicationOutcome::Requested(ExitIntent {
            category: ExitCategory::UserCancelledOperation,
            user_message: Some("bye".into()),
        });
        assert!(!requested.is_completed());

        let aborted: ApplicationOutcome<(), ()> =
            ApplicationOutcome::Aborted(InfrastructureAbort {
                reason: "host lost".into(),
            });
        assert!(!aborted.is_cancelled());
    }

    #[test]
    fn subject_failure_and_map_branches() {
        let failure: SubjectOutcome<(), &str> = SubjectOutcome::Failure("err");
        assert!(failure.is_failure());
        assert!(!failure.is_success());
        assert!(matches!(
            failure.map_err(|e| e.len()),
            SubjectOutcome::Failure(3)
        ));

        let success: SubjectOutcome<i32, ()> = SubjectOutcome::Success(5);
        assert!(matches!(success.map(|n| n + 1), SubjectOutcome::Success(6)));

        let cancelled: SubjectOutcome<i32, ()> = SubjectOutcome::Cancelled;
        assert!(matches!(
            cancelled.map(|n| n + 1),
            SubjectOutcome::Cancelled
        ));
    }

    #[test]
    fn application_failed_and_completed_flags() {
        let failed: ApplicationOutcome<(), &str> = ApplicationOutcome::Failed("boom");
        assert!(!failed.is_completed());
        assert!(!failed.is_cancelled());
        assert!(!failed.is_defected());

        let completed: ApplicationOutcome<i32, ()> = ApplicationOutcome::Completed(1);
        assert!(completed.is_completed());
    }

    #[test]
    fn subject_defect_map_and_map_err() {
        use crate::defect::{DefectCode, DefectReport, DefectScope};

        let defect = DefectReport::new(
            1,
            DefectCode::new("test", "D"),
            "inv",
            DefectScope::Task,
            "sub",
            "msg",
        );
        let defected: SubjectOutcome<i32, ()> = SubjectOutcome::Defect(Box::new(defect));
        assert!(matches!(
            defected.clone().map(|n| n + 1),
            SubjectOutcome::Defect(_)
        ));
        assert!(matches!(
            defected.map_err(|()| "x"),
            SubjectOutcome::Defect(_)
        ));
    }

    #[test]
    fn application_outcome_map_err_on_failed() {
        let failed: ApplicationOutcome<(), &str> = ApplicationOutcome::Failed("boom");
        // ApplicationOutcome has no map_err — exercise Failed branch via match.
        assert!(!failed.is_completed());
        assert!(!failed.is_cancelled());
    }

    #[test]
    fn subject_failure_map_preserves_cancelled() {
        let cancelled: SubjectOutcome<i32, &str> = SubjectOutcome::Cancelled;
        assert!(matches!(
            cancelled.map_err(|e| e.len()),
            SubjectOutcome::Cancelled
        ));
    }

    #[test]
    fn subject_map_on_failure_and_defect() {
        let failure: SubjectOutcome<i32, &str> = SubjectOutcome::Failure("x");
        assert!(matches!(
            failure.map(|n| n + 1),
            SubjectOutcome::Failure("x")
        ));

        use crate::defect::{DefectCode, DefectReport, DefectScope};
        let defect = DefectReport::new(
            2,
            DefectCode::new("t", "D"),
            "inv",
            DefectScope::Process,
            "s",
            "m",
        );
        let defected: SubjectOutcome<i32, ()> = SubjectOutcome::Defect(Box::new(defect));
        assert!(matches!(defected.map(|n| n + 1), SubjectOutcome::Defect(_)));
    }

    #[test]
    fn application_requested_and_aborted_flags() {
        let requested: ApplicationOutcome<(), ()> = ApplicationOutcome::Requested(ExitIntent {
            category: ExitCategory::Success,
            user_message: None,
        });
        assert!(!requested.is_completed());
        assert!(!requested.is_defected());

        let aborted: ApplicationOutcome<(), ()> =
            ApplicationOutcome::Aborted(InfrastructureAbort {
                reason: "disk".into(),
            });
        assert!(!aborted.is_cancelled());
    }

    #[test]
    fn exit_category_equivalence_partition() {
        for cat in [
            ExitCategory::Success,
            ExitCategory::NoChanges,
            ExitCategory::UserCancelledOperation,
            ExitCategory::RestartRequested,
            ExitCategory::Custom,
        ] {
            let intent = ExitIntent {
                category: cat,
                user_message: None,
            };
            let outcome: ApplicationOutcome<(), ()> = ApplicationOutcome::Requested(intent);
            assert!(!outcome.is_completed());
            let _ = format!("{:?}", cat);
        }
    }

    #[test]
    fn subject_map_err_on_success() {
        let success: SubjectOutcome<i32, &str> = SubjectOutcome::Success(3);
        assert!(matches!(
            success.map_err(|e| e.len()),
            SubjectOutcome::Success(3)
        ));
    }

    #[test]
    fn infrastructure_abort_debug_and_eq() {
        let a = InfrastructureAbort { reason: "x".into() };
        let b = InfrastructureAbort { reason: "x".into() };
        assert_eq!(a, b);
        assert!(format!("{:?}", a).contains("x"));
    }
}
