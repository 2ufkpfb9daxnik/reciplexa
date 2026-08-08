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
                    if j + 3 < tokens.len()
                        && tokens[j] == "("
                        && tokens[j + 1] == "path"
                    {
                        path = Some(tokens[j + 2].clone());
                        j += 4;
                    }
                    dependencies.push(DependencySpec {
                        name: dep_name,
                        version_req: ver,
                        path,
                    });
                    // consume closing ) of dep
                    if j < tokens.len() && tokens[j] == ")" {
                        j += 1;
                    }
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
                if !s.is_empty() {
                    out.push(s);
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_rpxm() {
        let m = parse_rpxm(
            r#"(package
  (name demo)
  (version 0.1.0)
  (entry main.rpx)
  (dep util 1.0 (path ../util))
  (target lib))"#,
        )
        .unwrap();
        assert_eq!(m.name, "demo");
        assert_eq!(m.dependencies[0].path.as_deref(), Some("../util"));
        assert_eq!(m.targets, vec!["lib".to_string()]);
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(parse_rpxm(""), Err(RpxmError::Empty));
        assert_eq!(parse_rpxm("   ; only comment\n"), Err(RpxmError::Empty));
    }

    #[test]
    fn rejects_missing_name() {
        let err = parse_rpxm("(package (version 1) (entry main.rpx))").unwrap_err();
        assert_eq!(err, RpxmError::MissingName);
    }

    #[test]
    fn rejects_missing_version() {
        let err = parse_rpxm("(package (name demo) (entry main.rpx))").unwrap_err();
        assert_eq!(err, RpxmError::MissingVersion);
    }

    #[test]
    fn rejects_missing_entry() {
        let err = parse_rpxm("(package (name demo) (version 1.0))").unwrap_err();
        assert_eq!(err, RpxmError::MissingEntry);
    }

    #[test]
    fn ignores_semicolon_comments() {
        let m = parse_rpxm(
            r#"; header comment
(package
  ; inline
  (name demo)
  (version 0.1.0)
  (entry main.rpx))"#,
        )
        .unwrap();
        assert_eq!(m.name, "demo");
        assert_eq!(m.version, "0.1.0");
    }

    #[test]
    fn parses_quoted_strings() {
        let m = parse_rpxm(
            r#"(package
  (name "my package")
  (version "0.2.0")
  (entry "src/main.rpx"))"#,
        )
        .unwrap();
        assert_eq!(m.name, "my package");
        assert_eq!(m.entry, "src/main.rpx");
    }

    #[test]
    fn parses_dep_without_path() {
        let m = parse_rpxm(
            r#"(package
  (name app)
  (version 1)
  (entry main.rpx)
  (dep util 2.0))"#,
        )
        .unwrap();
        assert_eq!(m.dependencies.len(), 1);
        assert_eq!(m.dependencies[0].name, "util");
        assert_eq!(m.dependencies[0].version_req, "2.0");
        assert_eq!(m.dependencies[0].path, None);
    }

    #[test]
    fn parses_multiple_targets() {
        let m = parse_rpxm(
            r#"(package
  (name multi)
  (version 1)
  (entry main.rpx)
  (target document)
  (target slide)
  (target preview))"#,
        )
        .unwrap();
        assert_eq!(
            m.targets,
            vec!["document".to_string(), "slide".to_string(), "preview".to_string()]
        );
    }

    #[test]
    fn rejects_unclosed_string_syntax() {
        assert!(matches!(parse_rpxm(r#"(name "broken)"#), Err(RpxmError::Syntax(_))));
    }
}
