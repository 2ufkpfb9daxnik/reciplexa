//! Proof status for an obligation or lemma.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProofStatus {
    /// Not yet attempted.
    Open,
    /// Sketch / informal argument recorded.
    Draft,
    /// Machine-checked or accepted lemma.
    Discharged,
    /// Known gap / deferred.
    Deferred,
    /// Counterexample or failed attempt.
    Failed,
}

impl ProofStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, ProofStatus::Discharged | ProofStatus::Failed)
    }

    pub fn is_blocking(self) -> bool {
        matches!(self, ProofStatus::Open | ProofStatus::Failed)
    }
}
