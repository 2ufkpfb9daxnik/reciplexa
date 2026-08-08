//! Plain-text diagnostic rendering for CLI and tests.

use crate::kind::Diagnostic;
use reciplexa_source::line_index::LineIndex;

/// Render a diagnostic as a single human-readable line.
pub fn render_diagnostic_line(index: &LineIndex, diagnostic: &Diagnostic) -> String {
    let severity = diagnostic.severity.to_string();
    let code = diagnostic.code.as_path();
    let message = diagnostic.message.fallback_summary();
    let location = diagnostic
        .primary_origin
        .as_ref()
        .and_then(|origin| match origin {
            crate::origin::DiagnosticOrigin::Source(src) => {
                let pos = index.position(src.text_range.start());
                Some(format!("{}:{}", pos.line.get(), pos.column.get()))
            }
            crate::origin::DiagnosticOrigin::Unknown => None,
        })
        .unwrap_or_else(|| "-".to_string());
    format!("{location}: {severity} [{code}] {message}")
}

/// Render all diagnostics, one per line.
pub fn render_diagnostics(index: &LineIndex, diagnostics: &[Diagnostic]) -> String {
    if diagnostics.is_empty() {
        return String::new();
    }
    diagnostics
        .iter()
        .map(|d| render_diagnostic_line(index, d))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code::{DiagnosticCode, DiagnosticId};
    use crate::kind::{Diagnostic, DiagnosticCategory, DiagnosticLifecycleStage};
    use crate::message::DiagnosticMessage;
    use crate::origin::{DiagnosticOrigin, SourceOrigin};
    use crate::severity::DiagnosticSeverity;
    use reciplexa_identity::package::{ModuleId, PackageInstanceId};
    use reciplexa_source::offset::ByteOffset;
    use reciplexa_source::range::TextRange;
    use reciplexa_source::resource::SourceResourceId;

    #[test]
    fn renders_line_with_location() {
        let index = LineIndex::new("(page a4");
        let diag = Diagnostic::new(
            DiagnosticId::new(1),
            DiagnosticCode::new("compiler", "syntax", "SYN-0001"),
            DiagnosticSeverity::Error,
            DiagnosticCategory::Syntax,
            DiagnosticLifecycleStage::Parse,
            DiagnosticMessage::new("parse.unclosed"),
        )
        .with_primary_origin(DiagnosticOrigin::Source(SourceOrigin::new(
            PackageInstanceId::new(1),
            ModuleId::new(1),
            SourceResourceId::new(1),
            TextRange::try_new(ByteOffset::new(0), ByteOffset::new(5)).unwrap(),
        )));
        let line = render_diagnostic_line(&index, &diag);
        assert!(line.contains("error"));
        assert!(line.contains("SYN-0001"));
    }
}
