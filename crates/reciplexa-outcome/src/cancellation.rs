//! Cancellation reports (`specification.md` CON-001 §4).

use core::fmt;

use reciplexa_identity::document::DocumentIdentity;

/// Why a scope was cancelled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancellationReason {
    UserRequested,
    HostShutdown,
    Timeout,
    Superseded,
    Custom(String),
}

/// Status of structured unwind / cleanup after cancellation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupStatus {
    Pending,
    Completed,
    Partial,
    Failed,
}

/// Structured cancellation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancellationReport {
    pub cancellation_id: u64,
    pub reason: CancellationReason,
    pub requested_by: Option<String>,
    pub affected_scope: Option<String>,
    pub cleanup_status: CleanupStatus,
    pub document_id: Option<DocumentIdentity>,
}

impl CancellationReport {
    pub fn new(cancellation_id: u64, reason: CancellationReason) -> Self {
        Self {
            cancellation_id,
            reason,
            requested_by: None,
            affected_scope: None,
            cleanup_status: CleanupStatus::Pending,
            document_id: None,
        }
    }
}

impl fmt::Display for CancellationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cancellation:{} ({:?}, cleanup={:?})",
            self.cancellation_id, self.reason, self.cleanup_status
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_formats_without_panic() {
        let report = CancellationReport::new(1, CancellationReason::UserRequested);
        assert!(report.to_string().contains("cancellation:1"));
    }
}
