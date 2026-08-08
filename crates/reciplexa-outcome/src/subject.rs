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
    fn subject_success_is_distinct_from_test_pass() {
        let subject: SubjectOutcome<i32, ()> = SubjectOutcome::Success(42);
        assert!(subject.is_success());
    }

    #[test]
    fn application_completed_is_success_path() {
        let outcome: ApplicationOutcome<(), ()> = ApplicationOutcome::Completed(());
        assert!(matches!(outcome, ApplicationOutcome::Completed(())));
    }
}
