//! Diagnostic record and categories.

use crate::code::DiagnosticCode;
use crate::message::DiagnosticMessage;
use crate::origin::DiagnosticOrigin;
use crate::severity::DiagnosticSeverity;

pub use crate::code::DiagnosticId;

/// Processing stage where a diagnostic was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticLifecycleStage {
    Parse,
    Expand,
    Resolve,
    TypeCheck,
    Lower,
    Evaluate,
    Plan,
    Emit,
    Verify,
    Shutdown,
    Test,
}

/// Semantic category of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCategory {
    Syntax,
    Macro,
    NameResolution,
    Type,
    Effect,
    Resource,
    Task,
    Runtime,
    Native,
    Package,
    Manifest,
    Layout,
    Render,
    Backend,
    Test,
    Codec,
    Gui,
    Security,
    Performance,
    Internal,
}

/// Canonical diagnostic record (`specification.md` DIAG-001 §21).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub id: DiagnosticId,
    pub code: DiagnosticCode,
    pub schema_version: u32,
    pub severity: DiagnosticSeverity,
    pub category: DiagnosticCategory,
    pub lifecycle_stage: DiagnosticLifecycleStage,
    pub message: DiagnosticMessage,
    pub primary_origin: Option<DiagnosticOrigin>,
    pub related_origins: Vec<DiagnosticOrigin>,
}

impl Diagnostic {
    pub fn new(
        id: DiagnosticId,
        code: DiagnosticCode,
        severity: DiagnosticSeverity,
        category: DiagnosticCategory,
        lifecycle_stage: DiagnosticLifecycleStage,
        message: DiagnosticMessage,
    ) -> Self {
        Self {
            id,
            code,
            schema_version: 1,
            severity,
            category,
            lifecycle_stage,
            message,
            primary_origin: None,
            related_origins: Vec::new(),
        }
    }

    pub fn with_primary_origin(mut self, origin: DiagnosticOrigin) -> Self {
        self.primary_origin = Some(origin);
        self
    }

    pub fn with_related_origin(mut self, origin: DiagnosticOrigin) -> Self {
        self.related_origins.push(origin);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::DiagnosticMessage;
    use reciplexa_identity::package::{ModuleId, PackageInstanceId};
    use reciplexa_identity::syntax::SyntaxNodeId;
    use reciplexa_source::offset::ByteOffset;
    use reciplexa_source::range::TextRange;
    use reciplexa_source::resource::SourceResourceId;

    use crate::origin::SourceOrigin;

    #[test]
    fn diagnostic_builder_and_enum_partitions() {
        let stages = [
            DiagnosticLifecycleStage::Parse,
            DiagnosticLifecycleStage::Expand,
            DiagnosticLifecycleStage::Resolve,
            DiagnosticLifecycleStage::TypeCheck,
            DiagnosticLifecycleStage::Lower,
            DiagnosticLifecycleStage::Evaluate,
            DiagnosticLifecycleStage::Plan,
            DiagnosticLifecycleStage::Emit,
            DiagnosticLifecycleStage::Verify,
            DiagnosticLifecycleStage::Shutdown,
            DiagnosticLifecycleStage::Test,
        ];
        assert_eq!(stages.len(), 11);

        let categories = [
            DiagnosticCategory::Syntax,
            DiagnosticCategory::Macro,
            DiagnosticCategory::NameResolution,
            DiagnosticCategory::Type,
            DiagnosticCategory::Effect,
            DiagnosticCategory::Resource,
            DiagnosticCategory::Task,
            DiagnosticCategory::Runtime,
            DiagnosticCategory::Native,
            DiagnosticCategory::Package,
            DiagnosticCategory::Manifest,
            DiagnosticCategory::Layout,
            DiagnosticCategory::Render,
            DiagnosticCategory::Backend,
            DiagnosticCategory::Test,
            DiagnosticCategory::Codec,
            DiagnosticCategory::Gui,
            DiagnosticCategory::Security,
            DiagnosticCategory::Performance,
            DiagnosticCategory::Internal,
        ];
        assert_eq!(categories.len(), 20);

        let origin = DiagnosticOrigin::Source(
            SourceOrigin::new(
                PackageInstanceId::new(1),
                ModuleId::new(2),
                SourceResourceId::new(3),
                TextRange::new(ByteOffset::new(0), ByteOffset::new(4)),
            )
            .with_syntax_node_id(SyntaxNodeId::new(5)),
        );
        let d = Diagnostic::new(
            DiagnosticId::new(1),
            DiagnosticCode::new("ns", "cat", "C1"),
            DiagnosticSeverity::Error,
            DiagnosticCategory::Type,
            DiagnosticLifecycleStage::TypeCheck,
            DiagnosticMessage::new("t").with_argument("k", "v"),
        )
        .with_primary_origin(origin.clone())
        .with_related_origin(DiagnosticOrigin::Unknown);
        assert_eq!(d.schema_version, 1);
        assert!(d.primary_origin.is_some());
        assert_eq!(d.related_origins.len(), 1);
        let _ = format!("{:?}", d);
    }
}
