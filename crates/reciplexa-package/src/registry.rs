//! Offline local registry mirror (OPEN-PKG-001 light).
//!
//! Network registry protocol is not implemented. Hosts may vendor packages under
//! `{workspace|package}/registry/{name}/{version}/package.rpxm` or set
//! `RPIX_REGISTRY_ROOT` to resolve `source registry` / `registry:` lock entries
//! without network I/O.

use std::fs;
use std::path::{Path, PathBuf};

use crate::lockfile::content_checksum;
use crate::manifest::PackageManifest;
use crate::rpxm::parse_rpxm;

/// Lock / resolve source prefix for mirrored registry packages.
pub const REGISTRY_LOCK_PREFIX: &str = "registry:";

/// Environment variable overriding the local registry mirror root directory.
pub const REGISTRY_ROOT_ENV: &str = "RPIX_REGISTRY_ROOT";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRegistryMirror {
    root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryResolveError {
    NotFound(String),
    Manifest(String),
    VersionMismatch {
        package: String,
        wanted: String,
        found: String,
    },
}

impl std::fmt::Display for RegistryResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(s) | Self::Manifest(s) => write!(f, "{s}"),
            Self::VersionMismatch {
                package,
                wanted,
                found,
            } => write!(
                f,
                "registry package `{package}` version mismatch: wanted `{wanted}`, mirror has `{found}`"
            ),
        }
    }
}

impl std::error::Error for RegistryResolveError {}

impl LocalRegistryMirror {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Discover a mirror near `start`: `RPIX_REGISTRY_ROOT`, then walk parents for `registry/`.
    pub fn discover_near(start: &Path) -> Option<Self> {
        if let Ok(env) = std::env::var(REGISTRY_ROOT_ENV) {
            let p = PathBuf::from(env);
            if p.is_dir() {
                return Some(Self::new(p));
            }
        }
        let mut cur = if start.is_file() {
            start.parent()?.to_path_buf()
        } else {
            start.to_path_buf()
        };
        loop {
            let candidate = cur.join("registry");
            if candidate.is_dir() {
                return Some(Self::new(candidate));
            }
            cur = cur.parent()?.to_path_buf();
        }
    }

    pub fn package_dir(&self, name: &str, version: &str) -> PathBuf {
        self.root.join(name).join(version)
    }

    pub fn lock_source(name: &str, version: &str) -> String {
        format!("{REGISTRY_LOCK_PREFIX}{name}@{version}")
    }

    pub fn parse_lock_source(source: &str) -> Option<(String, String)> {
        let rest = source.strip_prefix(REGISTRY_LOCK_PREFIX)?;
        let (name, version) = rest.split_once('@')?;
        if name.is_empty() || version.is_empty() {
            return None;
        }
        Some((name.to_string(), version.to_string()))
    }

    /// Resolve a formal package name + version requirement from the mirror.
    pub fn resolve(
        &self,
        formal: &str,
        version_req: &str,
    ) -> Result<(PathBuf, PackageManifest), RegistryResolveError> {
        let version_req = version_req.trim_matches('"');
        let entries = fs::read_dir(self.root.join(formal)).map_err(|e| {
            RegistryResolveError::NotFound(format!(
                "registry mirror has no package `{formal}` under `{}` ({e})",
                self.root.display()
            ))
        })?;
        let mut candidates = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| RegistryResolveError::NotFound(e.to_string()))?;
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let version = entry.file_name().to_string_lossy().into_owned();
            if version_req != "*" && version_req != version {
                continue;
            }
            let root = entry.path();
            let rpxm = root.join("package.rpxm");
            if !rpxm.is_file() {
                continue;
            }
            let src = fs::read_to_string(&rpxm).map_err(|e| {
                RegistryResolveError::Manifest(format!("read `{}`: {e}", rpxm.display()))
            })?;
            let manifest = parse_rpxm(&src).map_err(|e| {
                RegistryResolveError::Manifest(format!("parse `{}`: {e:?}", rpxm.display()))
            })?;
            if manifest.name != formal {
                return Err(RegistryResolveError::Manifest(format!(
                    "registry mirror `{formal}/{version}` manifest name is `{}`",
                    manifest.name
                )));
            }
            if version_req == "*" || manifest.version == version_req {
                candidates.push((root, manifest));
            }
        }
        if candidates.len() == 1 {
            return Ok(candidates.remove(0));
        }
        if candidates.is_empty() {
            return Err(RegistryResolveError::NotFound(format!(
                "registry mirror has no `{formal}` version matching `{version_req}`"
            )));
        }
        Err(RegistryResolveError::NotFound(format!(
            "registry mirror has ambiguous `{formal}` versions for `{version_req}`"
        )))
    }

    pub fn locked_package(
        &self,
        formal: &str,
        version_req: &str,
    ) -> Result<crate::lockfile::LockedPackage, RegistryResolveError> {
        let (root, manifest) = self.resolve(formal, version_req)?;
        Ok(crate::lockfile::LockedPackage {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            source: Self::lock_source(&manifest.name, &manifest.version),
            dependencies: manifest
                .dependencies
                .iter()
                .filter(|d| d.path.is_some())
                .map(|d| d.package.clone().unwrap_or_else(|| d.name.clone()))
                .collect(),
            checksum: Some(content_checksum(root.join("package.rpxm"))),
        })
    }
}
