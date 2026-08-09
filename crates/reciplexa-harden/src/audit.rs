//! Capability and privacy audit finding stubs.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuditKind {
    Capability,
    Privacy,
    ReproducibleBuild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrivacyLabel {
    Public,
    Internal,
    Secret,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditFinding {
    pub kind: AuditKind,
    pub severity: AuditSeverity,
    pub code: String,
    pub message: String,
}

impl AuditFinding {
    pub fn new(
        kind: AuditKind,
        severity: AuditSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            severity,
            code: code.into(),
            message: message.into(),
        }
    }
}

/// Capability audit: required vs granted capabilities.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilityAudit {
    pub required: Vec<String>,
    pub granted: Vec<String>,
}

impl CapabilityAudit {
    pub fn new(required: Vec<String>, granted: Vec<String>) -> Self {
        Self { required, granted }
    }

    pub fn findings(&self) -> Vec<AuditFinding> {
        let mut out = Vec::new();
        for req in &self.required {
            if !self.granted.iter().any(|g| g == req) {
                out.push(AuditFinding::new(
                    AuditKind::Capability,
                    AuditSeverity::Error,
                    "CAP001",
                    format!("missing capability: {req}"),
                ));
            }
        }
        for g in &self.granted {
            if !self.required.iter().any(|r| r == g) {
                out.push(AuditFinding::new(
                    AuditKind::Capability,
                    AuditSeverity::Warning,
                    "CAP002",
                    format!("extra capability granted: {g}"),
                ));
            }
        }
        out
    }

    pub fn is_clean(&self) -> bool {
        self.findings()
            .iter()
            .all(|f| f.severity == AuditSeverity::Info)
    }
}

/// Privacy audit: labels must not leak Secret into Public reports.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrivacyAudit {
    pub fields: Vec<(String, PrivacyLabel)>,
}

impl PrivacyAudit {
    pub fn new(fields: Vec<(String, PrivacyLabel)>) -> Self {
        Self { fields }
    }

    pub fn findings(&self) -> Vec<AuditFinding> {
        let mut out = Vec::new();
        for (name, label) in &self.fields {
            match label {
                PrivacyLabel::Secret => {
                    out.push(AuditFinding::new(
                        AuditKind::Privacy,
                        AuditSeverity::Error,
                        "PRIV001",
                        format!("secret field must not be reported publicly: {name}"),
                    ));
                }
                PrivacyLabel::Unknown => {
                    out.push(AuditFinding::new(
                        AuditKind::Privacy,
                        AuditSeverity::Warning,
                        "PRIV002",
                        format!("unlabeled privacy field: {name}"),
                    ));
                }
                PrivacyLabel::Public | PrivacyLabel::Internal => {}
            }
        }
        out
    }

    pub fn has_secrets(&self) -> bool {
        self.fields.iter().any(|(_, l)| *l == PrivacyLabel::Secret)
    }
}
