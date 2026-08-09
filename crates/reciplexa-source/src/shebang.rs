//! Shebang detection per `specification.md` SYN-001 §1.5.

/// Whether `text` begins with a Unix shebang at byte offset 0 (after optional BOM strip).
pub fn detect_shebang(text: &str) -> Option<&str> {
    let body = text.strip_prefix('\u{feff}').unwrap_or(text);
    body.starts_with("#!")
        .then(|| body.lines().next().expect("shebang body is non-empty"))
}
