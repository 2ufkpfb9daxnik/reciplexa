//! Package manifest model.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencySpec {
    pub name: String,
    pub version_req: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<DependencySpec>,
    pub entry: String,
    pub targets: Vec<String>,
}

impl PackageManifest {
    pub fn parse_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_garbage_json() {
        assert!(PackageManifest::parse_json("not-json").is_err());
        assert!(PackageManifest::parse_json("{}").is_err());
    }

    #[test]
    fn parse_json_roundtrip() {
        let m = PackageManifest {
            name: "demo".into(),
            version: "0.1.0".into(),
            dependencies: vec![DependencySpec {
                name: "util".into(),
                version_req: "1.0".into(),
                path: Some("../util".into()),
            }],
            entry: "main.rpx".into(),
            targets: vec!["document".into()],
        };
        let json = serde_json::to_string(&m).unwrap();
        let parsed = PackageManifest::parse_json(&json).unwrap();
        assert_eq!(parsed, m);
    }
}
