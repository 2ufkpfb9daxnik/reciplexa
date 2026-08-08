//! Diagnostic message templates.

use std::collections::BTreeMap;

/// Structured message — not a finished display string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticMessage {
    pub template_id: String,
    pub arguments: BTreeMap<String, String>,
}

impl DiagnosticMessage {
    pub fn new(template_id: impl Into<String>) -> Self {
        Self {
            template_id: template_id.into(),
            arguments: BTreeMap::new(),
        }
    }

    pub fn with_argument(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Fallback plain-text summary for tooling without localization catalogs.
    pub fn fallback_summary(&self) -> String {
        if self.arguments.is_empty() {
            return self.template_id.clone();
        }
        let args: Vec<_> = self
            .arguments
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        format!("{} ({})", self.template_id, args.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_includes_arguments() {
        let msg = DiagnosticMessage::new("parse.unclosed-paren").with_argument("offset", "12");
        assert!(msg.fallback_summary().contains("offset=12"));
    }
}
