//! Coverage gap tests for reciplexa-opt.

use reciplexa_opt::{
    CacheDomain, Fingerprint, InvalidationKind, InvalidationKey, InvalidationSet, MemoCache,
    MemoKey, MemoLayer,
};

#[test]
fn is_empty_all_branches() {
    let empty = InvalidationSet::new();
    assert!(empty.is_empty());

    let mut domains_only = InvalidationSet::new();
    domains_only.insert_domain(CacheDomain::Types);
    assert!(!domains_only.is_empty());

    let mut keys_only = InvalidationSet::new();
    keys_only.insert_key(InvalidationKey::new(
        CacheDomain::Ir,
        Fingerprint::of(&1u8),
    ));
    assert!(!keys_only.is_empty());
}

#[test]
fn merge_with_keys_and_empty_other() {
    let mut a = InvalidationSet::new();
    let mut b = InvalidationSet::new();
    b.insert_key(InvalidationKey::new(
        CacheDomain::Layout,
        Fingerprint::of(&9u8),
    ));
    b.insert_domain(CacheDomain::Render);
    a.merge(&b);
    assert_eq!(a.len_keys(), 1);
    assert!(a.contains_domain(CacheDomain::Render));

    let empty = InvalidationSet::new();
    a.merge(&empty);
    assert_eq!(a.len_keys(), 1);
}

#[test]
fn memo_local_invalidation_and_empty_cache() {
    let mut cache = MemoCache::new();
    assert!(cache.is_empty());
    let key = MemoKey::new(MemoLayer::Layout, Fingerprint::of(&"x"));
    cache.insert(key.clone(), Fingerprint::of(&"y"));
    let set = InvalidationSet::from_kind(
        InvalidationKind::Local,
        key.as_invalidation_key(),
    );
    assert_eq!(cache.invalidate(&set), 1);
    assert_eq!(cache.invalidate(&set), 0);
}
