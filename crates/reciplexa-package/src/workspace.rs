//! `workspace.rpxm` stub parser (PKG-001 DD-001 §20).

use crate::rpxm::{tokenize, RpxmError};

/// Parsed workspace manifest (`workspace.rpxm`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceManifest {
    pub format_version: u32,
    /// Member paths relative to the workspace root (no globs in v1).
    pub members: Vec<String>,
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
