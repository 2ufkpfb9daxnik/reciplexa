//! Lemma registry — named lemmas attached to proof obligations.

use std::collections::BTreeMap;

use crate::obligation::ProofPriority;
use crate::status::ProofStatus;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LemmaId(pub String);

impl LemmaId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lemma {
    pub id: LemmaId,
    pub obligation: ProofPriority,
    pub statement: String,
    pub status: ProofStatus,
}

impl Lemma {
    pub fn new(
        id: impl Into<String>,
        obligation: ProofPriority,
        statement: impl Into<String>,
    ) -> Self {
        Self {
            id: LemmaId::new(id),
            obligation,
            statement: statement.into(),
            status: ProofStatus::Open,
        }
    }

    pub fn with_status(mut self, status: ProofStatus) -> Self {
        self.status = status;
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LemmaRegistry {
    lemmas: BTreeMap<LemmaId, Lemma>,
}

impl LemmaRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.lemmas.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lemmas.is_empty()
    }

    pub fn insert(&mut self, lemma: Lemma) -> Option<Lemma> {
        self.lemmas.insert(lemma.id.clone(), lemma)
    }

    pub fn get(&self, id: &LemmaId) -> Option<&Lemma> {
        self.lemmas.get(id)
    }

    pub fn set_status(&mut self, id: &LemmaId, status: ProofStatus) -> bool {
        if let Some(l) = self.lemmas.get_mut(id) {
            l.status = status;
            true
        } else {
            false
        }
    }

    pub fn for_obligation(&self, priority: ProofPriority) -> Vec<&Lemma> {
        self.lemmas
            .values()
            .filter(|l| l.obligation == priority)
            .collect()
    }

    pub fn discharged_count(&self) -> usize {
        self.lemmas
            .values()
            .filter(|l| l.status == ProofStatus::Discharged)
            .count()
    }

    pub fn ids(&self) -> impl Iterator<Item = &LemmaId> {
        self.lemmas.keys()
    }
}
