//! ERR-001 §28 — primary Failure diagnostic + suppressed cleanup failures.

use crate::code::DiagnosticCode;
use crate::kind::{Diagnostic, DiagnosticCategory, DiagnosticId, DiagnosticLifecycleStage};
use crate::message::DiagnosticMessage;
use crate::severity::DiagnosticSeverity;

/// Bundle pairing a primary failure diagnostic with suppressed cleanup diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureDiagnosticBundle {
    pub primary: Diagnostic,
    pub suppressed: Vec<Diagnostic>,
}

impl FailureDiagnosticBundle {
    pub fn new(primary: Diagnostic, suppressed: Vec<Diagnostic>) -> Self {
        Self {
            primary,
            suppressed,
        }
    }

    /// Render order: primary first, then suppressed cleanup failures.
    pub fn display_order(&self) -> impl Iterator<Item = &Diagnostic> {
        std::iter::once(&self.primary).chain(self.suppressed.iter())
    }
}

/// Build a primary runtime-failure diagnostic.
pub fn primary_failure_diagnostic(
    id: DiagnosticId,
    code: DiagnosticCode,
    message: impl Into<String>,
) -> Diagnostic {
    Diagnostic::new(
        id,
        code,
        DiagnosticSeverity::Error,
        DiagnosticCategory::Runtime,
        DiagnosticLifecycleStage::Evaluate,
        DiagnosticMessage::new(message),
    )
}

/// Build a suppressed cleanup-failure diagnostic (ERR-001 §28.2).
pub fn suppressed_cleanup_diagnostic(
    id: DiagnosticId,
    code: DiagnosticCode,
    message: impl Into<String>,
) -> Diagnostic {
    Diagnostic::new(
        id,
        code,
        DiagnosticSeverity::Warning,
        DiagnosticCategory::Runtime,
        DiagnosticLifecycleStage::Evaluate,
        DiagnosticMessage::new(format!(
            "Additional failure during cleanup: {}",
            message.into()
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_precedes_suppressed() {
        let primary = primary_failure_diagnostic(
            DiagnosticId(1),
            DiagnosticCode::new("err", "runtime", "PRIMARY"),
            "Document load failed: invalid header",
        );
        let suppressed = suppressed_cleanup_diagnostic(
            DiagnosticId(2),
            DiagnosticCode::new("err", "runtime", "CLEANUP"),
            "failed to close resource",
        );
        let bundle = FailureDiagnosticBundle::new(primary.clone(), vec![suppressed.clone()]);
        let order: Vec<_> = bundle.display_order().cloned().collect();
        assert_eq!(order[0].message.template_id, primary.message.template_id);
        assert!(order[1].message.template_id.contains("cleanup"));
        assert_eq!(order[1].severity, DiagnosticSeverity::Warning);
    }
}
