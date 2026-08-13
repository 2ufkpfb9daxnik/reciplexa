//! Package manifest, resolver, lockfile (Phase 10).
//!
//! # OPEN stubs
//!
//! [`OPEN_PKG_001_REGISTRY`] / [`WorkspaceError::RegistryUnavailable`]: the package
//! registry protocol is not implemented. Resolvers must refuse registry-shaped
//! dependencies without network I/O.

#![forbid(unsafe_code)]

pub mod build;
pub mod domain_bodies;
pub mod domain_native;
pub mod graphics_bridge;
pub mod japanese_bridge;
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
pub use domain_bodies::{
    color_srgb_module, color_srgb_source, document_page_module, document_page_source,
    graphics_color_module, graphics_color_source, graphics_page_module, graphics_page_source,
    graphics_shapes_module, graphics_shapes_source, japanese_classes_module,
    japanese_classes_source, japanese_kihon_module, japanese_kihon_source,
    japanese_linebreak_module, japanese_linebreak_source, japanese_markup_module,
    japanese_markup_source, length_units_module, length_units_source, math_accents_module,
    math_accents_source, math_align_module, math_align_source, math_atoms_module,
    math_atoms_source, math_bigops_module, math_bigops_source, math_cases_module,
    math_cases_source, math_delimiters_module, math_delimiters_source, math_frac_module,
    math_frac_source, math_matrix_module, math_matrix_source, math_scripts_module,
    math_scripts_source, math_sqrt_module, math_sqrt_source, math_stack_module, math_stack_source,
    std_domain_natives,
};
pub use domain_native::{DomainNativeModule, DomainNativeRegistry};
pub use graphics_bridge::{
    document_from_package_entry, document_from_package_source, GraphicsBridgeError,
};
pub use japanese_bridge::{
    check_linebreak_std_parity, linebreak_parity_samples, LinebreakParitySample,
};
pub use load::{
    elaborate_with_packages, load_module_tree_with_packages, LocalPackageIndex, PackageLoadError,
    ResolvedImport,
};
pub use lockfile::{LockedPackage, Lockfile};
pub use manifest::{
    normalize_resource_path, DependencySpec, OPEN_PKG_001_REGISTRY, PackageManifest,
    ResourceCheckError,
};
pub use resolver::{resolve_packages, ResolveError, ResolvedGraph};
pub use rpi::parse_rpi_exports;
pub use rpxm::{parse_rpxm, RpxmError};
pub use target::{BuildTarget, RuntimeProfile};
pub use workspace::{
    check_package_lock_consistency, discover_workspace, find_enclosing_workspace,
    parse_workspace_rpxm, read_lock_for_package, resolve_workspace_dependencies, WorkspaceError,
    WorkspaceIndex, WorkspaceManifest,
};
