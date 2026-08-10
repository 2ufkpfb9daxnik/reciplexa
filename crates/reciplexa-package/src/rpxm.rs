//! `package.rpxm` text format parser (Phase 10 + DD-001 flat fields).

use crate::manifest::{DependencySpec, PackageManifest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpxmError {
    Empty,
    MissingName,
    MissingVersion,
    MissingEntry,
    UnsupportedFormatVersion(u32),
    Syntax(String),
}

/// Parse a `package.rpxm` document.
///
/// Accepts the Phase 10 nested form:
///
/// ```text
/// (package
///   (name demo)
///   (version 0.1.0)
///   (entry main.rpx)
///   (dep util 1.0 (path ../util))
///   (target lib))
/// ```
///
/// and the DD-001 flat form (package name as first atom after `package`):
///
/// ```text
/// (package graphics
///   format-version 1
///   version "0.1.0"
///   source-root "src"
///   (public-modules shapes color)
///   (entry-points cli))
/// ```
///
/// Library packages may omit `entry` / `entry-points`.
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
    let mut format_version = 1u32;
    let mut source_root = "src".to_string();
    let mut interface_root = None;
    let mut resource_root = "resources".to_string();
    let mut public_modules = Vec::new();
    let mut entry_points = Vec::new();

    // DD-001: `(package <name> …)` — name immediately after `package`.
    let mut dd001_positional_name = false;
    if tokens.len() >= 3 && tokens[0] == "(" && tokens[1] == "package" {
        let maybe_name = &tokens[2];
        if maybe_name != "(" && maybe_name != ")" {
            name = Some(maybe_name.clone());
            dd001_positional_name = true;
        }
    }

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
                        package: None,
                    });
                    i = j;
                    continue;
                }
                "public-modules" => {
                    let (items, next) = parse_ident_list(&tokens, i)?;
                    public_modules = items;
                    i = next;
                    continue;
                }
                "entry-points" => {
                    let (items, next) = parse_ident_list(&tokens, i)?;
                    entry_points = items;
                    i = next;
                    continue;
                }
                "dependencies" => {
                    let (deps, next) = parse_dependencies_block(&tokens, i)?;
                    dependencies.extend(deps);
                    i = next;
                    continue;
                }
                _ => {}
            }
        }

        // Flat DD-001 fields: `format-version 1`, `version "0.1.0"`, `source-root "src"`.
        if i + 1 < tokens.len() {
            match tokens[i].as_str() {
                "format-version" => {
                    let v = tokens[i + 1].parse::<u32>().map_err(|_| {
                        RpxmError::Syntax(format!("invalid format-version `{}`", tokens[i + 1]))
                    })?;
                    if v != 1 {
                        return Err(RpxmError::UnsupportedFormatVersion(v));
                    }
                    format_version = v;
                    i += 2;
                    continue;
                }
                "version" => {
                    version = Some(tokens[i + 1].clone());
                    i += 2;
                    continue;
                }
                "source-root" => {
                    source_root = tokens[i + 1].clone();
                    i += 2;
                    continue;
                }
                "interface-root" => {
                    interface_root = Some(tokens[i + 1].clone());
                    i += 2;
                    continue;
                }
                "resource-root" => {
                    resource_root = tokens[i + 1].clone();
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        i += 1;
    }

    let name = name.ok_or(RpxmError::MissingName)?;
    let version = version.ok_or(RpxmError::MissingVersion)?;

    // Legacy Phase 10 documents required `(entry …)`. DD-001 library packages may omit it.
    let entry = match entry {
        Some(e) => e,
        None if !entry_points.is_empty() => format!("{}.rpx", entry_points[0]),
        None if dd001_positional_name || !public_modules.is_empty() => String::new(),
        None => return Err(RpxmError::MissingEntry),
    };

    Ok(PackageManifest {
        name,
        version,
        dependencies,
        entry,
        targets,
        format_version,
        source_root,
        interface_root,
        resource_root,
        public_modules,
        entry_points,
    })
}

/// Parse `(keyword a b c)` starting at `(`; returns items and index after closing `)`.
fn parse_ident_list(tokens: &[String], start: usize) -> Result<(Vec<String>, usize), RpxmError> {
    if tokens.get(start).map(String::as_str) != Some("(") {
        return Err(RpxmError::Syntax("expected `(` for list field".into()));
    }
    let mut i = start + 2; // skip ( keyword
    let mut items = Vec::new();
    while i < tokens.len() && tokens[i] != ")" {
        if tokens[i] == "(" {
            return Err(RpxmError::Syntax(
                "nested lists are not allowed in module path lists".into(),
            ));
        }
        items.push(tokens[i].clone());
        i += 1;
    }
    if i >= tokens.len() {
        return Err(RpxmError::Syntax("unclosed list field".into()));
    }
    Ok((items, i + 1))
}

/// Parse DD-001 `(dependencies (alias package name version "…" path "…") …)`.
fn parse_dependencies_block(
    tokens: &[String],
    start: usize,
) -> Result<(Vec<DependencySpec>, usize), RpxmError> {
    if tokens.get(start).map(String::as_str) != Some("(")
        || tokens.get(start + 1).map(String::as_str) != Some("dependencies")
    {
        return Err(RpxmError::Syntax("expected `(dependencies …)`".into()));
    }
    let mut i = start + 2;
    let mut deps = Vec::new();
    while i < tokens.len() && tokens[i] != ")" {
        if tokens[i] != "(" {
            return Err(RpxmError::Syntax(
                "dependency entries must be parenthesized".into(),
            ));
        }
        let (dep, next) = parse_one_dependency(tokens, i)?;
        deps.push(dep);
        i = next;
    }
    if i >= tokens.len() {
        return Err(RpxmError::Syntax("unclosed dependencies block".into()));
    }
    Ok((deps, i + 1))
}

fn parse_one_dependency(
    tokens: &[String],
    start: usize,
) -> Result<(DependencySpec, usize), RpxmError> {
    // (alias package formal-name version "…" [path "…"])
    if tokens.get(start).map(String::as_str) != Some("(") {
        return Err(RpxmError::Syntax("expected `(` for dependency".into()));
    }
    let alias = tokens
        .get(start + 1)
        .ok_or_else(|| RpxmError::Syntax("dependency missing alias".into()))?
        .clone();
    let mut package = None;
    let mut version_req = None;
    let mut path = None;
    let mut i = start + 2;
    while i < tokens.len() && tokens[i] != ")" {
        match tokens[i].as_str() {
            "package" if i + 1 < tokens.len() => {
                package = Some(tokens[i + 1].clone());
                i += 2;
            }
            "version" if i + 1 < tokens.len() => {
                version_req = Some(tokens[i + 1].clone());
                i += 2;
            }
            "path" if i + 1 < tokens.len() => {
                path = Some(tokens[i + 1].clone());
                i += 2;
            }
            "(" => {
                // Nested `(path …)` style leftover — rare; skip as unknown group.
                let mut depth = 1usize;
                i += 1;
                while i < tokens.len() && depth > 0 {
                    match tokens[i].as_str() {
                        "(" => depth += 1,
                        ")" => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    if i >= tokens.len() {
        return Err(RpxmError::Syntax("unclosed dependency entry".into()));
    }
    let version_req = version_req
        .ok_or_else(|| RpxmError::Syntax(format!("dependency `{alias}` missing version")))?;
    Ok((
        DependencySpec {
            name: alias,
            version_req,
            path,
            package,
        },
        i + 1,
    ))
}

fn tokenize(src: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            '(' => {
                // SYN-001-style structured comment: `(// …)` with nested parens.
                let mut lookahead = chars.clone();
                lookahead.next(); // (
                while let Some(&w) = lookahead.peek() {
                    if w.is_whitespace() {
                        lookahead.next();
                    } else {
                        break;
                    }
                }
                let is_comment = lookahead.next() == Some('/') && lookahead.next() == Some('/');
                if is_comment {
                    chars.next(); // (
                    let mut depth = 1usize;
                    while depth > 0 {
                        let Some(ch) = chars.next() else {
                            return Err("unclosed structured comment".into());
                        };
                        match ch {
                            '(' => depth += 1,
                            ')' => depth -= 1,
                            '"' => {
                                for ch in chars.by_ref() {
                                    if ch == '"' {
                                        break;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    continue;
                }
                out.push("(".to_string());
                chars.next();
            }
            ')' => {
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
                    if ch.is_whitespace() || ch == '(' || ch == ')' {
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
