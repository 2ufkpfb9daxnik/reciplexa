//! Specification linkage for tests.

use core::fmt;

/// Reference to a specification section (e.g. `SYN-001` or `§1.4`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecSection(pub String);

impl SpecSection {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for SpecSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Conformance test identifier from `specification.md`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConformanceId(pub String);

impl ConformanceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for ConformanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conformance_id_formats() {
        let id = ConformanceId::new("TEST-SYN-001");
        assert_eq!(id.to_string(), "TEST-SYN-001");
    }
}
