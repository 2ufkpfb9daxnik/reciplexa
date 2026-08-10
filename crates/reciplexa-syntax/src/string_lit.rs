//! Decode RPX string literal token text (SYN-001 §8).
//!
//! Token text includes delimiters. Short `"…"` and multi-quote `"""…"""` (n≥3)
//! share this decoder; there are no backslash escapes.

/// Decode a [`crate::SyntaxKind::String`] token's raw text into a runtime string.
///
/// Returns an error message when the token is malformed or a multi-quote body
/// has a non-empty line shallower than the closing delimiter's indentation.
pub fn decode_string_literal(raw: &str) -> Result<String, String> {
    if raw.len() < 2 || !raw.starts_with('"') || !raw.ends_with('"') {
        return Err("malformed string literal".into());
    }

    let mut open = 0usize;
    for c in raw.chars() {
        if c == '"' {
            open += 1;
        } else {
            break;
        }
    }
    if open == 0 {
        return Err("malformed string literal delimiters".into());
    }

    // `""` is the empty string (delimiter length 2 is reserved for this).
    if open == 2 {
        return if raw.len() == 2 {
            Ok(String::new())
        } else {
            Err("malformed empty string literal".into())
        };
    }

    if raw.len() < open * 2 {
        return Err("malformed string literal delimiters".into());
    }
    if !raw[raw.len() - open..].chars().all(|c| c == '"') {
        return Err("malformed string literal delimiters".into());
    }

    let inner = &raw[open..raw.len() - open];
    if open == 1 {
        return Ok(inner.to_string());
    }

    // Multi-quote (§8.3).
    let mut body = normalize_newlines(inner);
    let baseline = strip_closing_line(&mut body);
    if body.starts_with('\n') {
        body.remove(0);
    }
    dedent_lines(&body, &baseline)
}

fn normalize_newlines(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push('\n');
            }
            other => out.push(other),
        }
    }
    out
}

/// Drop the closing delimiter line (newline + indent before `"""`), returning that indent.
fn strip_closing_line(body: &mut String) -> String {
    match body.rfind('\n') {
        Some(idx) => {
            let after = body[idx + 1..].to_string();
            if after.chars().all(|c| c == ' ' || c == '\t') {
                body.truncate(idx);
                after
            } else {
                // Closer shares a line with content; no indent baseline.
                String::new()
            }
        }
        None => String::new(),
    }
}

fn dedent_lines(body: &str, baseline: &str) -> Result<String, String> {
    if baseline.is_empty() {
        return Ok(body.to_string());
    }
    let mut out = String::with_capacity(body.len());
    for (i, line) in body.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if line.is_empty() {
            continue;
        }
        if !line.starts_with(baseline) {
            return Err(
                "multi-quote string line is less indented than the closing delimiter".into(),
            );
        }
        out.push_str(&line[baseline.len()..]);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_string() {
        assert_eq!(decode_string_literal(r#""hi""#).unwrap(), "hi");
    }

    #[test]
    fn empty_string() {
        assert_eq!(decode_string_literal(r#""""#).unwrap(), "");
    }

    #[test]
    fn multi_with_inner_quotes() {
        let raw = "\"\"\"She said \"hello\".\"\"\"";
        assert_eq!(
            decode_string_literal(raw).unwrap(),
            "She said \"hello\"."
        );
    }

    #[test]
    fn multi_multiline_strips_edge_newlines() {
        let raw = "\"\"\"\nhello\nworld\n\"\"\"";
        assert_eq!(decode_string_literal(raw).unwrap(), "hello\nworld");
    }

    #[test]
    fn multi_dedents_to_closing_indent() {
        let raw = "\"\"\"\n  a\n  b\n  \"\"\"";
        assert_eq!(decode_string_literal(raw).unwrap(), "a\nb");
    }

    #[test]
    fn multi_rejects_shallow_line() {
        let raw = "\"\"\"\na\n  \"\"\"";
        assert!(decode_string_literal(raw).unwrap_err().contains("indented"));
    }
}
