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
            })
            .collect();
        Self { packages }
    }

    /// Slice B+: lock a consumer plus its path dependencies (no registry).
    ///
    /// Records `path:<rel>` sources and dependency edges; suitable as a
    /// root `rpx.lock` for a single-package consumer or workspace root.
    pub fn from_consumer(
        consumer: &PackageManifest,
        deps: &[(&DependencySpec, &PackageManifest)],
    ) -> Self {
        let mut packages = vec![LockedPackage {
            name: consumer.name.clone(),
            version: consumer.version.clone(),
            source: "workspace".into(),
            dependencies: deps.iter().map(|(_, m)| m.name.clone()).collect::<Vec<_>>(),
        }];
        for (spec, manifest) in deps {
            let source = match &spec.path {
                Some(p) => format!("path:{p}"),
                None => "workspace".into(),
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
        let json = self.to_json().unwrap_or_else(|_| unreachable!("Lockfile serde"));
        fs::write(path.as_ref(), json).map_err(|e| format!("write lockfile: {e}"))
    }

    pub fn read_rpx_lock(path: impl AsRef<Path>) -> Result<Self, String> {
        let src = fs::read_to_string(path.as_ref()).map_err(|e| format!("read lockfile: {e}"))?;
        Self::from_json(&src).map_err(|e| e.to_string())
    }
}
