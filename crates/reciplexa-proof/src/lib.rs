//! Lightweight proof obligation and lemma registry stubs (Phase 13).
//!
//! Not formal verification — a registry of obligations corresponding to the
//! roadmap's deferred proof priorities, with status tracking only.

#![forbid(unsafe_code)]

pub mod lemma;
pub mod obligation;
pub mod status;

pub use lemma::{Lemma, LemmaId, LemmaRegistry};
pub use obligation::{ProofObligation, ProofPriority};
pub use status::ProofStatus;
