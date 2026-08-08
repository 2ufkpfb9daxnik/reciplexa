//! Lockfile for reproducible package graphs.

use serde::{Deserialize, Serialize};

use crate::manifest::PackageManifest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
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
}
