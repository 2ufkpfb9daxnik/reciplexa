//! Explicit output loss reporting (never silent for Raster Preview vs Final).

use crate::profile::ProfileKind;

/// Spec-aligned loss kind (Phase 12 subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputLossKind {
    SemanticText,
    Editability,
    VectorRepresentation,
    VisualFidelity,
    GeometricPrecision,
}

/// How this profile treats the loss (AllowSilently is forbidden for Raster).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LossDisposition {
    Report,
    Warn,
    RequireExplicitApproval,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputLoss {
    pub kind: OutputLossKind,
    pub disposition: LossDisposition,
    pub profile: ProfileKind,
    pub detail: String,
}

impl OutputLoss {
    pub fn report(kind: OutputLossKind, profile: ProfileKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            disposition: LossDisposition::Report,
            profile,
            detail: detail.into(),
        }
    }

    pub fn warn(kind: OutputLossKind, profile: ProfileKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            disposition: LossDisposition::Warn,
            profile,
            detail: detail.into(),
        }
    }
}

/// Bundle of losses attached to an artifact; always carries the profile that produced them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LossReport {
    pub profile: ProfileKind,
    pub losses: Vec<OutputLoss>,
}

impl LossReport {
    pub fn empty(profile: ProfileKind) -> Self {
        Self {
            profile,
            losses: Vec::new(),
        }
    }

    pub fn push(&mut self, loss: OutputLoss) {
        debug_assert_eq!(loss.profile, self.profile);
        self.losses.push(loss);
    }

    pub fn is_empty(&self) -> bool {
        self.losses.is_empty()
    }
}
