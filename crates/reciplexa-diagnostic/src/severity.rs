//! Diagnostic severity (`specification.md` DIAG-001 §23).

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticSeverity {
    Fatal,
    Error,
    Warning,
    Notice,
    Info,
    Hint,
}

impl DiagnosticSeverity {
    pub const fn is_error_or_worse(self) -> bool {
        matches!(self, Self::Fatal | Self::Error)
    }
}

impl fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Fatal => "fatal",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Notice => "notice",
            Self::Info => "info",
            Self::Hint => "hint",
        };
        write!(f, "{label}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_is_error_or_worse() {
        assert!(DiagnosticSeverity::Error.is_error_or_worse());
        assert!(!DiagnosticSeverity::Warning.is_error_or_worse());
    }
}
