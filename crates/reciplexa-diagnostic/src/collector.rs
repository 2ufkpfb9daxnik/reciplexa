//! Collect diagnostics during compilation.

use crate::code::{DiagnosticCode, DiagnosticId};
use crate::kind::{Diagnostic, DiagnosticCategory, DiagnosticLifecycleStage};
use crate::message::DiagnosticMessage;
use crate::origin::DiagnosticOrigin;
use crate::severity::DiagnosticSeverity;

/// Accumulates structured diagnostics with monotonic instance ids.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticCollector {
    next_id: u64,
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            diagnostics: Vec::new(),
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity.is_error_or_worse())
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity.is_error_or_worse())
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| matches!(d.severity, DiagnosticSeverity::Warning))
            .count()
    }

    pub fn extend(&mut self, other: DiagnosticCollector) {
        for diagnostic in other.diagnostics {
            let id = DiagnosticId::new(self.next_id);
            self.next_id = self.next_id.saturating_add(1);
            let mut d = diagnostic;
            d.id = id;
            self.diagnostics.push(d);
        }
    }

    pub fn push(
        &mut self,
        code: DiagnosticCode,
        severity: DiagnosticSeverity,
        category: DiagnosticCategory,
        lifecycle_stage: DiagnosticLifecycleStage,
        message: DiagnosticMessage,
    ) -> DiagnosticId {
        let id = DiagnosticId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        let diagnostic = Diagnostic::new(id, code, severity, category, lifecycle_stage, message);
        self.diagnostics.push(diagnostic);
        id
    }

    pub fn push_with_origin(
        &mut self,
        code: DiagnosticCode,
        severity: DiagnosticSeverity,
        category: DiagnosticCategory,
        lifecycle_stage: DiagnosticLifecycleStage,
        message: DiagnosticMessage,
        origin: DiagnosticOrigin,
    ) -> DiagnosticId {
        let id = DiagnosticId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        let diagnostic = Diagnostic::new(id, code, severity, category, lifecycle_stage, message)
            .with_primary_origin(origin);
        self.diagnostics.push(diagnostic);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collector_assigns_unique_ids() {
        let mut c = DiagnosticCollector::new();
        let a = c.push(
            DiagnosticCode::new("compiler", "syntax", "SYN-0001"),
            DiagnosticSeverity::Error,
            DiagnosticCategory::Syntax,
            DiagnosticLifecycleStage::Parse,
            DiagnosticMessage::new("parse.error"),
        );
        let b = c.push(
            DiagnosticCode::new("compiler", "syntax", "SYN-0002"),
            DiagnosticSeverity::Warning,
            DiagnosticCategory::Syntax,
            DiagnosticLifecycleStage::Parse,
            DiagnosticMessage::new("parse.warn"),
        );
        assert_ne!(a, b);
        assert!(c.has_errors());
    }

    #[test]
    fn extend_reassigns_ids() {
        let mut a = DiagnosticCollector::new();
        a.push(
            DiagnosticCode::new("compiler", "syntax", "SYN-0001"),
            DiagnosticSeverity::Error,
            DiagnosticCategory::Syntax,
            DiagnosticLifecycleStage::Parse,
            DiagnosticMessage::new("a"),
        );
        let mut b = DiagnosticCollector::new();
        b.push(
            DiagnosticCode::new("compiler", "syntax", "SYN-0002"),
            DiagnosticSeverity::Warning,
            DiagnosticCategory::Syntax,
            DiagnosticLifecycleStage::Parse,
            DiagnosticMessage::new("b"),
        );
        a.extend(b);
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn collector_counts_and_push_with_origin() {
        let mut c = DiagnosticCollector::new();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
        c.push_with_origin(
            DiagnosticCode::new("compiler", "syntax", "SYN-0003"),
            DiagnosticSeverity::Warning,
            DiagnosticCategory::Syntax,
            DiagnosticLifecycleStage::Parse,
            DiagnosticMessage::new("warn"),
            DiagnosticOrigin::Unknown,
        );
        assert_eq!(c.len(), 1);
        assert_eq!(c.warning_count(), 1);
        assert_eq!(c.error_count(), 0);
        assert!(!c.has_errors());
        c.push(
            DiagnosticCode::new("compiler", "syntax", "SYN-0004"),
            DiagnosticSeverity::Fatal,
            DiagnosticCategory::Syntax,
            DiagnosticLifecycleStage::Parse,
            DiagnosticMessage::new("fatal"),
        );
        assert_eq!(c.error_count(), 1);
        assert!(c.has_errors());
        assert_eq!(c.diagnostics().len(), 2);
        let all = c.into_diagnostics();
        assert_eq!(all.len(), 2);
    }
}
