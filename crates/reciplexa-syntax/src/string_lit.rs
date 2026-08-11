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

/// SYN §8.4 special character values (no escapes — use these identifiers / `unicode`).
pub fn special_char_value(name: &str) -> Option<&'static str> {
    match name {
        "newline" => Some("\n"),
        "tab" => Some("\t"),
        "carriage-return" => Some("\r"),
        "nul" => Some("\0"),
        _ => None,
    }
}

/// Build a one-character `str` from a Unicode scalar value (SYN §8.4 `(unicode 0x…)`).
pub fn unicode_scalar_value(code: f64) -> Result<String, String> {
    if !code.is_finite() || code < 0.0 || code.fract() != 0.0 {
        return Err("unicode code point must be a non-negative integer".into());
    }
    let cp = code as u32;
    char::from_u32(cp)
        .map(|c| c.to_string())
        .ok_or_else(|| format!("invalid Unicode scalar value U+{cp:X}"))
}

/// When a string token's source text lacks a closing delimiter, return the virtual closer.
pub fn virtual_close_delimiter(raw: &str) -> Option<String> {
    if raw.is_empty() || !raw.starts_with('"') {
        return None;
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
        return None;
    }
    if open == 2 {
        return if raw.len() == 2 {
            None
        } else {
            Some("\"".to_string())
        };
    }
    if raw.len() < open * 2 {
        return Some("\"".repeat(open));
    }
    if !raw[raw.len() - open..].chars().all(|c| c == '"') {
        return Some("\"".repeat(open));
    }
    None
}

/// Encode a runtime string as an RPX string literal (SYN-001 §8).
///
/// Never invents backslash escapes. Uses short `"…"` when safe; otherwise a
/// multi-quote delimiter long enough that the content cannot close it early.
pub fn encode_string_literal(s: &str) -> String {
    if s.is_empty() {
        return r#""""#.to_string();
    }
    let needs_multi = s.contains('"') || s.contains('\n') || s.contains('\r');
    if !needs_multi {
        return format!("\"{s}\"");
    }
    let mut n = 3usize;
    loop {
        let delim: String = std::iter::repeat_n('"', n).collect();
        if !s.contains(delim.as_str()) {
            // Prefer multiline so trailing `"` in content cannot glue to the closer.
            return format!("{delim}\n{s}\n{delim}");
        }
        n += 1;
    }
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
        assert_eq!(decode_string_literal(raw).unwrap(), "She said \"hello\".");
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
    fn encode_roundtrips_quotes_and_newlines() {
        let s = "say \"hi\"\nnext";
        let lit = encode_string_literal(s);
        assert!(!lit.contains('\\'));
        assert_eq!(decode_string_literal(&lit).unwrap(), s);
    }

    #[test]
    fn encode_roundtrips_unicode_content() {
        let s = "日本語\u{3002}emoji🎉";
        let lit = encode_string_literal(s);
        assert!(!lit.contains('\\'));
        assert_eq!(decode_string_literal(&lit).unwrap(), s);
    }

    #[test]
    fn special_char_helpers() {
        assert_eq!(special_char_value("newline"), Some("\n"));
        assert_eq!(unicode_scalar_value(12290.0).unwrap(), "。");
        assert!(unicode_scalar_value(-1.0).is_err());
        assert!(unicode_scalar_value(1114112.0).is_err());
    }

    #[test]
    fn virtual_close_for_unterminated_short_string() {
        assert_eq!(virtual_close_delimiter("\"abc"), Some("\"".into()));
        assert_eq!(virtual_close_delimiter("\"hi\""), None);
    }
}
