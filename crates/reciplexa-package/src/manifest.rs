//! Package manifest model.

use serde::{Deserialize, Serialize};

fn default_format_version() -> u32 {
    1
}

fn default_source_root() -> String {
    "src".into()
}

fn default_resource_root() -> String {
    "resources".into()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencySpec {
    pub name: String,
    pub version_req: String,
    pub path: Option<String>,
    /// Formal package identity when distinct from the local alias (`name`).
    #[serde(default)]
    pub package: Option<String>,
    /// Explicit dependency source: `workspace`, `registry`, or unset (prefer workspace).
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<DependencySpec>,
    /// Legacy single entry path (Phase 10). Empty for library-only packages.
    #[serde(default)]
    pub entry: String,
    #[serde(default)]
    pub targets: Vec<String>,
    /// Manifest schema version (DD-001 `format-version`).
    #[serde(default = "default_format_version")]
    pub format_version: u32,
    /// Implementation source root relative to package root (default `src`).
    #[serde(default = "default_source_root")]
    pub source_root: String,
    /// Optional interface root (default when present: `interface`).
    #[serde(default)]
    pub interface_root: Option<String>,
    /// Resource root relative to package root (default `resources`).
    #[serde(default = "default_resource_root")]
    pub resource_root: String,
    /// Outer module paths exported to dependents (DD-001 `public-modules`).
    #[serde(default)]
    pub public_modules: Vec<String>,
    /// Executable entry module paths (DD-001 `entry-points`).
    #[serde(default)]
    pub entry_points: Vec<String>,
    /// Distributed resource paths relative to `resource_root` (PKG §21.10).
    #[serde(default)]
    pub resources: Vec<String>,
}

impl Default for PackageManifest {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: String::new(),
            dependencies: Vec::new(),
            entry: String::new(),
            targets: Vec::new(),
            format_version: default_format_version(),
            source_root: default_source_root(),
            interface_root: None,
            resource_root: default_resource_root(),
            public_modules: Vec::new(),
            entry_points: Vec::new(),
            resources: Vec::new(),
        }
    }
}

/// Normalize a resource path relative to `resource_root` (PKG §21.11).
///
/// Accepts `/` or `\` separators; returns `/`-joined form. Rejects absolute
/// paths, `..`, and empty path components.
pub fn normalize_resource_path(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("resource path escapes the package resource root".into());
    }
    // Absolute: Unix `/…`, Windows drive `C:…`, or UNC / rooted `\…`.
    let first = trimmed.chars().next().unwrap_or('\0');
    if first == '/' || first == '\\' {
        return Err("resource path escapes the package resource root".into());
    }
    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        if bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
            return Err("resource path escapes the package resource root".into());
        }
    }

    let mut parts = Vec::new();
    for seg in trimmed.split(['/', '\\']) {
        if seg.is_empty() {
            return Err("resource path escapes the package resource root".into());
        }
        if seg == ".." {
            return Err("resource path escapes the package resource root".into());
        }
        if seg == "." {
            continue;
        }
        parts.push(seg);
    }
    if parts.is_empty() {
        return Err("resource path escapes the package resource root".into());
    }
    Ok(parts.join("/"))
}

impl PackageManifest {
    pub fn parse_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Resolve a public module path to a source file under this package root.
    pub fn module_source_path(
        &self,
        package_root: &std::path::Path,
        module_path: &str,
    ) -> std::path::PathBuf {
        let mut path = package_root.join(&self.source_root);
        for seg in module_path.split('/') {
            path.push(seg);
        }
        path.set_extension("rpx");
        path
    }

    /// Resolve the `.rpi` interface path for a public module (MOD §2.3), if
    /// `interface-root` is configured.
    pub fn module_interface_path(
        &self,
        package_root: &std::path::Path,
        module_path: &str,
    ) -> Option<std::path::PathBuf> {
        let iface = self.interface_root.as_ref()?;
        let mut path = package_root.join(iface);
        for seg in module_path.split('/') {
            path.push(seg);
        }
        path.set_extension("rpi");
        Some(path)
    }

