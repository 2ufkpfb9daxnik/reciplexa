//! Build graph and incremental cache (Phase 10).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::manifest::{PackageManifest, ResourceCheckError};
use crate::target::BuildTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidationKind {
    /// Contract/API change — invalidate dependents.
    Contract,
    /// Implementation-only change — invalidate this node only.
    Implementation,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildNodeId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildNode {
    pub id: BuildNodeId,
    pub target: BuildTarget,
    pub deps: Vec<BuildNodeId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildGraph {
    pub nodes: BTreeMap<BuildNodeId, BuildNode>,
}

impl BuildGraph {
    pub fn from_manifest(manifest: &PackageManifest, target: BuildTarget) -> Self {
        let mut g = BuildGraph::default();
        let root = BuildNodeId(format!("{}:{}", manifest.name, target.as_str()));
        let deps = manifest
            .dependencies
            .iter()
            .map(|d| BuildNodeId(format!("{}:{}", d.name, target.as_str())))
            .collect::<Vec<_>>();
        for d in &deps {
            g.nodes.insert(
                d.clone(),
                BuildNode {
                    id: d.clone(),
                    target,
                    deps: Vec::new(),
                },
            );
        }
        g.nodes.insert(
            root.clone(),
            BuildNode {
                id: root,
                target,
                deps,
            },
        );
        g
    }

    /// Deterministic topological order (sorted ids, Kahn).
    pub fn topo_order(&self) -> Vec<BuildNodeId> {
        // Indegree = number of deps present in the graph (dep must come before node).
        let mut indeg: BTreeMap<&BuildNodeId, usize> = BTreeMap::new();
        for (id, n) in &self.nodes {
            indeg.insert(
                id,
                n.deps.iter().filter(|d| self.nodes.contains_key(d)).count(),
            );
        }
        let mut ready: BTreeSet<BuildNodeId> = indeg
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(k, _)| (*k).clone())
            .collect();
        let mut out = Vec::new();
        while let Some(id) = ready.iter().next().cloned() {
            ready.remove(&id);
            out.push(id.clone());
            for (oid, n) in &self.nodes {
                if n.deps.iter().any(|d| d == &id) {
                    let e = indeg.get_mut(oid).unwrap();
                    *e = e.saturating_sub(1);
                    if *e == 0 {
                        ready.insert(oid.clone());
                    }
                }
            }
        }
        out
    }

    pub fn invalidate(
        &self,
        changed: &BuildNodeId,
        kind: InvalidationKind,
    ) -> BTreeSet<BuildNodeId> {
        let mut out = BTreeSet::new();
        out.insert(changed.clone());
        if kind == InvalidationKind::Contract {
            // dependents that list `changed` as a dep
            for (id, n) in &self.nodes {
                if n.deps.iter().any(|d| d == changed) {
                    out.insert(id.clone());
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IncrementalCache {
    /// node id → content hash
    pub hashes: BTreeMap<BuildNodeId, String>,
}

impl IncrementalCache {
    pub fn needs_rebuild(&self, id: &BuildNodeId, new_hash: &str) -> bool {
        self.hashes.get(id).map(String::as_str) != Some(new_hash)
    }

    pub fn put(&mut self, id: BuildNodeId, hash: impl Into<String>) {
        self.hashes.insert(id, hash.into());
    }

    pub fn invalidate_set(&mut self, ids: &BTreeSet<BuildNodeId>) {
        for id in ids {
            self.hashes.remove(id);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDiagnostic {
    pub code: String,
    pub message: String,
    pub package: Option<String>,
}

/// Static manifest diagnostics (PKG001–PKG003, PKG005). Does not touch the filesystem.
pub fn diagnose_manifest(m: &PackageManifest) -> Vec<PackageDiagnostic> {
    diagnose_manifest_inner(m, None)
}

/// Like [`diagnose_manifest`], plus PKG004 when a listed resource is missing under
/// `package_root` / `resource_root` (reuses the same existence rules as
/// [`PackageManifest::check_resources_exist`]).
pub fn diagnose_manifest_with_root(
    m: &PackageManifest,
    package_root: &Path,
) -> Vec<PackageDiagnostic> {
    diagnose_manifest_inner(m, Some(package_root))
}

/// PKG006: when a locked path dep carries a `checksum`, compare it to
/// [`crate::content_checksum`] of `{resolve_root}/{path}/package.rpxm`.
///
/// Missing checksums are skipped (pre-CS0 locks). Mismatch → error-severity
/// diagnostic (hosts may treat any non-empty diagnose list as failure).
pub fn diagnose_lockfile_checksums(
    lock: &crate::lockfile::Lockfile,
    resolve_root: &Path,
) -> Vec<PackageDiagnostic> {
    let mut diags = Vec::new();
    for pkg in &lock.packages {
        let Some(expected) = &pkg.checksum else {
            continue;
        };
        let Some(rel) = pkg.source.strip_prefix("path:") else {
            continue;
        };
        let rpxm = resolve_root.join(rel).join("package.rpxm");
        let actual = crate::content_checksum(&rpxm);
        if &actual != expected {
            diags.push(PackageDiagnostic {
                code: "PKG006".into(),
                message: format!(
                    "path dependency `{}` checksum mismatch: lock has `{expected}`, package.rpxm has `{actual}`",
                    pkg.name
                ),
                package: Some(pkg.name.clone()),
            });
        }
    }
    diags
}

fn diagnose_manifest_inner(
    m: &PackageManifest,
    package_root: Option<&Path>,
) -> Vec<PackageDiagnostic> {
    let mut diags = Vec::new();
    if m.name.is_empty() {
        diags.push(PackageDiagnostic {
            code: "PKG001".into(),
            message: "package name is empty".into(),
            package: None,
        });
    }
    // Library packages (public-modules only) may omit entry / entry-points.
    if m.entry.is_empty() && m.entry_points.is_empty() && m.public_modules.is_empty() {
        diags.push(PackageDiagnostic {
            code: "PKG002".into(),
            message: "entry is empty".into(),
            package: Some(m.name.clone()),
        });
    }
    let mut seen = BTreeSet::new();
    for d in &m.dependencies {
        if !seen.insert(d.name.clone()) {
            diags.push(PackageDiagnostic {
                code: "PKG003".into(),
                message: format!("duplicate dependency {}", d.name),
                package: Some(m.name.clone()),
            });
        }
        // HC13: static refuse of `source registry` (OPEN-PKG-001; no network).
        if d.source.as_deref() == Some("registry") {
            diags.push(PackageDiagnostic {
                code: "PKG005".into(),
                message: format!(
                    "dependency `{}` uses source registry ({})",
                    d.name,
                    crate::manifest::OPEN_PKG_001_CODE
                ),
                package: Some(m.name.clone()),
            });
        }
    }
    if let Some(root) = package_root {
        for rel in &m.resources {
            match m.resource_fs_path(root, rel) {
                Err(msg) => diags.push(PackageDiagnostic {
                    code: "PKG004".into(),
                    message: msg,
                    package: Some(m.name.clone()),
                }),
                Ok(path) if !path.exists() => {
                    // Same wording as ResourceCheckError::Missing.
                    let err = ResourceCheckError::Missing {
                        resource: rel.clone(),
                        path: path.display().to_string(),
                    };
                    diags.push(PackageDiagnostic {
                        code: "PKG004".into(),
                        message: err.to_string(),
                        package: Some(m.name.clone()),
                    });
                }
                Ok(_) => {}
            }
        }
    }
    diags
}
