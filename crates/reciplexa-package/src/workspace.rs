//! `workspace.rpxm` parser and member discovery (PKG-001 DD-001 §20).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::manifest::PackageManifest;
use crate::rpxm::{parse_rpxm, tokenize, RpxmError};

/// Parsed workspace manifest (`workspace.rpxm`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceManifest {
    pub format_version: u32,
    /// Member paths relative to the workspace root (no globs in v1).
    pub members: Vec<String>,
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
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(s) | Self::NestedWorkspace(s) | Self::DuplicateName(s) | Self::MissingMember(s) => {
                write!(f, "{s}")
            }
            Self::Manifest(e) => write!(f, "workspace manifest error: {e:?}"),
        }
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
    })
}

/// Discover a workspace at `root`: parse `workspace.rpxm`, load each member's
/// `package.rpxm`, require unique formal names, and reject nested `workspace.rpxm`
/// inside member directories (PKG §20.4–20.8).
pub fn discover_workspace(root: impl AsRef<Path>) -> Result<WorkspaceIndex, WorkspaceError> {
    let root = root.as_ref();
    let ws_path = root.join("workspace.rpxm");
    let src = fs::read_to_string(&ws_path).map_err(|e| {
        WorkspaceError::Io(format!("read `{}`: {e}", ws_path.display()))
    })?;
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
        let pkg_src = fs::read_to_string(&pkg_path).map_err(|e| {
            WorkspaceError::Io(format!("read `{}`: {e}", pkg_path.display()))
        })?;
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
