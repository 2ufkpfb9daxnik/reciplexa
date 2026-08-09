//! Native instance cache stub — keyed by contract fingerprint, not raw pointers.

use std::collections::BTreeMap;

use crate::fingerprint::Fingerprint;
use crate::invalidation::{CacheDomain, InvalidationKey, InvalidationSet};

/// Logical key for a cached native instance (never a host pointer).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NativeInstanceKey {
    pub adapter_fp: Fingerprint,
    pub abi_fp: Fingerprint,
}

impl NativeInstanceKey {
    pub fn new(adapter_fp: Fingerprint, abi_fp: Fingerprint) -> Self {
        Self { adapter_fp, abi_fp }
    }

    pub fn combined(&self) -> Fingerprint {
        Fingerprint::combine(&[self.adapter_fp, self.abi_fp])
    }

    pub fn as_invalidation_key(&self) -> InvalidationKey {
        InvalidationKey::new(CacheDomain::NativeInstance, self.combined())
    }
}

/// Opaque handle id (not a live pointer) for a Ready native instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CachedNativeHandle(pub u64);

/// Stub cache: Ready instances only; Quarantined entries are purged.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeInstanceCache {
    ready: BTreeMap<NativeInstanceKey, CachedNativeHandle>,
    next_handle: u64,
}

impl NativeInstanceCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.ready.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ready.is_empty()
    }

    pub fn lookup(&self, key: &NativeInstanceKey) -> Option<CachedNativeHandle> {
        self.ready.get(key).copied()
    }

    /// Insert or replace a Ready instance handle.
    pub fn insert_ready(&mut self, key: NativeInstanceKey) -> CachedNativeHandle {
        if let Some(h) = self.ready.get(&key).copied() {
            return h;
        }
        self.next_handle = self.next_handle.saturating_add(1);
        let h = CachedNativeHandle(self.next_handle);
        self.ready.insert(key, h);
        h
    }

    pub fn remove(&mut self, key: &NativeInstanceKey) -> Option<CachedNativeHandle> {
        self.ready.remove(key)
    }

    pub fn invalidate(&mut self, set: &InvalidationSet) -> usize {
        let before = self.ready.len();
        self.ready
            .retain(|k, _| !set.contains_key(&k.as_invalidation_key()));
        before - self.ready.len()
    }

    /// Quarantine removes the entry (must not publish non-Ready instances).
    pub fn quarantine(&mut self, key: &NativeInstanceKey) -> bool {
        self.ready.remove(key).is_some()
    }
}
