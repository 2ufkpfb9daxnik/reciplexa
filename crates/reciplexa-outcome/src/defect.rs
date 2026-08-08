//! Defect reports (`specification.md` ERR-001 §23, CON-001 §6).

use core::fmt;

use reciplexa_identity::document::DocumentIdentity;
use reciplexa_source::range::TextRange;

/// Scope where a defect was observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefectScope {
    Task,
    Service,
    RootScope,
    RuntimeInstance,
    WorkerProcess,
    Process,
}

/// Stable defect classification code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefectCode {
    pub namespace: String,
    pub code: String,
}

impl DefectCode {
    pub fn new(namespace: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            code: code.into(),
        }
    }
}

/// Whether the runtime can continue trusting its internal bookkeeping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTrustStatus {
    Trusted,
    Degraded,
    Untrusted,
}

/// Whether recovery actions completed after a defect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStatus {
    NotAttempted,
    Isolated,
    ShutdownRequired,
}

/// Minimal origin attached to a defect report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefectOrigin {
    pub subsystem: String,
    pub operation: Option<String>,
    pub source_range: Option<TextRange>,
    pub document_id: Option<DocumentIdentity>,
}

/// Structured defect report — not a plain error string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefectReport {
    pub defect_id: u64,
    pub code: DefectCode,
    pub violated_invariant: String,
    pub scope: DefectScope,
    pub origin: DefectOrigin,
    pub recovery_status: RecoveryStatus,
    pub runtime_trust_status: RuntimeTrustStatus,
    pub safe_message: String,
    pub suppressed_cleanup_failures: Vec<String>,
}

impl DefectReport {
    pub fn new(
        defect_id: u64,
        code: DefectCode,
        violated_invariant: impl Into<String>,
        scope: DefectScope,
        subsystem: impl Into<String>,
        safe_message: impl Into<String>,
    ) -> Self {
        Self {
            defect_id,
            code,
            violated_invariant: violated_invariant.into(),
            scope,
            origin: DefectOrigin {
                subsystem: subsystem.into(),
                operation: None,
                source_range: None,
                document_id: None,
            },
            recovery_status: RecoveryStatus::NotAttempted,
            runtime_trust_status: RuntimeTrustStatus::Trusted,
            safe_message: safe_message.into(),
            suppressed_cleanup_failures: Vec::new(),
        }
    }
}

impl fmt::Display for DefectReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "defect:{} [{}] {}",
            self.defect_id, self.code.namespace, self.safe_message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defect_report_is_not_empty_string_only() {
        let report = DefectReport::new(
            1,
            DefectCode::new("runtime", "DOUBLE_RESUME"),
            "continuation resumed twice",
            DefectScope::Task,
            "effect-runtime",
            "internal effect handler error",
        );
        assert_eq!(report.violated_invariant, "continuation resumed twice");
        assert!(report.to_string().contains("defect:1"));
    }
}
