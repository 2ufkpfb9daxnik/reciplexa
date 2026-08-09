//! Native instance cache stub tests.

use reciplexa_opt::{
    CacheDomain, CachedNativeHandle, Fingerprint, InvalidationKey, InvalidationKind,
    InvalidationSet, NativeInstanceCache, NativeInstanceKey,
};

#[test]
fn insert_ready_reuses_handle() {
    let mut cache = NativeInstanceCache::new();
    assert!(cache.is_empty());
    let key = NativeInstanceKey::new(Fingerprint::of(&"adapter"), Fingerprint::of(&"abi"));
    let h1 = cache.insert_ready(key.clone());
    let h2 = cache.insert_ready(key.clone());
    assert_eq!(h1, h2);
    assert_eq!(cache.lookup(&key), Some(h1));
    assert_eq!(cache.len(), 1);
}

#[test]
fn quarantine_removes_entry() {
    let mut cache = NativeInstanceCache::new();
    let key = NativeInstanceKey::new(Fingerprint::of(&1u8), Fingerprint::of(&2u8));
    cache.insert_ready(key.clone());
    assert!(cache.quarantine(&key));
    assert!(!cache.quarantine(&key));
    assert!(cache.lookup(&key).is_none());
}

#[test]
fn invalidate_by_key() {
    let mut cache = NativeInstanceCache::new();
    let key = NativeInstanceKey::new(Fingerprint::of(&"a"), Fingerprint::of(&"b"));
    cache.insert_ready(key.clone());
    let set = InvalidationSet::from_kind(InvalidationKind::Local, key.as_invalidation_key());
    assert_eq!(cache.invalidate(&set), 1);
    assert!(cache.is_empty());
}

#[test]
fn domain_wipe_and_remove() {
    let mut cache = NativeInstanceCache::new();
    let k1 = NativeInstanceKey::new(Fingerprint::of(&1u32), Fingerprint::of(&2u32));
    let k2 = NativeInstanceKey::new(Fingerprint::of(&3u32), Fingerprint::of(&4u32));
    cache.insert_ready(k1.clone());
    cache.insert_ready(k2.clone());
    let set = InvalidationSet::from_kind(
        InvalidationKind::Domain,
        InvalidationKey::new(CacheDomain::NativeInstance, Fingerprint::zero()),
    );
    assert_eq!(cache.invalidate(&set), 2);
    cache.insert_ready(k1.clone());
    assert_eq!(cache.remove(&k1), Some(CachedNativeHandle(3)));
}
