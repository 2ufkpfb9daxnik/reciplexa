//! `workspace.rpxm` parser and member discovery (PKG-001 DD-001 §20).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::build::diagnose_lockfile_checksums;
use crate::lockfile::{content_checksum, LockedPackage, Lockfile};
use crate::manifest::PackageManifest;
use crate::resource_value::find_enclosing_package_root;
use crate::rpxm::{parse_rpxm, tokenize, RpxmError};

/// Parsed workspace manifest (`workspace.rpxm`).
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceManifest {
    pub format_version: u32,
    /// Member paths relative to the workspace root (no globs in v1).
    pub members: Vec<String>,
    /// Optional workspace-level path-free deps (OPEN-PKG registry refuse surface).
    pub dependencies: Vec<crate::manifest::DependencySpec>,
}

/// Indexed workspace: root path, manifest, and each member's package root + manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceIndex {
    pub root: PathBuf,
    pub manifest: WorkspaceManifest,
    /// Formal package name → (member package root, package.rpxm).
    pub members: BTreeMap<String, (PathBuf, PackageManifest)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceError {
    Io(String),
    Manifest(RpxmError),
    NestedWorkspace(String),
    DuplicateName(String),
    MissingMember(String),
    MemberLocalLock(String),
    Lock(String),
    DependencyCycle(String),
    VersionMismatch(String),
    /// OPEN-PKG-001: registry protocol is not implemented (no network).
    RegistryUnavailable(String),
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(s)
            | Self::NestedWorkspace(s)
            | Self::DuplicateName(s)
            | Self::MissingMember(s)
            | Self::MemberLocalLock(s)
            | Self::Lock(s)
            | Self::DependencyCycle(s)
            | Self::VersionMismatch(s)
            | Self::RegistryUnavailable(s) => write!(f, "{s}"),
            Self::Manifest(e) => write!(f, "workspace manifest error: {e:?}"),
        }
    }
}

impl WorkspaceError {
    /// Structured error code when applicable (e.g. [`crate::OPEN_PKG_001_CODE`]).
    pub fn code(&self) -> Option<&'static str> {
        match self {
            Self::RegistryUnavailable(_) => Some(crate::manifest::OPEN_PKG_001_CODE),
            _ => None,
        }
    }

    /// OPEN-PKG-001 refusal with a human detail (no network I/O).
    pub fn registry_unavailable(detail: impl Into<String>) -> Self {
        let detail = detail.into();
        Self::RegistryUnavailable(format!(
            "{}: {detail}",
            crate::manifest::OPEN_PKG_001_REGISTRY
        ))
    }
}

impl From<RpxmError> for WorkspaceError {
    fn from(e: RpxmError) -> Self {
        Self::Manifest(e)
    }
}

/// Parse a DD-001 `workspace.rpxm` document.
///
/// ```text
/// (workspace
///   format-version 1
///   (members
///     "document-core"
///     "document-render"))
/// ```
pub fn parse_workspace_rpxm(src: &str) -> Result<WorkspaceManifest, RpxmError> {
    let tokens = tokenize(src).map_err(RpxmError::Syntax)?;
    if tokens.is_empty() {
        return Err(RpxmError::Empty);
    }
    if tokens.len() < 2 || tokens[0] != "(" || tokens[1] != "workspace" {
        return Err(RpxmError::Syntax(
            "expected `(workspace …)` as workspace.rpxm root".into(),
        ));
    }

    let mut format_version = 1u32;
    let mut members = Vec::new();
    let mut dependencies = Vec::new();
    let mut i = 2usize;
    while i < tokens.len() {
        match tokens[i].as_str() {
            "format-version" if i + 1 < tokens.len() => {
                format_version = tokens[i + 1]
                    .parse()
                    .map_err(|_| RpxmError::Syntax("invalid format-version".into()))?;
                if format_version != 1 {
                    return Err(RpxmError::UnsupportedFormatVersion(format_version));
                }
                i += 2;
            }
            "(" if i + 1 < tokens.len() && tokens[i + 1] == "members" => {
                i += 2;
                while i < tokens.len() && tokens[i] != ")" {
                    let m = tokens[i].trim_matches('"').to_string();
                    if m.is_empty() {
                        return Err(RpxmError::Syntax("empty workspace member".into()));
                    }
                    members.push(m);
                    i += 1;
                }
                if i >= tokens.len() || tokens[i] != ")" {
                    return Err(RpxmError::Syntax("unclosed (members …)".into()));
                }
                i += 1;
            }
            "(" if i + 1 < tokens.len() && tokens[i + 1] == "dependencies" => {
                let (deps, next) = crate::rpxm::parse_dependencies_block(&tokens, i)?;
                dependencies.extend(deps);
                i = next;
            }
            ")" => break,
            other => {
                return Err(RpxmError::UnknownField(format!(
                    "unknown workspace field `{other}`"
                )));
            }
        }
    }

    if members.is_empty() {
        return Err(RpxmError::Syntax(
            "workspace.rpxm requires a non-empty (members …) list".into(),
        ));
    }

    Ok(WorkspaceManifest {
        format_version,
        members,
        dependencies,
    })
}

