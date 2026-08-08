//! Package manifest, resolver, lockfile (Phase 10).

#![forbid(unsafe_code)]

pub mod lockfile;
pub mod manifest;
pub mod resolver;
pub mod target;

pub use lockfile::{Lockfile, LockedPackage};
pub use manifest::{DependencySpec, PackageManifest};
pub use resolver::{resolve_packages, ResolveError, ResolvedGraph};
pub use target::{BuildTarget, RuntimeProfile};
