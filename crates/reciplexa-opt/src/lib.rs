//! Optimization substrate — incremental invalidation and memoization (Phase 13).
//!
//! Stubs only: fine-grained invalidation, IR/layout/render caches, parallel
//! compile job graph, and native instance cache hooks. Not a full incremental
//! type checker.

#![forbid(unsafe_code)]

pub mod fingerprint;
pub mod invalidation;
pub mod memo;
pub mod native_cache;
pub mod parallel;

pub use fingerprint::Fingerprint;
pub use invalidation::{CacheDomain, InvalidationKey, InvalidationKind, InvalidationSet};
pub use memo::{MemoCache, MemoKey, MemoLayer};
pub use native_cache::{CachedNativeHandle, NativeInstanceCache, NativeInstanceKey};
pub use parallel::{CompileJob, CompileJobGraph, JobId, JobKind, JobStatus, ParallelSchedule};
