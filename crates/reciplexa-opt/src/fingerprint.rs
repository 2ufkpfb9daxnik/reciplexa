//! Stable content fingerprints for memoization and incremental invalidation.
//!
//! Spec: same meaning → same fingerprint; never include secrets, capability
//! tokens, or native pointers.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Opaque, deterministic fingerprint of a cacheable unit of work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fingerprint(u64);

impl Fingerprint {
    /// Hash structured input that is already stripped of secrets/pointers.
    pub fn of<T: Hash>(value: &T) -> Self {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Combine ordered fingerprints (order-sensitive).
    pub fn combine(parts: &[Fingerprint]) -> Self {
        let mut hasher = DefaultHasher::new();
        for p in parts {
            p.0.hash(&mut hasher);
        }
        Self(hasher.finish())
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Empty / zero fingerprint (useful as sentinel).
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<u64> for Fingerprint {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fp:{:016x}", self.0)
    }
}
