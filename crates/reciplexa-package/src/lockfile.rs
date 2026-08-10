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
            })
            .collect();
        Self { packages }
    }

    /// Slice B: lock a consumer plus its path dependencies (no registry).
    pub fn from_consumer(
        consumer: &PackageManifest,
        deps: &[(&DependencySpec, &PackageManifest)],
    ) -> Self {
        let mut packages = vec![LockedPackage {
            name: consumer.name.clone(),
            version: consumer.version.clone(),
            source: "workspace".into(),
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
            });
        }
        packages.sort_by(|a, b| a.name.cmp(&b.name));
        Self { packages }
    }

    pub fn write_rpx_lock(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let json = self.to_json().map_err(|e| e.to_string())?;
        fs::write(path.as_ref(), json).map_err(|e| format!("write lockfile: {e}"))
    }

    pub fn read_rpx_lock(path: impl AsRef<Path>) -> Result<Self, String> {
        let src = fs::read_to_string(path.as_ref()).map_err(|e| format!("read lockfile: {e}"))?;
        Self::from_json(&src).map_err(|e| e.to_string())
    }
}
