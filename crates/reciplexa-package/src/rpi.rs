//! Minimal `.rpi` interface stub parser (MOD §8 / Slice B).
//!
//! Full signature checking is deferred; this extracts the public export name set
//! from `(val name)`, `(type name)`, and named `(fn name …)` forms so package
//! linking can enforce the interface export boundary.

use crate::PackageLoadError;

/// Parse a `.rpi` interface stub and return the ordered public export names.
pub fn parse_rpi_exports(src: &str) -> Result<Vec<String>, PackageLoadError> {
    let tokens = tokenize(src).map_err(PackageLoadError::Interface)?;
    let mut exports = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut i = 0usize;
    while i < tokens.len() {
        if tokens[i] != "(" {
            i += 1;
            continue;
        }
        if i + 1 >= tokens.len() {
            break;
        }
        match tokens[i + 1].as_str() {
            "val" | "type" => {
                if i + 2 >= tokens.len() || tokens[i + 2] == "(" || tokens[i + 2] == ")" {
                    return Err(PackageLoadError::Interface(format!(
                        "`{}` form in .rpi requires a name",
                        tokens[i + 1]
                    )));
                }
                let name = tokens[i + 2].clone();
                if seen.insert(name.clone()) {
                    exports.push(name);
                }
                i = skip_list(&tokens, i)?;
            }
            "fn" => {
                // Named: `(fn name (params…) …)` — skip anonymous `(fn (params…) …)`.
                if i + 2 < tokens.len() && tokens[i + 2] != "(" && tokens[i + 2] != ")" {
                    let name = tokens[i + 2].clone();
                    if seen.insert(name.clone()) {
                        exports.push(name);
                    }
                }
                i = skip_list(&tokens, i)?;
            }
            "//" => {
                // Structured comment `(// …)` — skip.
                i = skip_list(&tokens, i)?;
            }
            _ => {
                i = skip_list(&tokens, i)?;
            }
        }
    }
    Ok(exports)
}

fn skip_list(tokens: &[String], start: usize) -> Result<usize, PackageLoadError> {
    let mut depth = 0i32;
    let mut i = start;
    while i < tokens.len() {
        match tokens[i].as_str() {
            "(" => depth += 1,
            ")" => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(PackageLoadError::Interface("unclosed list in .rpi".into()))
}

fn tokenize(src: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '(' || c == ')' {
            tokens.push(c.to_string());
            chars.next();
            continue;
        }
        if c == '"' {
            chars.next();
            let mut s = String::from("\"");
            let mut closed = false;
            for ch in chars.by_ref() {
                s.push(ch);
                if ch == '"' {
                    closed = true;
                    break;
                }
            }
            if !closed {
                return Err("unclosed string in .rpi".into());
            }
            tokens.push(s);
            continue;
        }
        let mut atom = String::new();
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() || ch == '(' || ch == ')' {
                break;
            }
            atom.push(ch);
            chars.next();
        }
        // First peeked char was a non-delimiter (otherwise handled above), so atom is non-empty.
        tokens.push(atom);
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_val_exports() {
        let exports = parse_rpi_exports(
            r#"(// stub)
(val circle)
(val rect)
(type point)
"#,
        )
        .unwrap();
        assert_eq!(exports, vec!["circle", "rect", "point"]);
    }
}
