//! Test outcome helpers separating subject from harness.

use reciplexa_outcome::defect::DefectReport;
use reciplexa_outcome::subject::SubjectOutcome;

/// Overall test harness result (`specification.md` TEST-001 §39).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestOutcome<A> {
    Passed(A),
    Failed(String),
    Defected(Box<DefectReport>),
}

impl<A> TestOutcome<A> {
    pub fn is_passed(&self) -> bool {
        matches!(self, Self::Passed(_))
    }
}

/// Wrapper pairing a conformance id with subject execution.
#[derive(Debug, Clone)]
pub struct TestSubject {
    pub conformance_id: crate::conformance::ConformanceId,
    pub spec_section: crate::conformance::SpecSection,
}

impl TestSubject {
    pub fn new(conformance_id: impl Into<String>, spec_section: impl Into<String>) -> Self {
        Self {
            conformance_id: crate::conformance::ConformanceId::new(conformance_id),
            spec_section: crate::conformance::SpecSection::new(spec_section),
        }
    }

    pub fn label(&self) -> String {
        format!("{} {}", self.conformance_id, self.spec_section)
    }
}

/// Assert the subject succeeded.
pub fn assert_subject_success<E, A: PartialEq + std::fmt::Debug>(
    outcome: SubjectOutcome<A, E>,
    expected: A,
) {
    match outcome {
        SubjectOutcome::Success(value) => assert_eq!(value, expected),
        other => panic!("expected SubjectOutcome::Success, got {other:?}"),
    }
}

/// Assert the subject failed with the expected error payload.
pub fn assert_subject_failure<E: PartialEq + std::fmt::Debug, A: std::fmt::Debug>(
    outcome: SubjectOutcome<A, E>,
    expected: E,
) {
    match outcome {
        SubjectOutcome::Failure(err) => assert_eq!(err, expected),
        other => panic!("expected SubjectOutcome::Failure, got {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_subject_failure_matches() {
        assert_subject_failure::<&str, ()>(SubjectOutcome::Failure("err"), "err");
    }

    #[test]
    #[should_panic]
    fn assert_subject_failure_rejects_success() {
        assert_subject_failure::<&str, i32>(SubjectOutcome::Success(1), "err");
    }
}
