//! Diagnostic origins.

use reciplexa_identity::package::{ModuleId, PackageInstanceId};
use reciplexa_identity::syntax::SyntaxNodeId;
use reciplexa_source::range::TextRange;
use reciplexa_source::resource::SourceResourceId;

/// Where a diagnostic points (`specification.md` DIAG-001 §27–28).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticOrigin {
    Source(SourceOrigin),
    Unknown,
}

/// Canonical source origin — file paths are not the primary key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceOrigin {
    pub package_instance_id: PackageInstanceId,
    pub module_id: ModuleId,
    pub source_resource_id: SourceResourceId,
    pub text_range: TextRange,
    pub syntax_node_id: Option<SyntaxNodeId>,
}

impl SourceOrigin {
    pub fn new(
        package_instance_id: PackageInstanceId,
        module_id: ModuleId,
        source_resource_id: SourceResourceId,
        text_range: TextRange,
    ) -> Self {
        Self {
            package_instance_id,
            module_id,
            source_resource_id,
            text_range,
            syntax_node_id: None,
        }
    }

    pub fn with_syntax_node_id(mut self, id: SyntaxNodeId) -> Self {
        self.syntax_node_id = Some(id);
        self
    }
}
