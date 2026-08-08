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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::PackageManifest;

    #[test]
    fn rejects_garbage_json() {
        assert!(Lockfile::from_json("{not json").is_err());
        assert!(Lockfile::from_json("[]").is_err());
    }

    #[test]
    fn json_roundtrip() {
        let lf = Lockfile {
            packages: vec![LockedPackage {
                name: "demo".into(),
                version: "1.0".into(),
                source: "workspace".into(),
            }],
        };
        let json = lf.to_json().unwrap();
        let parsed = Lockfile::from_json(&json).unwrap();
        assert_eq!(parsed, lf);
    }

    #[test]
    fn from_graph_matches_manifests() {
        let manifests = vec![PackageManifest {
            name: "a".into(),
            version: "0.1".into(),
            dependencies: vec![],
            entry: "main".into(),
            targets: vec![],
        }];
        let lf = Lockfile::from_graph(&manifests);
        assert_eq!(lf.packages[0].name, "a");
        assert_eq!(lf.packages[0].version, "0.1");
        assert_eq!(lf.packages[0].source, "workspace");
    }
}
