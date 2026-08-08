//! Package manifest, resolver, lockfile (Phase 10).

#![forbid(unsafe_code)]

pub mod build;
pub mod lockfile;
pub mod manifest;
pub mod resolver;
pub mod rpxm;
pub mod target;

pub use build::{
    diagnose_manifest, BuildGraph, BuildNode, BuildNodeId, IncrementalCache, InvalidationKind,
    PackageDiagnostic,
};
pub use lockfile::{LockedPackage, Lockfile};
pub use manifest::{DependencySpec, PackageManifest};
pub use resolver::{resolve_packages, ResolveError, ResolvedGraph};
pub use rpxm::{parse_rpxm, RpxmError};
pub use target::{BuildTarget, RuntimeProfile};
