//! Fine-grained invalidation sets for incremental rebuild.

use std::collections::BTreeSet;

use crate::fingerprint::Fingerprint;

/// Cache domain that may be invalidated independently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CacheDomain {
    Types,
    Ir,
    Layout,
    Render,
    NativeInstance,
}

/// How far an invalidation should propagate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidationKind {
    /// Local only — this key's memo entry.
    Local,
    /// Dependents that consume this fingerprint.
    Dependents,
    /// Entire domain (e.g. after a global config change).
    Domain,
}

/// A single invalidation target.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InvalidationKey {
    pub domain: CacheDomain,
    pub fingerprint: Fingerprint,
}

impl InvalidationKey {
    pub fn new(domain: CacheDomain, fingerprint: Fingerprint) -> Self {
        Self {
            domain,
            fingerprint,
        }
    }
}

/// Ordered set of invalidation keys (deterministic iteration).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InvalidationSet {
    keys: BTreeSet<InvalidationKey>,
    domains: BTreeSet<CacheDomain>,
}

impl InvalidationSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty() && self.domains.is_empty()
    }

    pub fn len_keys(&self) -> usize {
        self.keys.len()
    }

    pub fn insert_key(&mut self, key: InvalidationKey) {
        self.keys.insert(key);
    }

    pub fn insert_domain(&mut self, domain: CacheDomain) {
        self.domains.insert(domain);
    }

    pub fn contains_key(&self, key: &InvalidationKey) -> bool {
        self.keys.contains(key) || self.domains.contains(&key.domain)
    }

    pub fn contains_domain(&self, domain: CacheDomain) -> bool {
        self.domains.contains(&domain)
    }

    pub fn keys(&self) -> impl Iterator<Item = &InvalidationKey> {
        self.keys.iter()
    }

    pub fn domains(&self) -> impl Iterator<Item = CacheDomain> + '_ {
        self.domains.iter().copied()
    }

    /// Union `other` into `self`.
    pub fn merge(&mut self, other: &InvalidationSet) {
        for k in &other.keys {
            self.keys.insert(k.clone());
        }
        for d in &other.domains {
            self.domains.insert(*d);
        }
    }

    /// Build an invalidation set from a kind + key.
    pub fn from_kind(kind: InvalidationKind, key: InvalidationKey) -> Self {
        let mut set = Self::new();
        match kind {
            InvalidationKind::Local | InvalidationKind::Dependents => {
                set.insert_key(key);
            }
            InvalidationKind::Domain => {
                set.insert_domain(key.domain);
            }
        }
        set
    }
}
