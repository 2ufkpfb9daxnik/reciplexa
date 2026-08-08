//! `package.rpxm` text format parser (Phase 10).

use crate::manifest::{DependencySpec, PackageManifest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpxmError {
    Empty,
    MissingName,
    MissingVersion,
    MissingEntry,
    Syntax(String),
}

/// Parse a minimal `package.rpxm` document.
///
/// ```text
/// (package
///   (name demo)
///   (version 0.1.0)
///   (entry main.rpx)
///   (dep util 1.0 (path ../util))
///   (target lib)
///   (target gui))
/// ```
pub fn parse_rpxm(src: &str) -> Result<PackageManifest, RpxmError> {
    let tokens = tokenize(src).map_err(RpxmError::Syntax)?;
    if tokens.is_empty() {
        return Err(RpxmError::Empty);
    }
    let mut name = None;
    let mut version = None;
    let mut entry = None;
    let mut dependencies = Vec::new();
    let mut targets = Vec::new();

    let mut i = 0usize;
    while i < tokens.len() {
        if tokens[i] == "(" && i + 1 < tokens.len() {
            match tokens[i + 1].as_str() {
                "name" if i + 2 < tokens.len() => {
                    name = Some(tokens[i + 2].clone());
                    i += 4; // ( name value )
                    continue;
                }
                "version" if i + 2 < tokens.len() => {
                    version = Some(tokens[i + 2].clone());
                    i += 4;
                    continue;
                }
                "entry" if i + 2 < tokens.len() => {
                    entry = Some(tokens[i + 2].clone());
                    i += 4;
                    continue;
                }
                "target" if i + 2 < tokens.len() => {
                    targets.push(tokens[i + 2].clone());
                    i += 4;
                    continue;
                }
                "dep" if i + 3 < tokens.len() => {
                    let dep_name = tokens[i + 2].clone();
                    let ver = tokens[i + 3].clone();
                    let mut path = None;
                    let mut j = i + 4;
                    if j + 3 < tokens.len() && tokens[j] == "(" && tokens[j + 1] == "path" {
                        path = Some(tokens[j + 2].clone());
                        j += 4;
                    }
                    dependencies.push(DependencySpec {
                        name: dep_name,
                        version_req: ver,
                        path,
                    });
                    // Trailing `)` of `(dep ...)` is skipped by the outer scan.
                    i = j;
                    continue;
                }
                _ => {}
            }
        }
        i += 1;
    }

    Ok(PackageManifest {
        name: name.ok_or(RpxmError::MissingName)?,
        version: version.ok_or(RpxmError::MissingVersion)?,
        dependencies,
        entry: entry.ok_or(RpxmError::MissingEntry)?,
        targets,
    })
}

fn tokenize(src: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ';' => {
                while let Some(&c2) = chars.peek() {
                    chars.next();
                    if c2 == '\n' {
                        break;
                    }
                }
            }
            '(' | ')' => {
                out.push(c.to_string());
                chars.next();
            }
            ch if ch.is_whitespace() => {
                chars.next();
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                let mut closed = false;
                for ch in chars.by_ref() {
                    if ch == '"' {
                        closed = true;
                        break;
                    }
                    s.push(ch);
                }
                if !closed {
                    return Err("unclosed string literal".into());
                }
                out.push(s);
            }
            _ => {
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() || ch == '(' || ch == ')' || ch == ';' {
                        break;
                    }
                    s.push(ch);
                    chars.next();
                }
                // `_` only matches a non-delimiter char, so `s` is never empty.
                out.push(s);
            }
        }
    }
    Ok(out)
}
