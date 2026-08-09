//! InvalidationSet tests.

use reciplexa_opt::{
    CacheDomain, Fingerprint, InvalidationKind, InvalidationKey, InvalidationSet,
};

#[test]
fn local_invalidation_hits_key_only() {
    let fp = Fingerprint::of(&"node-1");
    let key = InvalidationKey::new(CacheDomain::Ir, fp);
    let set = InvalidationSet::from_kind(InvalidationKind::Local, key.clone());
    assert!(set.contains_key(&key));
    assert!(!set.contains_domain(CacheDomain::Ir));
    assert_eq!(set.len_keys(), 1);
}

#[test]
fn domain_invalidation_covers_all_keys_in_domain() {
    let key = InvalidationKey::new(CacheDomain::Layout, Fingerprint::of(&7u32));
    let set = InvalidationSet::from_kind(InvalidationKind::Domain, key.clone());
    assert!(set.contains_domain(CacheDomain::Layout));
    assert!(set.contains_key(&key));
    assert!(!set.contains_domain(CacheDomain::Render));
}

#[test]
fn merge_unions_keys_and_domains() {
    let mut a = InvalidationSet::new();
    a.insert_key(InvalidationKey::new(CacheDomain::Types, Fingerprint::of(&1u8)));
    let mut b = InvalidationSet::new();
    b.insert_domain(CacheDomain::Render);
    a.merge(&b);
    assert!(!a.is_empty());
    assert!(a.contains_domain(CacheDomain::Render));
    assert_eq!(a.keys().count(), 1);
    assert_eq!(a.domains().count(), 1);
}

#[test]
fn dependents_kind_inserts_key() {
    let key = InvalidationKey::new(CacheDomain::NativeInstance, Fingerprint::zero());
    let set = InvalidationSet::from_kind(InvalidationKind::Dependents, key.clone());
    assert!(set.contains_key(&key));
}
