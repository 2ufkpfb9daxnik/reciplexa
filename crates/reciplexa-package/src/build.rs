//! Build graph and incremental cache (Phase 10).

use std::collections::{BTreeMap, BTreeSet};

use crate::manifest::PackageManifest;
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

pub fn diagnose_manifest(m: &PackageManifest) -> Vec<PackageDiagnostic> {
    let mut diags = Vec::new();
    if m.name.is_empty() {
        diags.push(PackageDiagnostic {
            code: "PKG001".into(),
            message: "package name is empty".into(),
            package: None,
        });
    }
    if m.entry.is_empty() {
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
    }
    diags
}
