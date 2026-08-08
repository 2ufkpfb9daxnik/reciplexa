//! Expected failure reports distinct from defects (`specification.md` ERR-001).

use core::fmt;

use reciplexa_source::range::TextRange;

/// Stable failure classification code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureCode {
    pub namespace: String,
    pub code: String,
}

impl FailureCode {
    pub fn new(namespace: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            code: code.into(),
        }
    }

    pub fn as_path(&self) -> String {
        format!("{}/{}", self.namespace, self.code)
    }
}

/// Structured expected failure — not a defect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureReport {
    pub failure_id: u64,
    pub code: FailureCode,
    pub message: String,
    pub source_range: Option<TextRange>,
}

impl FailureReport {
    pub fn new(failure_id: u64, code: FailureCode, message: impl Into<String>) -> Self {
        Self {
            failure_id,
            code,
            message: message.into(),
            source_range: None,
        }
    }

    pub fn with_source_range(mut self, range: TextRange) -> Self {
        self.source_range = Some(range);
        self
    }
}

impl fmt::Display for FailureReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failure:{} [{}] {}",
            self.failure_id, self.code.namespace, self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_is_not_defect_string() {
        let report =
            FailureReport::new(1, FailureCode::new("compiler", "TYPE-001"), "type mismatch");
        assert!(report.to_string().contains("failure:1"));
    }

    #[test]
    fn failure_code_path_and_source_range() {
        use reciplexa_source::offset::ByteOffset;
        use reciplexa_source::range::TextRange;

        let code = FailureCode::new("bind", "UNBOUND");
        assert_eq!(code.as_path(), "bind/UNBOUND");
        let report = FailureReport::new(2, code, "unbound `foo`")
            .with_source_range(TextRange::try_new(ByteOffset::new(1), ByteOffset::new(4)).unwrap());
        assert_eq!(report.source_range.unwrap().len(), 3);
        assert!(report.to_string().contains("failure:2"));
    }
}
