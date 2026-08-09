//! Proof obligations matching Phase 13 roadmap priorities.

use crate::status::ProofStatus;

/// Ordered priorities from the roadmap (1 = highest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProofPriority {
    CoreTypeSafety = 1,
    EffectSoundness = 2,
    HandlerLoweringPreservation = 3,
    PerceusPassSoundness = 4,
    OwnershipReuseVerifier = 5,
    IrLoweringPreservation = 6,
    TransactionAtomicity = 7,
    IncrementalFullRebuildEquiv = 8,
}

impl ProofPriority {
    pub fn all() -> &'static [ProofPriority] {
        &[
            ProofPriority::CoreTypeSafety,
            ProofPriority::EffectSoundness,
            ProofPriority::HandlerLoweringPreservation,
            ProofPriority::PerceusPassSoundness,
            ProofPriority::OwnershipReuseVerifier,
            ProofPriority::IrLoweringPreservation,
            ProofPriority::TransactionAtomicity,
            ProofPriority::IncrementalFullRebuildEquiv,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ProofPriority::CoreTypeSafety => "core-type-safety",
            ProofPriority::EffectSoundness => "effect-soundness",
            ProofPriority::HandlerLoweringPreservation => "handler-lowering-preservation",
            ProofPriority::PerceusPassSoundness => "perceus-pass-soundness",
            ProofPriority::OwnershipReuseVerifier => "ownership-reuse-verifier",
            ProofPriority::IrLoweringPreservation => "ir-lowering-preservation",
            ProofPriority::TransactionAtomicity => "transaction-atomicity",
            ProofPriority::IncrementalFullRebuildEquiv => "incremental-full-rebuild-equiv",
        }
    }

    pub fn rank(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofObligation {
    pub priority: ProofPriority,
    pub title: String,
    pub status: ProofStatus,
    pub notes: String,
}

impl ProofObligation {
    pub fn new(priority: ProofPriority, title: impl Into<String>) -> Self {
        Self {
            priority,
            title: title.into(),
            status: ProofStatus::Open,
            notes: String::new(),
        }
    }

    pub fn with_status(mut self, status: ProofStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = notes.into();
        self
    }

    pub fn set_status(&mut self, status: ProofStatus) {
        self.status = status;
    }

    /// Seed the eight roadmap obligations as Open stubs.
    pub fn roadmap_defaults() -> Vec<Self> {
        ProofPriority::all()
            .iter()
            .map(|p| Self::new(*p, p.as_str()))
            .collect()
    }
}
