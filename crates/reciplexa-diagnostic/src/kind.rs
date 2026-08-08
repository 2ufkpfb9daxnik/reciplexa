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
