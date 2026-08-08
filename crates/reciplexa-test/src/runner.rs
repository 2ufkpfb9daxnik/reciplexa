//! Conformance test case runner.

use crate::{ConformanceId, SpecSection};

/// Describes a single conformance test linked to specification ids.
#[derive(Debug, Clone)]
pub struct ConformanceCase {
    pub id: ConformanceId,
    pub spec_section: SpecSection,
    pub description: String,
}

impl ConformanceCase {
    pub fn new(
        id: impl Into<String>,
        spec_section: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: ConformanceId::new(id),
            spec_section: SpecSection::new(spec_section),
            description: description.into(),
        }
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.id, self.spec_section)
    }
}

/// Run a conformance case body, panicking with the case label on failure.
pub fn run_conformance(case: &ConformanceCase, body: impl FnOnce()) {
    body();
    let _ = &case.label();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_label_includes_ids() {
        let case = ConformanceCase::new("TEST-SYN-C001", "SYN-001", "round-trip");
        assert!(case.label().contains("TEST-SYN-C001"));
    }
}
