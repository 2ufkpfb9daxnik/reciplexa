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
        let mut indeg: BTreeMap<&BuildNodeId, usize> =
            self.nodes.keys().map(|k| (k, 0)).collect();
        for n in self.nodes.values() {
            for d in &n.deps {
                if let Some(e) = indeg.get_mut(d) {
                    // edge dep -> node means dep must come first; count incoming on node
                    let _ = e;
                }
            }
        }
        // Recompute: for each node, indegree = number of deps present in graph
        for (id, n) in &self.nodes {
            *indeg.get_mut(id).unwrap() = n
                .deps
                .iter()
                .filter(|d| self.nodes.contains_key(d))
                .count();
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

    pub fn invalidate(&self, changed: &BuildNodeId, kind: InvalidationKind) -> BTreeSet<BuildNodeId> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::DependencySpec;

    #[test]
    fn contract_invalidation_wider_than_impl() {
        let m = PackageManifest {
            name: "app".into(),
            version: "1".into(),
            dependencies: vec![DependencySpec {
                name: "lib".into(),
                version_req: "1".into(),
                path: None,
            }],
            entry: "main.rpx".into(),
            targets: vec!["document".into()],
        };
        let g = BuildGraph::from_manifest(&m, BuildTarget::Document);
        let lib = BuildNodeId("lib:document".into());
        let impl_only = g.invalidate(&lib, InvalidationKind::Implementation);
        let contract = g.invalidate(&lib, InvalidationKind::Contract);
        assert_eq!(impl_only.len(), 1);
        assert!(contract.len() > impl_only.len());
    }

    #[test]
    fn topo_order_is_linear_for_chain() {
        let m = PackageManifest {
            name: "app".into(),
            version: "1".into(),
            dependencies: vec![
                DependencySpec {
                    name: "b".into(),
                    version_req: "1".into(),
                    path: None,
                },
                DependencySpec {
                    name: "a".into(),
                    version_req: "1".into(),
                    path: None,
                },
            ],
            entry: "main.rpx".into(),
            targets: vec![],
        };
        let g = BuildGraph::from_manifest(&m, BuildTarget::Document);
        let order = g.topo_order();
        assert_eq!(order.len(), 3);
        let a_pos = order.iter().position(|n| n.0 == "a:document").unwrap();
        let b_pos = order.iter().position(|n| n.0 == "b:document").unwrap();
        let app_pos = order.iter().position(|n| n.0 == "app:document").unwrap();
        assert!(a_pos < app_pos);
        assert!(b_pos < app_pos);
    }

    #[test]
    fn needs_rebuild_tracks_hash_changes() {
        let id = BuildNodeId("node:document".into());
        let mut cache = IncrementalCache::default();
        assert!(cache.needs_rebuild(&id, "abc"));
        cache.put(id.clone(), "abc");
        assert!(!cache.needs_rebuild(&id, "abc"));
        assert!(cache.needs_rebuild(&id, "def"));
    }

    #[test]
    fn diagnose_pkg001_empty_name() {
        let m = PackageManifest {
            name: "".into(),
            version: "1".into(),
            dependencies: vec![],
            entry: "main.rpx".into(),
            targets: vec![],
        };
        let diags = diagnose_manifest(&m);
        assert!(diags.iter().any(|d| d.code == "PKG001"));
    }

    #[test]
    fn diagnose_pkg002_empty_entry() {
        let m = PackageManifest {
            name: "app".into(),
            version: "1".into(),
            dependencies: vec![],
            entry: "".into(),
            targets: vec![],
        };
        let diags = diagnose_manifest(&m);
        assert!(diags.iter().any(|d| d.code == "PKG002" && d.package == Some("app".into())));
    }

    #[test]
    fn diagnose_pkg003_duplicate_dependency() {
        let m = PackageManifest {
            name: "app".into(),
            version: "1".into(),
            dependencies: vec![
                DependencySpec {
                    name: "lib".into(),
                    version_req: "1".into(),
                    path: None,
                },
                DependencySpec {
                    name: "lib".into(),
                    version_req: "2".into(),
                    path: None,
                },
            ],
            entry: "main.rpx".into(),
            targets: vec![],
        };
        let diags = diagnose_manifest(&m);
        assert!(diags.iter().any(|d| d.code == "PKG003"));
    }
}
