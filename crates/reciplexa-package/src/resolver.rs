//! Deterministic package resolution.

use std::collections::BTreeMap;

use reciplexa_bind::package::{resolve_package, PackageResolveResult};
use reciplexa_identity::package::PackageInstanceId;

use crate::lockfile::Lockfile;
use crate::manifest::PackageManifest;
use crate::target::{BuildTarget, RuntimeProfile};

#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub manifest: PackageManifest,
    pub resolve: PackageResolveResult,
}

#[derive(Debug, Clone)]
pub struct ResolvedGraph {
    pub packages: Vec<ResolvedPackage>,
    pub lockfile: Lockfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    DuplicateName(String),
    MissingEntry(String),
    TargetValidation(String),
}

/// Resolve packages deterministically (sorted by name).
pub fn resolve_packages(
    manifests: Vec<PackageManifest>,
    sources: &BTreeMap<String, Vec<(String, &str)>>,
    target: BuildTarget,
    profile: &RuntimeProfile,
) -> Result<ResolvedGraph, ResolveError> {
    let _ = profile;
    let mut sorted = manifests;
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    let mut seen = std::collections::HashSet::new();
    let mut packages = Vec::new();
    for m in &sorted {
        if !seen.insert(m.name.clone()) {
            return Err(ResolveError::DuplicateName(m.name.clone()));
        }
        target
            .validate_entry(&m.entry)
            .map_err(ResolveError::TargetValidation)?;
        let units = sources.get(&m.name).cloned().unwrap_or_default();
        if units.is_empty() {
            return Err(ResolveError::MissingEntry(m.entry.clone()));
        }
        let refs: Vec<(String, &str)> = units;
        let resolve = resolve_package(PackageInstanceId::new(packages.len() as u64 + 1), &refs);
        packages.push(ResolvedPackage {
            manifest: m.clone(),
            resolve,
        });
    }
    let lockfile = Lockfile::from_graph(&sorted);
    Ok(ResolvedGraph { packages, lockfile })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn resolution_is_order_independent() {
        let m1 = PackageManifest {
            name: "b".into(),
            version: "1.0".into(),
            dependencies: vec![],
            entry: "main".into(),
            targets: vec!["document".into()],
        };
        let m2 = PackageManifest {
            name: "a".into(),
            version: "1.0".into(),
            dependencies: vec![],
            entry: "main".into(),
            targets: vec!["document".into()],
        };
        let mut sources = BTreeMap::new();
        sources.insert("a".into(), vec![("main".into(), "(page a4)")]);
        sources.insert("b".into(), vec![("main".into(), "(page a4)")]);
        let g1 = resolve_packages(
            vec![m1.clone(), m2.clone()],
            &sources,
            BuildTarget::Document,
            &RuntimeProfile::document(),
        )
        .unwrap();
        let g2 = resolve_packages(
            vec![m2, m1],
            &sources,
            BuildTarget::Document,
            &RuntimeProfile::document(),
        )
        .unwrap();
        assert_eq!(g1.lockfile, g2.lockfile);
    }
}
