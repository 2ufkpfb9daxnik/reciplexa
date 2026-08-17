//! Package manifest, resolver, lockfile (Phase 10).
//!
//! # OPEN stubs
//!
//! [`OPEN_PKG_001_REGISTRY`] / [`WorkspaceError::RegistryUnavailable`]: network
//! registry is not implemented. Resolvers may use [`LocalRegistryMirror`] under
//! `registry/` or `RPIX_REGISTRY_ROOT` for offline `source registry` / `registry:`
//! lock entries; otherwise refuse without network I/O.
//!
//! [`content_checksum`]: lockfile content hash is **SHA-256** of the hashed
//! file (`sha256:` + 64 hex chars). Path-dep writers may fill
//! `LockedPackage.checksum` from `package.rpxm` only (CS0); mismatch is
//! reported as PKG006 via [`diagnose_lockfile_checksums`] (CS1). Registry
//! artifact integrity and full-tree hashes remain OPEN-PKG-001.

#![forbid(unsafe_code)]

pub mod build;
pub mod doc_preview;
pub mod domain_bodies;
pub mod domain_differential;
pub mod domain_length_units;
pub mod domain_native;
pub mod domain_native_env;
pub mod graphics_bridge;
pub mod japanese_bridge;
pub mod live_layout_bridge;
pub mod load;
pub mod lockfile;
pub mod manifest;
pub mod math_bridge;
pub mod registry;
pub mod resolver;
pub mod resource_value;
pub mod rpi;
pub mod rpxm;
pub mod target;
pub mod typecheck;
pub mod workspace;

pub use build::{
    diagnose_lockfile_checksums, diagnose_lockfile_registry_sources, diagnose_manifest,
    diagnose_manifest_with_root, diagnose_package_resource_replay,
    diagnose_required_lockfile_missing, BuildGraph, BuildNode, BuildNodeId, IncrementalCache,
    InvalidationKind, PackageDiagnostic,
};
pub use doc_preview::{
    count_ruby_tate_in_value, debug_layout_summary, preview_doc_text_metrics,
    preview_doc_text_metrics_from_document, preview_doc_text_metrics_from_entry,
    DocTextPreviewMetrics, DOC_TEXT_MAX_EM,
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
pub use domain_differential::{
    differential_eval_package_modules, eval_package_entry_main_dn2, DifferentialError,
};
pub use domain_length_units::populate_length_units_typed_exports;
pub use domain_native::{
    DomainNativeBindMap, DomainNativeExport, DomainNativeMode, DomainNativeModule,
    DomainNativeRegistry,
};
pub use domain_native_env::{
    build_domain_native_eval_env, build_domain_native_type_env, register_test_ping_module,
};
pub use graphics_bridge::{
    document_from_package_entry, document_from_package_source, GraphicsBridgeError,
};
pub use japanese_bridge::{
    check_linebreak_std_parity, linebreak_parity_samples, LinebreakParitySample,
};
pub use live_layout_bridge::{
    document_from_live_layout_entry, document_from_live_layout_source,
    document_from_live_layout_value, document_from_live_layout_value_with_style,
};
pub use load::{
    elaborate_with_packages, eval_package_entry_main, load_module_tree_with_packages,
    LocalPackageIndex, PackageLoadError, ResolvedImport,
};
pub use lockfile::{content_checksum, LockedPackage, Lockfile};
pub use manifest::{
    normalize_resource_path, resolve_package_resource, DependencySpec, PackageManifest,
    ResourceCheckError, ResourceResolveError, OPEN_PKG_001_CODE, OPEN_PKG_001_REGISTRY,
};
pub use math_bridge::{
    estimate_package_math_main, estimate_package_math_main_with_style, MathBridgeError,
};
pub use registry::{
    LocalRegistryMirror, RegistryResolveError, REGISTRY_LOCK_PREFIX, REGISTRY_ROOT_ENV,
};
pub use resolver::{resolve_packages, ResolveError, ResolvedGraph};
pub use resource_value::{
    find_enclosing_package_root, is_package_resource_value, materialize_package_resource,
    materialize_package_resources_in_tree, maybe_materialize_package_resources_for_entry,
    package_resource_id, resolve_resource_value, resource_file_content_hash,
    verify_package_resource_replay, ResourceValueError, PACKAGE_RESOURCE_EFFECT,
    PACKAGE_RESOURCE_TAG,
};
pub use rpi::parse_rpi_exports;
pub use rpxm::{parse_rpxm, RpxmError};
pub use target::{BuildTarget, RuntimeProfile};
pub use typecheck::{typecheck_package_source, typecheck_with_packages, PackageTypecheckError};
pub use workspace::{
    check_package_lock_consistency, discover_workspace, find_enclosing_workspace,
    parse_workspace_rpxm, read_lock_for_package, resolve_workspace_dependencies,
    verify_package_lock_for_entry, WorkspaceError, WorkspaceIndex, WorkspaceManifest,
};
