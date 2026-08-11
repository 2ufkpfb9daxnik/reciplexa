//! Provenance kind taxonomy (EDT-001 §17.4) shared across document and language kernel.

use core::fmt;

/// EDT-001 §17.4 provenance classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProvenanceKind {
    UserCreated,
    SourceGenerated,
    MacroGenerated,
    Imported,
    Copied,
    Derived,
}

impl fmt::Display for ProvenanceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::UserCreated => "user-created",
            Self::SourceGenerated => "source-generated",
            Self::MacroGenerated => "macro-generated",
            Self::Imported => "imported",
            Self::Copied => "copied",
            Self::Derived => "derived",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provenance_kind_display() {
        assert_eq!(
            ProvenanceKind::SourceGenerated.to_string(),
            "source-generated"
        );
    }
}
