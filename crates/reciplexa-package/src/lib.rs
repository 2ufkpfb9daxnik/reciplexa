//! Package manifest, resolver, lockfile (Phase 10).

#![forbid(unsafe_code)]

pub mod build;
pub mod graphics_bridge;
pub mod load;
pub mod lockfile;
pub mod manifest;
pub mod resolver;
pub mod rpi;
pub mod rpxm;
pub mod target;
pub mod workspace;

pub use build::{
    diagnose_manifest, BuildGraph, BuildNode, BuildNodeId, IncrementalCache, InvalidationKind,
    PackageDiagnostic,
};
pub use graphics_bridge::{document_from_package_entry, GraphicsBridgeError};
pub use load::{
    elaborate_with_packages, load_module_tree_with_packages, LocalPackageIndex, PackageLoadError,
    ResolvedImport,
};
pub use lockfile::{LockedPackage, Lockfile};
pub use manifest::{DependencySpec, PackageManifest};
pub use resolver::{resolve_packages, ResolveError, ResolvedGraph};
pub use rpi::parse_rpi_exports;
pub use rpxm::{parse_rpxm, RpxmError};
pub use target::{BuildTarget, RuntimeProfile};
pub use workspace::{parse_workspace_rpxm, WorkspaceManifest};
