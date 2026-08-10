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
/// - interim module paths containing `/` (MOD qualified refs)
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

    // Interim: allow `/`-separated path idents for MOD qualified refs.
    if body.contains('/') {
        for part in body.split('/') {
            validate_ident_segment(part)?;
        }
        return Ok(());
    }

    validate_ident_segment(body)
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
        return Err(format!(
            "identifier `{name}` must not start with `{first}`"
        ));
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
}
