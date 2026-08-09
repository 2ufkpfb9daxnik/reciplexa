//! MemoCache tests.

use reciplexa_opt::{
    CacheDomain, Fingerprint, InvalidationKey, InvalidationKind, InvalidationSet, MemoCache,
    MemoKey, MemoLayer,
};

#[test]
fn memo_hit_and_miss() {
    let mut cache = MemoCache::new();
    assert!(cache.is_empty());
    let key = MemoKey::new(MemoLayer::Ir, Fingerprint::of(&"src"));
    assert!(cache.get(&key).is_none());
    let out = Fingerprint::of(&"ir-out");
    cache.insert(key.clone(), out);
    assert_eq!(cache.get(&key), Some(out));
    assert_eq!(cache.len(), 1);
}

#[test]
fn get_or_insert_computes_once() {
    let mut cache = MemoCache::new();
    let key = MemoKey::new(MemoLayer::Layout, Fingerprint::of(&1u32));
    fn compute() -> Fingerprint {
        Fingerprint::of(&"L")
    }
    let a = cache.get_or_insert_with(key.clone(), compute);
    let b = cache.get_or_insert_with(key, compute);
    assert_eq!(a, b);
    assert_eq!(a, Fingerprint::of(&"L"));
}

#[test]
fn invalidate_by_domain_clears_layer() {
    let mut cache = MemoCache::new();
    cache.insert(
        MemoKey::new(MemoLayer::Render, Fingerprint::of(&1u8)),
        Fingerprint::of(&"r1"),
    );
    cache.insert(
        MemoKey::new(MemoLayer::Ir, Fingerprint::of(&2u8)),
        Fingerprint::of(&"i1"),
    );
    let set = InvalidationSet::from_kind(
        InvalidationKind::Domain,
        InvalidationKey::new(CacheDomain::Render, Fingerprint::zero()),
    );
    assert_eq!(cache.invalidate(&set), 1);
    assert_eq!(cache.len(), 1);
}

#[test]
fn remove_and_layer_domain_mapping() {
    assert_eq!(MemoLayer::Ir.domain(), CacheDomain::Ir);
    assert_eq!(MemoLayer::Layout.domain(), CacheDomain::Layout);
    assert_eq!(MemoLayer::Render.domain(), CacheDomain::Render);
    let mut cache = MemoCache::new();
    let key = MemoKey::new(MemoLayer::Ir, Fingerprint::of(&9u8));
    cache.insert(key.clone(), Fingerprint::of(&"x"));
    assert!(cache.remove(&key).is_some());
    assert!(cache.remove(&key).is_none());
}