    /// Absolute filesystem path for a listed resource under `resource_root`.
    pub fn resource_fs_path(
        &self,
        package_root: &std::path::Path,
        resource_path: &str,
    ) -> Result<std::path::PathBuf, String> {
        let normalized = normalize_resource_path(resource_path)?;
        let mut path = package_root.join(&self.resource_root);
        for seg in normalized.split('/') {
            path.push(seg);
        }
        Ok(path)
    }

    /// Resolve `rel` to an absolute path under this package's `resource_root`.
    ///
    /// Validates normalization (no escape) **and** membership in the manifest
    /// `resources` list (E0/E5). Language `(resource …)` / `package-resource`
    /// typing remains OPEN — this is the package-API host helper only.
    pub fn resolve_package_resource(
        &self,
        package_root: &std::path::Path,
        rel: &str,
    ) -> Result<std::path::PathBuf, ResourceResolveError> {
        resolve_package_resource(package_root, self, rel)
    }

    /// Optional FS check: every listed resource must exist under `resource_root`
    /// (static prep for PKG-12; no language `(resource …)` yet).
    pub fn check_resources_exist(
        &self,
        package_root: &std::path::Path,
    ) -> Result<(), ResourceCheckError> {
        for rel in &self.resources {
            let path = self
                .resource_fs_path(package_root, rel)
                .map_err(ResourceCheckError::InvalidPath)?;
            if !path.exists() {
                return Err(ResourceCheckError::Missing {
                    resource: rel.clone(),
                    path: path.display().to_string(),
                });
            }
        }
        Ok(())
    }
}

/// Resolve a package resource path relative to `package_root` / `resource_root`.
///
/// Requires `rel` to normalize cleanly and appear in `manifest.resources`.
/// Language `(resource "path")` elaborates to a deferred `package-resource`
/// record in language-only eval; hosts with a package root may pass the
/// record's `path` field here after load.
pub fn resolve_package_resource(
    package_root: &std::path::Path,
    manifest: &PackageManifest,
    rel: &str,
) -> Result<std::path::PathBuf, ResourceResolveError> {
    let normalized = normalize_resource_path(rel).map_err(ResourceResolveError::InvalidPath)?;
    if !manifest.resources.iter().any(|r| r == &normalized) {
        return Err(ResourceResolveError::NotListed {
            resource: normalized,
        });
    }
    manifest
        .resource_fs_path(package_root, &normalized)
        .map_err(ResourceResolveError::InvalidPath)
}

/// Error from [`resolve_package_resource`] (package API helper; not language eval).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceResolveError {
    InvalidPath(String),
    NotListed { resource: String },
}

impl std::fmt::Display for ResourceResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath(s) => write!(f, "{s}"),
            Self::NotListed { resource } => {
                write!(
                    f,
                    "package resource `{resource}` is not listed in manifest resources"
                )
            }
        }
    }
}

impl std::error::Error for ResourceResolveError {}

/// Error from optional listed-resource existence checks (PKG §21.10 / PKG-12 prep).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceCheckError {
    InvalidPath(String),
    Missing { resource: String, path: String },
}

impl std::fmt::Display for ResourceCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath(s) => write!(f, "{s}"),
            Self::Missing { resource, path } => {
                write!(
                    f,
                    "package resource does not exist: `{resource}` at `{path}`"
                )
            }
        }
    }
}

/// OPEN-PKG-001: package registry protocol / checksums are not implemented.
///
/// Path-free dependencies that do not match a workspace member, or that set
/// `source registry`, fail with [`crate::WorkspaceError::RegistryUnavailable`]
/// and must not perform network I/O.
pub const OPEN_PKG_001_REGISTRY: &str =
    "OPEN-PKG-001: package registry resolution is not implemented (no network)";
