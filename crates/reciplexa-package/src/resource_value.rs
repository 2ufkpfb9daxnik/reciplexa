//! Materialize language `package-resource` records when a package root is known.
//!
//! Language `(resource "rel")` elaborates to a deferred tagged record
//! (`tag "package-resource"`, `path`, `note`). Hosts with a package root +
//! manifest call [`materialize_package_resource`] / [`resolve_resource_value`]
//! to obtain an absolute FS path via [`resolve_package_resource`].

use std::path::{Path, PathBuf};

use reciplexa_eval::RuntimeValue;

use crate::manifest::{resolve_package_resource, PackageManifest, ResourceResolveError};

/// Error when a runtime value cannot be materialized as a package resource path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceValueError {
    /// Value is not a `package-resource` record (or fields are wrong shape).
    NotPackageResource(String),
    /// Path failed [`resolve_package_resource`] validation / listing.
    Resolve(ResourceResolveError),
}

impl std::fmt::Display for ResourceValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotPackageResource(s) => write!(f, "{s}"),
            Self::Resolve(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ResourceValueError {}

impl From<ResourceResolveError> for ResourceValueError {
    fn from(e: ResourceResolveError) -> Self {
        Self::Resolve(e)
    }
}

/// If `v` is a `package-resource` record, resolve its `path` under `package_root`.
///
/// Returns the absolute path on success. Does not rewrite `v`; callers keep the
/// deferred record for language identity and use this for host FS access.
pub fn materialize_package_resource(
    v: &RuntimeValue,
    package_root: &Path,
    manifest: &PackageManifest,
) -> Result<PathBuf, ResourceValueError> {
    let fields = match v {
        RuntimeValue::Record(fields) => fields.as_slice(),
        _ => {
            return Err(ResourceValueError::NotPackageResource(
                "expected package-resource record".into(),
            ));
        }
    };
    let tag = fields
        .iter()
        .find(|(k, _)| k == "tag")
        .and_then(|(_, v)| match v {
            RuntimeValue::String(s) => Some(s.as_str()),
            _ => None,
        });
    if tag != Some("package-resource") {
        return Err(ResourceValueError::NotPackageResource(format!(
            "expected tag \"package-resource\", got {tag:?}"
        )));
    }
    let rel = fields
        .iter()
        .find(|(k, _)| k == "path")
        .and_then(|(_, v)| match v {
            RuntimeValue::String(s) => Some(s.as_str()),
            _ => None,
        })
        .ok_or_else(|| {
            ResourceValueError::NotPackageResource(
                "package-resource record missing string `path`".into(),
            )
        })?;
    resolve_package_resource(package_root, manifest, rel).map_err(Into::into)
}

/// Alias for [`materialize_package_resource`] (host-facing name).
pub fn resolve_resource_value(
    v: &RuntimeValue,
    package_root: &Path,
    manifest: &PackageManifest,
) -> Result<PathBuf, ResourceValueError> {
    materialize_package_resource(v, package_root, manifest)
}
