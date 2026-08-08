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

    #[test]
    fn severity_partitions_and_display() {
        assert!(DiagnosticSeverity::Fatal.is_error_or_worse());
        assert!(DiagnosticSeverity::Error.is_error_or_worse());
        for s in [
            DiagnosticSeverity::Warning,
            DiagnosticSeverity::Notice,
            DiagnosticSeverity::Info,
            DiagnosticSeverity::Hint,
        ] {
            assert!(!s.is_error_or_worse());
        }
        assert_eq!(DiagnosticSeverity::Fatal.to_string(), "fatal");
        assert_eq!(DiagnosticSeverity::Error.to_string(), "error");
        assert_eq!(DiagnosticSeverity::Warning.to_string(), "warning");
        assert_eq!(DiagnosticSeverity::Notice.to_string(), "notice");
        assert_eq!(DiagnosticSeverity::Info.to_string(), "info");
        assert_eq!(DiagnosticSeverity::Hint.to_string(), "hint");
        assert!(DiagnosticSeverity::Fatal < DiagnosticSeverity::Hint);
    }
}
