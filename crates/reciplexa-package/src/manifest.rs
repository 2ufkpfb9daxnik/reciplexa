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
        }
    }
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
}
