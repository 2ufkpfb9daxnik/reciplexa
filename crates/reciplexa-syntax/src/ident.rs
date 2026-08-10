//! SYN §3 identifier rules: NFC, kebab-case, wildcard `_`, trailing `?!`.

use unicode_normalization::UnicodeNormalization;

/// NFC-normalize an identifier before name resolution / BindingId (SYN §3.2).
pub fn normalize_ident(raw: &str) -> String {
    raw.nfc().collect()
}

/// Fixed freestanding operator identifiers (SYN §5).
pub fn is_operator_ident(name: &str) -> bool {
    matches!(
        name,
        "+" | "-" | "*" | "/" | "=" | "<" | ">" | "<=" | ">=" | "!="
    )
}

/// True when `name` is the wildcard binder `_` (SYN §3.5).
pub fn is_wildcard_ident(name: &str) -> bool {
    name == "_"
}

/// Validate a normal (non-operator) identifier spelling per SYN §3.
///
/// Accepts:
/// - bare `_` (wildcard)
/// - `$` + kebab body (MAC-001 pattern variables)
/// - lowercase-start / caseless XID body with kebab `-`, optional trailing `?`/`!`
/// - joined module paths containing `/` (MOD qualified refs), each segment validated
///
/// Rejects uppercase start, `_` inside names, leading/trailing/double `-`, etc.
pub fn validate_ident(name: &str) -> Result<(), String> {
    if is_operator_ident(name) || is_wildcard_ident(name) {
        return Ok(());
    }
    if name == "..." || name == "...+" {
        return Ok(());
    }

    let body = if let Some(rest) = name.strip_prefix('$') {
        if rest.is_empty() {
            return Err("empty macro pattern variable after `$`".into());
        }
        rest
    } else {
        name
    };

    // Joined paths (`graphics/color`, `color/black`) after lexer splits on `/`.
    if body.contains('/') {
        for part in body.split('/') {
            validate_ident_segment(part)?;
        }
        return Ok(());
    }

    validate_ident_segment(body)
}

/// Result of coalescing `segment` `/` `segment` runs in a flat atom list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashAtom<T> {
    /// Joined path with at least one `/` (`graphics/color`).
    Path(String),
    /// Unchanged item (including freestanding `/` division).
    Item(T),
}

/// Join `a` `/` `b` `/` `c` Ident runs so module paths stay one logical name.
///
/// Freestanding `/` (division) and `//` (structured comments) are left alone.
pub fn coalesce_slash_paths<T>(
    items: Vec<T>,
    ident_text: impl Fn(&T) -> Option<&str>,
) -> Vec<SlashAtom<T>> {
    let mut slots: Vec<Option<T>> = items.into_iter().map(Some).collect();
    let mut out = Vec::with_capacity(slots.len());
    let mut i = 0;
    while i < slots.len() {
        let can_path = slots[i]
            .as_ref()
            .and_then(&ident_text)
            .is_some_and(|s| s != "/" && s != "//" && !is_operator_ident(s));
        if can_path {
            let mut j = i + 1;
            let mut path = ident_text(slots[i].as_ref().expect("slot"))
                .expect("checked")
                .to_string();
            while j + 1 < slots.len() {
                let slash = slots[j].as_ref().and_then(&ident_text);
                let seg = slots[j + 1].as_ref().and_then(&ident_text);
                match (slash, seg) {
                    (Some("/"), Some(seg))
                        if seg != "/" && seg != "//" && !is_operator_ident(seg) =>
                    {
                        path.push('/');
                        path.push_str(seg);
                        j += 2;
                    }
                    _ => break,
                }
            }
            if j > i + 1 {
                out.push(SlashAtom::Path(path));
                i = j;
                continue;
            }
        }
        out.push(SlashAtom::Item(
            slots[i].take().expect("slot present for emit"),
        ));
        i += 1;
    }
    out
}

fn validate_ident_segment(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("empty identifier segment".into());
    }
    if name.contains('_') {
        return Err(format!(
            "underscore is not allowed in identifier `{name}` (use kebab-case)"
        ));
    }

    let mut chars = name.chars().peekable();
    let first = *chars.peek().expect("non-empty");
    if first.is_ascii_digit() {
        return Err(format!("identifier `{name}` must not start with a digit"));
    }
    if first == '-' {
        return Err(format!("identifier `{name}` must not start with `-`"));
    }
    if first == '?' || first == '!' {
        return Err(format!("identifier `{name}` must not start with `{first}`"));
    }
    // Case-bearing letters must be lowercase at start (SYN §3.3).
    if first.is_uppercase() {
        return Err(format!(
            "identifier `{name}` must start with a lowercase letter"
        ));
    }
    if !is_ident_start_char(first) {
        return Err(format!("invalid identifier `{name}`"));
    }
    chars.next();

    let mut prev_hyphen = false;
    while let Some(&c) = chars.peek() {
        if c == '-' {
            if prev_hyphen {
                return Err(format!("identifier `{name}` has consecutive `-`"));
            }
            prev_hyphen = true;
            chars.next();
            continue;
        }
        if c == '?' || c == '!' {
            chars.next();
            // Must be last character.
            if chars.peek().is_some() {
                return Err(format!(
                    "`?`/`!` may only appear once at the end of `{name}`"
                ));
            }
            if prev_hyphen {
                return Err(format!("identifier `{name}` must not end with `-`"));
            }
            return Ok(());
        }
        if !is_ident_continue_char(c) {
            return Err(format!("invalid character in identifier `{name}`"));
        }
        prev_hyphen = false;
        chars.next();
    }
    if prev_hyphen {
        return Err(format!("identifier `{name}` must not end with `-`"));
    }
    Ok(())
}

fn is_ident_start_char(c: char) -> bool {
    // Unicode XID_Start-ish: alphabetic; `$` handled by caller strip.
    c.is_alphabetic() || c == '$'
}

fn is_ident_continue_char(c: char) -> bool {
    c.is_alphabetic() || c.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_kebab_and_suffix() {
        assert!(validate_ident("read-file").is_ok());
        assert!(validate_ident("empty?").is_ok());
        assert!(validate_ident("commit!").is_ok());
        assert!(validate_ident("_").is_ok());
        assert!(validate_ident("$body").is_ok());
        assert!(validate_ident("半径").is_ok());
        assert!(validate_ident("graphics/color").is_ok());
    }

    #[test]
    fn rejects_underscore_and_upper() {
        assert!(validate_ident("report_title").is_err());
        assert!(validate_ident("Report").is_err());
        assert!(validate_ident("-page").is_err());
        assert!(validate_ident("page-").is_err());
        assert!(validate_ident("a--b").is_err());
        assert!(validate_ident("value??").is_err());
    }

    #[test]
    fn nfc_normalize() {
        // é as e + combining acute → NFC é
        let decomposed = "e\u{0301}";
        let nfc = normalize_ident(decomposed);
        assert_eq!(nfc.chars().count(), 1);
    }

    #[test]
    fn coalesces_path_segments() {
        let items = ["graphics", "/", "color", "as", "color"];
        let out = coalesce_slash_paths(items.to_vec(), |s| Some(*s));
        assert_eq!(
            out,
            vec![
                SlashAtom::Path("graphics/color".into()),
                SlashAtom::Item("as"),
                SlashAtom::Item("color"),
            ]
        );
        let div = coalesce_slash_paths(vec!["/", "x", "y"], |s| Some(*s));
        assert!(matches!(div[0], SlashAtom::Item("/")));
    }
}
