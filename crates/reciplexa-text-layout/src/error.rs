//! Structured layout failures (missing font / missing glyph / policy).

use std::fmt;

/// Product typesetting failure. Never a silent `.notdef` success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// Face bytes could not be parsed as a TrueType/OpenType font.
    InvalidFont { detail: String },
    /// Requested font resource is not in the registry.
    MissingFont { font_id: String },
    /// Input scalar has no glyph in the selected face.
    MissingGlyph { font_id: String, scalar: char },
    /// MATH table or a required MATH constant is absent.
    MissingMathTable { font_id: String, detail: String },
    /// Metric-changing substitution was attempted without relayout.
    SubstitutionRequiresRelayout {
        requested: String,
        substitute: String,
        reason: String,
    },
    /// Profile or engine invariant violation.
    Engine { detail: String },
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFont { detail } => write!(f, "invalid font: {detail}"),
            Self::MissingFont { font_id } => write!(f, "missing font `{font_id}`"),
            Self::MissingGlyph { font_id, scalar } => {
                write!(
                    f,
                    "font `{font_id}` missing glyph for U+{:04X}",
                    *scalar as u32
                )
            }
            Self::MissingMathTable { font_id, detail } => {
                write!(f, "font `{font_id}` MATH table: {detail}")
            }
            Self::SubstitutionRequiresRelayout {
                requested,
                substitute,
                reason,
            } => write!(
                f,
                "font substitution `{requested}` → `{substitute}` requires relayout ({reason})"
            ),
            Self::Engine { detail } => write!(f, "layout engine: {detail}"),
        }
    }
}

impl std::error::Error for LayoutError {}
