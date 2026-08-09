//! Proof crate unit tests.

use reciplexa_proof::{
    Lemma, LemmaId, LemmaRegistry, ProofObligation, ProofPriority, ProofStatus,
};

#[test]
fn roadmap_defaults_are_eight_open() {
    let obs = ProofObligation::roadmap_defaults();
    assert_eq!(obs.len(), 8);
    assert!(obs.iter().all(|o| o.status == ProofStatus::Open));
    assert_eq!(ProofPriority::all().len(), 8);
    assert_eq!(ProofPriority::CoreTypeSafety.rank(), 1);
    assert_eq!(
        ProofPriority::IncrementalFullRebuildEquiv.as_str(),
        "incremental-full-rebuild-equiv"
    );
}

#[test]
fn obligation_status_and_notes() {
    let mut o = ProofObligation::new(ProofPriority::EffectSoundness, "effects")
        .with_notes("sketch")
        .with_status(ProofStatus::Draft);
    assert_eq!(o.notes, "sketch");
    assert_eq!(o.status, ProofStatus::Draft);
    o.set_status(ProofStatus::Deferred);
    assert!(o.status.is_blocking() == false);
    assert!(ProofStatus::Open.is_blocking());
    assert!(ProofStatus::Discharged.is_terminal());
    assert!(ProofStatus::Failed.is_terminal());
    assert!(!ProofStatus::Draft.is_terminal());
}

#[test]
fn lemma_registry_crud() {
    let mut reg = LemmaRegistry::new();
    assert!(reg.is_empty());
    let lemma = Lemma::new("L1", ProofPriority::CoreTypeSafety, "Γ ⊢ e : τ")
        .with_status(ProofStatus::Discharged);
    reg.insert(lemma);
    assert_eq!(reg.len(), 1);
    assert_eq!(reg.discharged_count(), 1);
    let id = LemmaId::new("L1");
    assert!(reg.get(&id).is_some());
    assert_eq!(reg.for_obligation(ProofPriority::CoreTypeSafety).len(), 1);
    assert_eq!(reg.for_obligation(ProofPriority::EffectSoundness).len(), 0);
    assert!(reg.set_status(&id, ProofStatus::Failed));
    assert!(!reg.set_status(&LemmaId::new("missing"), ProofStatus::Open));
    assert_eq!(reg.ids().count(), 1);
}
