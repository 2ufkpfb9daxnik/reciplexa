//! Diagnostic identifiers and codes.

use core::fmt;

/// Instance identifier for a single diagnostic emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticId(pub u64);

impl DiagnosticId {
    pub const INVALID: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for DiagnosticId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "diag:{}", self.0)
    }
}

/// Stable semantic code (`namespace/category/CODE`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticCode {
    pub namespace: String,
    pub category: String,
    pub code: String,
}

impl DiagnosticCode {
    pub fn new(
        namespace: impl Into<String>,
        category: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            category: category.into(),
            code: code.into(),
        }
    }

    pub fn as_path(&self) -> String {
        format!("{}/{}/{}", self.namespace, self.category, self.code)
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_path_is_stable() {
        let code = DiagnosticCode::new("compiler", "type", "TYPE-0012");
        assert_eq!(code.as_path(), "compiler/type/TYPE-0012");
    }
}