/// Discover a workspace at `root`: parse `workspace.rpxm`, load each member's
/// `package.rpxm`, require unique formal names, and reject nested `workspace.rpxm`
/// inside member directories (PKG §20.4–20.8).
pub fn discover_workspace(root: impl AsRef<Path>) -> Result<WorkspaceIndex, WorkspaceError> {
    let root = root.as_ref();
    let ws_path = root.join("workspace.rpxm");
    let src = fs::read_to_string(&ws_path)
        .map_err(|e| WorkspaceError::Io(format!("read `{}`: {e}", ws_path.display())))?;
    let manifest = parse_workspace_rpxm(&src)?;

    let mut members: BTreeMap<String, (PathBuf, PackageManifest)> = BTreeMap::new();
    for rel in &manifest.members {
        let member_root = root.join(rel);
        if !member_root.is_dir() {
            return Err(WorkspaceError::MissingMember(format!(
                "workspace member `{rel}` is not a directory at `{}`",
                member_root.display()
            )));
        }
        let nested_ws = member_root.join("workspace.rpxm");
        if nested_ws.is_file() {
            return Err(WorkspaceError::NestedWorkspace(format!(
                "nested workspace.rpxm is not allowed in member `{rel}` (`{}`)",
                nested_ws.display()
            )));
        }
        let pkg_path = member_root.join("package.rpxm");
        if !pkg_path.is_file() {
            return Err(WorkspaceError::MissingMember(format!(
                "workspace member `{rel}` missing package.rpxm at `{}`",
                pkg_path.display()
            )));
        }
        let pkg_src = fs::read_to_string(&pkg_path)
            .map_err(|e| WorkspaceError::Io(format!("read `{}`: {e}", pkg_path.display())))?;
        let package = parse_rpxm(&pkg_src).map_err(WorkspaceError::Manifest)?;
        if let Some((existing, _)) = members.get(&package.name) {
            return Err(WorkspaceError::DuplicateName(format!(
                "duplicate package name `{}` in workspace members `{}` and `{}`",
                package.name,
                existing.display(),
                member_root.display()
            )));
        }
        members.insert(package.name.clone(), (member_root, package));
    }

    Ok(WorkspaceIndex {
        root: root.to_path_buf(),
        manifest,
        members,
    })
}

impl WorkspaceIndex {
    /// Path of the shared workspace-root lockfile (`rpx.lock`).
    pub fn lock_path(&self) -> PathBuf {
        self.root.join("rpx.lock")
    }

    /// Reject member-local `rpx.lock` files (PKG §21.1).
    pub fn reject_member_local_locks(&self) -> Result<(), WorkspaceError> {
        for (name, (member_root, _)) in &self.members {
            let local = member_root.join("rpx.lock");
            if local.is_file() {
                return Err(WorkspaceError::MemberLocalLock(format!(
                    "workspace member `{name}` must not have its own rpx.lock at `{}`; use the workspace root lock",
                    local.display()
                )));
            }
        }
        Ok(())
    }

    /// Build a shared lock covering all workspace members (`source: workspace`).
    ///
    /// Fills each member's `checksum` from [`content_checksum`] of that
    /// member's `package.rpxm` (manifest file only — stub hash, not the full
    /// tree). PKG006 still only compares `path:` sources; workspace checksums
    /// are recorded for later integrity work.
    pub fn build_lock(&self) -> Lockfile {
        let mut packages: Vec<LockedPackage> = self
            .members
            .values()
            .map(|(root, m)| LockedPackage {
                name: m.name.clone(),
                version: m.version.clone(),
                source: "workspace".into(),
                dependencies: m
                    .dependencies
                    .iter()
                    .map(|d| d.package.clone().unwrap_or_else(|| d.name.clone()))
                    .collect(),
                checksum: Some(content_checksum(root.join("package.rpxm"))),
            })
            .collect();
        packages.sort_by(|a, b| a.name.cmp(&b.name));
        Lockfile { packages }
    }

    /// Write the shared root `rpx.lock` after rejecting member-local locks.
    pub fn write_lock(&self) -> Result<Lockfile, WorkspaceError> {
        self.reject_member_local_locks()?;
        let lock = self.build_lock();
        lock.write_rpx_lock(self.lock_path())
            .map_err(WorkspaceError::Lock)?;
        Ok(lock)
    }

