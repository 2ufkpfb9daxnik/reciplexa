//! SYN §7 numeric literal parsing (shared by lexer elaborate / lower).
//!
//! Accepts decimal, `0b`/`0o`/`0x` radix, `_` digit separators, and scientific
//! `e` notation. Rejects unnecessary leading zeros on decimal integers.

/// True when the token text denotes an `f64` literal (fractional or scientific).
pub fn is_f64_literal_form(text: &str) -> bool {
    let (_, body) = strip_sign(text);
    body.contains('.') || body.contains('e') || body.contains('E')
}

/// Parse a SYN §7 integer literal into `i128` (DD-TYP-NUM-001 kernel subset).
pub fn parse_int_literal(text: &str) -> Result<i128, String> {
    let (signed, body) = strip_sign(text);
    if body.is_empty() {
        return Err(format!("invalid number literal `{text}`"));
    }
    if is_f64_literal_form(text) {
        return Err(format!("expected integer literal, got f64 form `{text}`"));
    }

    let value = if let Some(rest) = body.strip_prefix("0x") {
        parse_radix_i128(rest, 16, text)?
    } else if let Some(rest) = body.strip_prefix("0b") {
        parse_radix_i128(rest, 2, text)?
    } else if let Some(rest) = body.strip_prefix("0o") {
        parse_radix_i128(rest, 8, text)?
    } else {
        reject_leading_zeros(body, text)?;
        let cleaned = strip_digit_separators(body, |c| c.is_ascii_digit(), text)?;
        if cleaned.is_empty() {
            return Err(format!("invalid number literal `{text}`"));
        }
        cleaned
            .parse::<i128>()
            .map_err(|_| format!("integer literal out of range `{text}`"))?
    };

    Ok(if signed { -value } else { value })
}

fn parse_radix_i128(digits: &str, radix: u32, raw: &str) -> Result<i128, String> {
    let cleaned = strip_digit_separators(digits, |c| char::is_digit(c, radix), raw)?;
    if cleaned.is_empty() {
        return Err(format!("invalid number literal `{raw}`"));
    }
    i128::from_str_radix(&cleaned, radix)
        .map_err(|_| format!("integer literal out of range `{raw}`"))
}

/// Parse a SYN §7 number token text into `f64`.
///
/// Underscores are ignored. Radix prefixes use lowercase `0b`/`0o`/`0x` only.
pub fn parse_number_literal(text: &str) -> Result<f64, String> {
    let (signed, body) = strip_sign(text);
    if body.is_empty() {
        return Err(format!("invalid number literal `{text}`"));
    }

    let value = if let Some(rest) = body.strip_prefix("0x") {
        parse_radix_int(rest, 16, text)?
    } else if let Some(rest) = body.strip_prefix("0b") {
        parse_radix_int(rest, 2, text)?
    } else if let Some(rest) = body.strip_prefix("0o") {
        parse_radix_int(rest, 8, text)?
    } else {
        parse_decimal(body, text)?
    };

    let value = if signed { -value } else { value };
    if !value.is_finite() {
        return Err(format!("non-finite number literal `{text}`"));
    }
    Ok(value)
}

fn strip_sign(text: &str) -> (bool, &str) {
    if let Some(rest) = text.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = text.strip_prefix('+') {
        (false, rest)
    } else {
        (false, text)
    }
}

fn parse_radix_int(digits: &str, radix: u32, raw: &str) -> Result<f64, String> {
    let cleaned = strip_digit_separators(digits, |c| char::is_digit(c, radix), raw)?;
    if cleaned.is_empty() {
        return Err(format!("invalid number literal `{raw}`"));
    }
    // Arbitrary-precision via u128 when possible; fall back to f64 for huge ints.
    if let Ok(n) = u128::from_str_radix(&cleaned, radix) {
        Ok(n as f64)
    } else {
        // Very large: approximate via iterative multiply (still f64).
        let mut acc = 0.0f64;
        for c in cleaned.chars() {
            let d = c.to_digit(radix).expect("validated");
            acc = acc * (radix as f64) + (d as f64);
        }
        if !acc.is_finite() {
            return Err(format!("non-finite number literal `{raw}`"));
        }
        Ok(acc)
    }
}

