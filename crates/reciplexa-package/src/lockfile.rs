//! Lockfile for reproducible package graphs.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::manifest::{DependencySpec, PackageManifest};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    /// `workspace`, `path:<rel>`, or later `registry:…` (OPEN-PKG-001).
    pub source: String,
    /// Direct dependency edges (formal package names), for path-dep graphs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    /// Optional content checksum stub (OPEN-PKG-001 / path-dep CS0).
    ///
    /// For path deps, writers may fill via [`content_checksum`] of that
    /// package's `package.rpxm` (not the full tree). Stub hash — see OPEN
    /// note on [`content_checksum`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// Compute a lockfile content-checksum string for `path`.
///
/// **OPEN:** No `blake3` / `sha2` crate is in the workspace yet, so this is a
/// non-cryptographic FNV-1a 64-bit stub (`stub-fnv1a64:…`). Replace with
/// blake3 or sha256 when a hash dependency is added; **do not treat as
/// registry integrity**. Read errors become `stub-error:…`.
pub fn content_checksum(path: impl AsRef<Path>) -> String {
    match fs::read(path.as_ref()) {
        Ok(bytes) => format!("stub-fnv1a64:{:016x}", fnv1a64(&bytes)),
        Err(e) => format!("stub-error:{e}"),
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lockfile {
    pub packages: Vec<LockedPackage>,
}

impl Lockfile {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn from_graph(manifests: &[PackageManifest]) -> Self {
        let packages = manifests
            .iter()
            .map(|m| LockedPackage {
                name: m.name.clone(),
                version: m.version.clone(),
                source: "workspace".into(),
                dependencies: m
                    .dependencies
                    .iter()
                    .map(|d| d.package.clone().unwrap_or_else(|| d.name.clone()))
                    .collect(),
                checksum: None,
            })
            .collect();
        Self { packages }
    }

    /// Slice B+: lock a consumer plus its path dependencies (no registry).
    ///
    /// Records `path:<rel>` sources and dependency edges; suitable as a
    /// root `rpx.lock` for a single-package consumer or workspace root.
    /// Does **not** fill checksums (no package roots). Prefer
    /// [`Self::from_consumer_with_roots`] when writing a lock that should
    /// carry path-dep `package.rpxm` stubs (CS0).
    pub fn from_consumer(
        consumer: &PackageManifest,
        deps: &[(&DependencySpec, &PackageManifest)],
    ) -> Self {
        let triples: Vec<(&DependencySpec, &PackageManifest, Option<&Path>)> =
            deps.iter().map(|(s, m)| (*s, *m, None)).collect();
        Self::from_consumer_with_roots(consumer, &triples)
    }

    /// Like [`Self::from_consumer`], and for each path dep whose package root
    /// is provided, fills `checksum` from [`content_checksum`] of
    /// `{root}/package.rpxm` (manifest file only — not the full tree).
    ///
    /// Always fills the stub when a root is given (no feature gate). Missing
    /// roots leave `checksum: None` (same as pre-CS0 writers).
    pub fn from_consumer_with_roots(
        consumer: &PackageManifest,
        deps: &[(&DependencySpec, &PackageManifest, Option<&Path>)],
    ) -> Self {
        let mut packages = vec![LockedPackage {
            name: consumer.name.clone(),
            version: consumer.version.clone(),
            source: "workspace".into(),
            dependencies: deps
                .iter()
                .map(|(_, m, _)| m.name.clone())
                .collect::<Vec<_>>(),
            checksum: None,
        }];
        for (spec, manifest, root) in deps {
            let source = match &spec.path {
                Some(p) => format!("path:{p}"),
                None => "workspace".into(),
            };
            let checksum = if spec.path.is_some() {
                root.map(|r| content_checksum(r.join("package.rpxm")))
            } else {
                None
            };
            packages.push(LockedPackage {
                name: manifest.name.clone(),
                version: manifest.version.clone(),
                source,
                dependencies: manifest
                    .dependencies
                    .iter()
                    .filter(|d| d.path.is_some())
                    .map(|d| d.package.clone().unwrap_or_else(|| d.name.clone()))
                    .collect(),
                checksum,
            });
        }
        packages.sort_by(|a, b| a.name.cmp(&b.name));
        Self { packages }
    }

    /// PKG §17.5: exact version of each locked path dep must satisfy the
    /// consumer manifest's version requirement (exact match / `*` only for now).
    pub fn is_consistent_with_consumer(&self, consumer: &PackageManifest) -> Result<(), String> {
        for dep in &consumer.dependencies {
            if dep.path.is_none() {
                continue;
            }
            let formal = dep.package.as_deref().unwrap_or(dep.name.as_str());
            let Some(locked) = self.packages.iter().find(|p| p.name == formal) else {
                return Err(format!(
                    "lockfile is not consistent with the package manifest: missing `{formal}`"
                ));
            };
            if !locked.source.starts_with("path:") {
                return Err(format!(
                    "lockfile is not consistent with the package manifest: `{formal}` expected path source"
                ));
            }
            if dep.version_req != "*" && dep.version_req != locked.version {
                return Err(format!(
                    "lockfile is not consistent with the package manifest: `{formal}` locked as `{}` but manifest wants `{}`",
                    locked.version, dep.version_req
                ));
            }
        }
        Ok(())
    }

    pub fn write_rpx_lock(&self, path: impl AsRef<Path>) -> Result<(), String> {
        // `LockedPackage` is a closed Serialize schema; serialization cannot fail.
        let json = self
            .to_json()
            .unwrap_or_else(|_| unreachable!("Lockfile serde"));
        fs::write(path.as_ref(), json).map_err(|e| format!("write lockfile: {e}"))
    }

    pub fn read_rpx_lock(path: impl AsRef<Path>) -> Result<Self, String> {
        let src = fs::read_to_string(path.as_ref()).map_err(|e| format!("read lockfile: {e}"))?;
        Self::from_json(&src).map_err(|e| e.to_string())
    }
}
