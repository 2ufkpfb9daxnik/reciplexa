//! Convert legacy parse errors into structured diagnostics.

use crate::code::DiagnosticCode;
use crate::collector::DiagnosticCollector;
use crate::kind::{DiagnosticCategory, DiagnosticLifecycleStage};
use crate::message::DiagnosticMessage;
use crate::origin::{DiagnosticOrigin, SourceOrigin};
use crate::severity::DiagnosticSeverity;
use reciplexa_identity::package::{ModuleId, PackageInstanceId};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;
use reciplexa_source::resource::SourceResourceId;

/// Minimal parse error input — decoupled from `reciplexa-syntax`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnosticInput {
    pub message: String,
    pub start: u32,
    pub end: u32,
}

impl ParseDiagnosticInput {
    pub fn new(message: impl Into<String>, start: u32, end: u32) -> Self {
        Self {
            message: message.into(),
            start,
            end,
        }
    }
}

/// Emit syntax diagnostics for parse failures.
pub fn push_parse_errors(
    collector: &mut DiagnosticCollector,
    package_instance_id: PackageInstanceId,
    module_id: ModuleId,
    source_resource_id: SourceResourceId,
    errors: &[ParseDiagnosticInput],
) {
    for err in errors {
        let range = TextRange::try_new(ByteOffset::new(err.start), ByteOffset::new(err.end))
            .unwrap_or(TextRange::at(ByteOffset::new(err.start)));
        let origin = DiagnosticOrigin::Source(SourceOrigin::new(
            package_instance_id,
            module_id,
            source_resource_id,
            range,
        ));
        collector.push_with_origin(
            DiagnosticCode::new("compiler", "syntax", "SYN-PARSE"),
            DiagnosticSeverity::Error,
            DiagnosticCategory::Syntax,
            DiagnosticLifecycleStage::Parse,
            DiagnosticMessage::new("parse.error").with_argument("detail", err.message.clone()),
            origin,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_errors_become_structured_diagnostics() {
        let mut collector = DiagnosticCollector::new();
        push_parse_errors(
            &mut collector,
            PackageInstanceId::new(1),
            ModuleId::new(1),
            SourceResourceId::new(1),
            &[ParseDiagnosticInput::new("unclosed paren", 0, 5)],
        );
        assert_eq!(collector.diagnostics().len(), 1);
        assert!(collector.has_errors());
        let diag = &collector.diagnostics()[0];
        assert_eq!(diag.code.as_path(), "compiler/syntax/SYN-PARSE");
    }
}