fn parse_decimal(body: &str, raw: &str) -> Result<f64, String> {
    // Reject unnecessary leading zeros on the integer part (SYN §7.2).
    reject_leading_zeros(body, raw)?;

    let cleaned = strip_decimal_separators(body, raw)?;
    cleaned
        .parse::<f64>()
        .map_err(|_| format!("invalid number literal `{raw}`"))
}

fn reject_leading_zeros(body: &str, raw: &str) -> Result<(), String> {
    // Integer part before `.` or `e`.
    let int_part = body.split_once(['.', 'e']).map(|(a, _)| a).unwrap_or(body);
    let digits: String = int_part.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() > 1 && digits.starts_with('0') {
        return Err(format!(
            "unnecessary leading zero in number literal `{raw}`"
        ));
    }
    Ok(())
}

fn strip_digit_separators(
    s: &str,
    is_digit: impl Fn(char) -> bool,
    raw: &str,
) -> Result<String, String> {
    if s.is_empty() {
        return Ok(String::new());
    }
    if s.starts_with('_') || s.ends_with('_') {
        return Err(format!("invalid digit separator in `{raw}`"));
    }
    let mut out = String::with_capacity(s.len());
    let mut prev_underscore = false;
    for c in s.chars() {
        if c == '_' {
            if prev_underscore {
                return Err(format!("invalid digit separator in `{raw}`"));
            }
            prev_underscore = true;
            continue;
        }
        if !is_digit(c) {
            return Err(format!("invalid number literal `{raw}`"));
        }
        prev_underscore = false;
        out.push(c);
    }
    Ok(out)
}

fn strip_decimal_separators(body: &str, raw: &str) -> Result<String, String> {
    // Split mantissa / exponent; validate `_` only between digits in each part.
    let (mant, exp) = match body.split_once('e') {
        Some((m, e)) => (m, Some(e)),
        None => (body, None),
    };

    let mut out = String::with_capacity(body.len());
    let (int_part, frac) = match mant.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (mant, None),
    };

    out.push_str(&strip_digit_separators(
        int_part,
        |c| c.is_ascii_digit(),
        raw,
    )?);
    if let Some(f) = frac {
        if f.is_empty() {
            return Err(format!("invalid number literal `{raw}`"));
        }
        out.push('.');
        out.push_str(&strip_digit_separators(f, |c| c.is_ascii_digit(), raw)?);
    }
    if let Some(e) = exp {
        out.push('e');
        let (neg, edigits) = if let Some(rest) = e.strip_prefix('+') {
            (false, rest)
        } else if let Some(rest) = e.strip_prefix('-') {
            (true, rest)
        } else {
            (false, e)
        };
        if neg {
            out.push('-');
        }
        let cleaned = strip_digit_separators(edigits, |c| c.is_ascii_digit(), raw)?;
        if cleaned.is_empty() {
            return Err(format!("invalid number literal `{raw}`"));
        }
        out.push_str(&cleaned);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_and_scientific() {
        assert_eq!(parse_number_literal("42").unwrap(), 42.0);
        assert_eq!(parse_number_literal("1_000_000").unwrap(), 1_000_000.0);
        assert_eq!(parse_number_literal("1e10").unwrap(), 1e10);
        assert_eq!(parse_number_literal("1.5e3").unwrap(), 1500.0);
        assert_eq!(parse_number_literal("-0xff").unwrap(), -255.0);
    }

    #[test]
    fn radix() {
        assert_eq!(parse_number_literal("0x2a").unwrap(), 42.0);
        assert_eq!(parse_number_literal("0b101010").unwrap(), 42.0);
        assert_eq!(parse_number_literal("0o52").unwrap(), 42.0);
        assert_eq!(parse_number_literal("0xff_ff").unwrap(), 0xffff as f64);
    }

    #[test]
    fn rejects_leading_zeros() {
        assert!(parse_number_literal("007").is_err());
        assert!(parse_number_literal("00").is_err());
        assert!(parse_number_literal("0123").is_err());
    }
}