    /// Read the shared root `rpx.lock`, rejecting member-local locks.
    pub fn read_lock(&self) -> Result<Lockfile, WorkspaceError> {
        self.reject_member_local_locks()?;
        Lockfile::read_rpx_lock(self.lock_path()).map_err(WorkspaceError::Lock)
    }

    /// True if `package_root` is one of this workspace's member directories.
    pub fn contains_member_root(&self, package_root: &Path) -> bool {
        self.members
            .values()
            .any(|(root, _)| roots_equal(root, package_root))
    }
}

fn roots_equal(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => a == b,
    }
}

/// Walk ancestors of `start` for `workspace.rpxm` and discover that workspace (PKG §21.2).
///
/// Returns `Ok(None)` when no enclosing workspace exists.
pub fn find_enclosing_workspace(
    start: impl AsRef<Path>,
) -> Result<Option<WorkspaceIndex>, WorkspaceError> {
    let mut cur = start.as_ref().to_path_buf();
    if cur.is_file() {
        if let Some(parent) = cur.parent() {
            cur = parent.to_path_buf();
        }
    }
    loop {
        let candidate = cur.join("workspace.rpxm");
        if candidate.is_file() {
            return discover_workspace(&cur).map(Some);
        }
        match cur.parent() {
            Some(parent) if parent != cur => cur = parent.to_path_buf(),
            _ => return Ok(None),
        }
    }
}

/// Resolve the lockfile path and contents for a package root (PKG §21.2).
///
/// When the package is a workspace member, uses the workspace-root `rpx.lock`
/// (and rejects a member-local lock). Otherwise reads `package_root/rpx.lock`.
pub fn read_lock_for_package(
    package_root: impl AsRef<Path>,
) -> Result<(PathBuf, Lockfile), WorkspaceError> {
    let package_root = package_root.as_ref();
    if let Some(ws) = find_enclosing_workspace(package_root)? {
        if ws.contains_member_root(package_root) {
            let lock = ws.read_lock()?;
            return Ok((ws.lock_path(), lock));
        }
    }
    let path = package_root.join("rpx.lock");
    let lock = Lockfile::read_rpx_lock(&path).map_err(WorkspaceError::Lock)?;
    Ok((path, lock))
}

/// Check that the package's manifest is consistent with the lock resolved for
/// its root (workspace root lock when in a workspace).
pub fn check_package_lock_consistency(
    package_root: impl AsRef<Path>,
) -> Result<(), WorkspaceError> {
    let package_root = package_root.as_ref();
    let manifest_path = package_root.join("package.rpxm");
    let src = fs::read_to_string(&manifest_path)
        .map_err(|e| WorkspaceError::Io(format!("read `{}`: {e}", manifest_path.display())))?;
    let manifest = parse_rpxm(&src).map_err(WorkspaceError::Manifest)?;
    let (_lock_path, lock) = read_lock_for_package(package_root)?;

    // Member itself must appear in the workspace lock with matching version.
    if let Some(ws) = find_enclosing_workspace(package_root)? {
        if ws.contains_member_root(package_root) {
            let Some(locked) = lock.packages.iter().find(|p| p.name == manifest.name) else {
                return Err(WorkspaceError::Lock(format!(
                    "lockfile is not consistent with the package manifest: missing `{}`",
                    manifest.name
                )));
            };
            if locked.version != manifest.version {
                return Err(WorkspaceError::Lock(format!(
                    "lockfile is not consistent with the package manifest: `{}` locked as `{}` but package is `{}`",
                    manifest.name, locked.version, manifest.version
                )));
            }
        }
    }

    lock.is_consistent_with_consumer(&manifest)
        .map_err(WorkspaceError::Lock)
}

/// When `entry_path` sits under a package root that has an `rpx.lock` (local or
/// workspace-root), verify manifest/lock consistency and PKG006 path-dep
/// checksums before package modules load.
pub fn verify_package_lock_for_entry(entry_path: &Path) -> Result<(), WorkspaceError> {
    let Some(package_root) = find_enclosing_package_root(entry_path) else {
        return Ok(());
    };
    if !lock_file_present(&package_root)? {
        return Ok(());
    }
    check_package_lock_consistency(&package_root)?;
    let resolve_root = lock_diagnose_root(&package_root)?;
    let (_lock_path, lock) = read_lock_for_package(&package_root)?;
    if let Some(d) = diagnose_lockfile_checksums(&lock, &resolve_root)
        .into_iter()
        .next()
    {
        return Err(WorkspaceError::Lock(format!("{}: {}", d.code, d.message)));
    }
    Ok(())
}

fn lock_file_present(package_root: &Path) -> Result<bool, WorkspaceError> {
    if let Some(ws) = find_enclosing_workspace(package_root)? {
        if ws.contains_member_root(package_root) {
            return Ok(ws.lock_path().is_file());
        }
    }
    Ok(package_root.join("rpx.lock").is_file())
}

