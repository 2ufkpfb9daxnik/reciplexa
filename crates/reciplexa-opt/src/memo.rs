//! Memoization caches for IR, layout, and render fingerprints.

use std::collections::BTreeMap;

use crate::fingerprint::Fingerprint;
use crate::invalidation::{CacheDomain, InvalidationKey, InvalidationSet};

/// Which pipeline layer a memo entry belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoLayer {
    Ir,
    Layout,
    Render,
}

impl MemoLayer {
    pub fn domain(self) -> CacheDomain {
        match self {
            MemoLayer::Ir => CacheDomain::Ir,
            MemoLayer::Layout => CacheDomain::Layout,
            MemoLayer::Render => CacheDomain::Render,
        }
    }
}

/// Key for a memoized artifact: layer + input fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoKey {
    pub layer: MemoLayer,
    pub input: Fingerprint,
}

impl MemoKey {
    pub fn new(layer: MemoLayer, input: Fingerprint) -> Self {
        Self { layer, input }
    }

    pub fn as_invalidation_key(&self) -> InvalidationKey {
        InvalidationKey::new(self.layer.domain(), self.input)
    }
}

/// Deterministic memo map: key → output fingerprint (stub value).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MemoCache {
    entries: BTreeMap<MemoKey, Fingerprint>,
}

impl MemoCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, key: &MemoKey) -> Option<Fingerprint> {
        self.entries.get(key).copied()
    }

    pub fn insert(&mut self, key: MemoKey, output: Fingerprint) {
        self.entries.insert(key, output);
    }

    pub fn remove(&mut self, key: &MemoKey) -> Option<Fingerprint> {
        self.entries.remove(key)
    }

    /// Drop entries matching an invalidation set.
    pub fn invalidate(&mut self, set: &InvalidationSet) -> usize {
        let before = self.entries.len();
        self.entries
            .retain(|k, _| !set.contains_key(&k.as_invalidation_key()));
        before - self.entries.len()
    }

    pub fn get_or_insert_with(
        &mut self,
        key: MemoKey,
        compute: fn() -> Fingerprint,
    ) -> Fingerprint {
        if let Some(v) = self.entries.get(&key).copied() {
            return v;
        }
        let v = compute();
        self.entries.insert(key, v);
        v
    }
}
