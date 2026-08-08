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