fn lock_diagnose_root(package_root: &Path) -> Result<PathBuf, WorkspaceError> {
    if let Some(ws) = find_enclosing_workspace(package_root)? {
        if ws.contains_member_root(package_root) {
            return Ok(ws.root.clone());
        }
    }
    Ok(package_root.to_path_buf())
}

fn version_satisfies(req: &str, version: &str) -> bool {
    let req = req.trim_matches('"');
    let version = version.trim_matches('"');
    req == "*" || req == version
}

fn formal_dep_name(dep: &crate::manifest::DependencySpec) -> &str {
    dep.package.as_deref().unwrap_or(dep.name.as_str())
}

/// Resolve path-free member dependencies preferring workspace members (PKG §21.3–21.6).
///
/// - Matching member name+version → `source: workspace`
/// - Explicit `source registry` or no matching member → [`WorkspaceError::RegistryUnavailable`]
/// - Member dependency graph must be a DAG
pub fn resolve_workspace_dependencies(ws: &WorkspaceIndex) -> Result<Lockfile, WorkspaceError> {
    use std::collections::{HashMap, HashSet};

    // Workspace-level path-free deps: refuse registry / missing members (no network).
    for dep in &ws.manifest.dependencies {
        if dep.path.is_some() {
            continue;
        }
        let formal = formal_dep_name(dep);
        let source = dep.source.as_deref().unwrap_or("");
        if source == "registry" {
            return Err(WorkspaceError::registry_unavailable(format!(
                "workspace cannot resolve `{formal}` via registry"
            )));
        }
        if source == "workspace" || source.is_empty() {
            if !ws.members.contains_key(formal) {
                return Err(WorkspaceError::registry_unavailable(format!(
                    "no workspace member matches workspace dependency `{formal}`"
                )));
            }
            continue;
        }
        return Err(WorkspaceError::registry_unavailable(format!(
            "unknown workspace dependency source `{source}` for `{formal}`"
        )));
    }

    // formal name → list of formal dependency names resolved via workspace
    let mut edges: HashMap<String, Vec<String>> = HashMap::new();

    for (name, (_root, manifest)) in &ws.members {
        let mut deps = Vec::new();
        for dep in &manifest.dependencies {
            if dep.path.is_some() {
                // Path deps remain path-sourced; skip for workspace-preference graph.
                continue;
            }
            let formal = formal_dep_name(dep).to_string();
            let source = dep.source.as_deref().unwrap_or("");
            if source == "registry" {
                return Err(WorkspaceError::registry_unavailable(format!(
                    "cannot resolve `{formal}` via registry"
                )));
            }
            if source == "workspace" || source.is_empty() {
                let Some((_, member)) = ws.members.get(&formal) else {
                    return Err(WorkspaceError::registry_unavailable(format!(
                        "no workspace member matches `{formal}`"
                    )));
                };
                if !version_satisfies(&dep.version_req, &member.version) {
                    return Err(WorkspaceError::VersionMismatch(format!(
                        "workspace member `{formal}` is version `{}` but `{}` requires `{}`",
                        member.version, name, dep.version_req
                    )));
                }
                deps.push(formal);
                continue;
            }
            return Err(WorkspaceError::registry_unavailable(format!(
                "unknown dependency source `{source}` for `{formal}`"
            )));
        }
        edges.insert(name.clone(), deps);
    }

    // Cycle detection (DFS).
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    fn dfs(
        node: &str,
        edges: &HashMap<String, Vec<String>>,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) -> Result<(), WorkspaceError> {
        if visited.contains(node) {
            return Ok(());
        }
        if !visiting.insert(node.to_string()) {
            let mut cycle = stack.clone();
            cycle.push(node.to_string());
            return Err(WorkspaceError::DependencyCycle(format!(
                "workspace member dependency cycle: {}",
                cycle.join(" -> ")
            )));
        }
        stack.push(node.to_string());
        if let Some(deps) = edges.get(node) {
            for d in deps {
                dfs(d, edges, visiting, visited, stack)?;
            }
        }
        stack.pop();
        visiting.remove(node);
        visited.insert(node.to_string());
        Ok(())
    }
    let mut stack = Vec::new();
    for name in edges.keys() {
        dfs(name, &edges, &mut visiting, &mut visited, &mut stack)?;
    }

    let mut packages: Vec<LockedPackage> = ws
        .members
        .values()
        .map(|(root, m)| LockedPackage {
            name: m.name.clone(),
            version: m.version.clone(),
            source: "workspace".into(),
            dependencies: edges.get(&m.name).cloned().unwrap_or_default(),
            checksum: Some(content_checksum(root.join("package.rpxm"))),
        })
        .collect();
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Lockfile { packages })
}
