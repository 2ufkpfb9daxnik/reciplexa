//! Optimization substrate — incremental invalidation and memoization (Phase 13).
//!
//! Stubs only: fine-grained invalidation, IR/layout/render caches, parallel
//! compile job graph, and native instance cache hooks. Not a full incremental
//! type checker.

#![forbid(unsafe_code)]

pub mod fingerprint;

pub use fingerprint::Fingerprint;
