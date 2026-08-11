//! ERR-001 §22 case classification helpers (foreign adapter / resource exhaustion).
//!
//! These map detectable host/native conditions onto Failure vs Defect vs Terminal,
//! without requiring a full Part III fault-boundary runtime.

use crate::defect::{DefectCode, DefectReport, DefectScope};
use crate::failure::{FailureCode, FailureReport};

/// ERR-001 §22.9 — foreign adapter outcome class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForeignAdapterClass {
    /// Safe contract violation → Defect; stop adapter job.
    Defect,
    /// Memory safety / runtime integrity destruction → Terminal failure.
    Terminal,
}

/// Classify a foreign-adapter failure description.
///
/// Heuristic interim: integrity/safety keywords → Terminal; otherwise Defect.
pub fn classify_foreign_adapter(message: &str) -> ForeignAdapterClass {
    let lower = message.to_ascii_lowercase();
    if lower.contains("memory safety")
        || lower.contains("use-after-free")
        || lower.contains("runtime integrity")
        || lower.contains("abi corrupt")
        || lower.contains("heap corrupt")
    {
        ForeignAdapterClass::Terminal
    } else {
        ForeignAdapterClass::Defect
    }
}

/// Build a DefectReport for a quarantined foreign-adapter contract violation.
pub fn foreign_adapter_defect(defect_id: u64, adapter: &str, message: &str) -> DefectReport {
    DefectReport::new(
        defect_id,
        DefectCode::new("foreign", "ADAPTER_CONTRACT"),
        format!("foreign adapter `{adapter}` contract violation"),
        DefectScope::Task,
        "foreign-adapter",
        message,
    )
}

/// ERR-001 §22.10 / MEM-001 §24 — resource exhaustion outcome class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceExhaustionClass {
    /// Local quota / FD / GPU buffer → typed Failure (`resource-error`).
    Failure,
    /// Unrecoverable process heap OOM → Terminal failure.
    Terminal,
}

/// Classify a resource-exhaustion condition.
pub fn classify_resource_exhaustion(message: &str) -> ResourceExhaustionClass {
    let lower = message.to_ascii_lowercase();
    if lower.contains("heap oom")
        || lower.contains("out of memory")
        || lower.contains("unrecoverable")
        || lower.contains("process oom")
    {
        ResourceExhaustionClass::Terminal
    } else {
        ResourceExhaustionClass::Failure
    }
}

/// Build a typed FailureReport for local quota-style resource exhaustion.
pub fn resource_exhausted_failure(failure_id: u64, message: impl Into<String>) -> FailureReport {
    FailureReport::new(
        failure_id,
        FailureCode::new("runtime", "resource-error"),
        message,
    )
}

/// Build a FailureReport for an unhandled Failure effect at the eval host.
pub fn unhandled_failure_report(failure_id: u64, payload: impl Into<String>) -> FailureReport {
    FailureReport::new(
        failure_id,
        FailureCode::new("runtime", "unhandled-failure"),
        payload,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreign_adapter_contract_is_defect() {
        assert_eq!(
            classify_foreign_adapter("unknown op decode"),
            ForeignAdapterClass::Defect
        );
        let d = foreign_adapter_defect(1, "portable-image-decode", "unknown op");
        assert_eq!(d.code.code, "ADAPTER_CONTRACT");
    }

    #[test]
    fn foreign_adapter_integrity_is_terminal() {
        assert_eq!(
            classify_foreign_adapter("memory safety violation in adapter"),
            ForeignAdapterClass::Terminal
        );
    }

    #[test]
    fn resource_quota_is_failure_oom_is_terminal() {
        assert_eq!(
            classify_resource_exhaustion("GPU buffer limit exceeded"),
            ResourceExhaustionClass::Failure
        );
        assert_eq!(
            classify_resource_exhaustion("process heap OOM"),
            ResourceExhaustionClass::Terminal
        );
        let f = resource_exhausted_failure(3, "fd limit");
        assert_eq!(f.code.code, "resource-error");
    }
}
