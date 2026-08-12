//! Residual coverage for number_lit / string_lit / ident public APIs.

use reciplexa_syntax::ident::{
    is_operator_ident, is_wildcard_ident, normalize_ident, validate_ident, validate_package_path,
};
use reciplexa_syntax::number_lit::{is_f64_literal_form, parse_int_literal, parse_number_literal};
use reciplexa_syntax::string_lit::{
    decode_string_literal, encode_string_literal, special_char_value, unicode_scalar_value,
    virtual_close_delimiter,
};

#[test]
fn number_lit_f64_form_and_sign_edge() {
    assert!(is_f64_literal_form("1.0"));
    assert!(is_f64_literal_form("1e3"));
    assert!(is_f64_literal_form("1E3"));
    assert!(!is_f64_literal_form("42"));
    assert!(!is_f64_literal_form("-"));
    assert!(!is_f64_literal_form("+"));
    assert!(!is_f64_literal_form(""));
}

#[test]
fn number_lit_int_ok_and_reject_matrix() {
    assert_eq!(parse_int_literal("42").unwrap(), 42);
    assert_eq!(parse_int_literal("-7").unwrap(), -7);
    assert_eq!(parse_int_literal("+9").unwrap(), 9);
    assert_eq!(parse_int_literal("0xff").unwrap(), 255);
    assert_eq!(parse_int_literal("0b1010").unwrap(), 10);
    assert_eq!(parse_int_literal("0o17").unwrap(), 15);
    assert_eq!(parse_int_literal("1_000").unwrap(), 1000);

    for bad in [
        "", "-", "+", "1.5", "1e2", "0x", "0b", "0o", "0x_", "0b__1", "_1", "1_", "007", "00",
        "ff", "0xg", "0b2", "0o9",
    ] {
        assert!(parse_int_literal(bad).is_err(), "{bad}");
    }
    // Overflow i128
    assert!(parse_int_literal("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").is_err());
}

#[test]
fn number_lit_f64_ok_and_reject_matrix() {
    assert_eq!(parse_number_literal("42").unwrap(), 42.0);
    assert_eq!(parse_number_literal("-0xff").unwrap(), -255.0);
    assert_eq!(parse_number_literal("+0b10").unwrap(), 2.0);
    assert_eq!(parse_number_literal("0o10").unwrap(), 8.0);
    assert_eq!(parse_number_literal("1.5").unwrap(), 1.5);
    assert_eq!(parse_number_literal("1e3").unwrap(), 1000.0);
    assert_eq!(parse_number_literal("1.5e+2").unwrap(), 150.0);
    assert_eq!(parse_number_literal("1.5e-1").unwrap(), 0.15);
    assert_eq!(parse_number_literal("1_000.5").unwrap(), 1000.5);

    for bad in [
        "", "-", "+", "007", "1.", "1e", "1e+", "1e-", "1__0", "_1", "1_", "0x", "0b_", "1.2.3",
        "1e1e2",
    ] {
        assert!(parse_number_literal(bad).is_err(), "{bad}");
    }

    // Huge hex → f64 iterative path (beyond u128).
    let huge = format!("0x{}", "f".repeat(40));
    let v = parse_number_literal(&huge).unwrap();
    assert!(v.is_finite() && v > 0.0);
}

#[test]
fn string_lit_malformed_and_virtual_close_matrix() {
    assert!(decode_string_literal("").is_err());
    assert!(decode_string_literal("\"").is_err());
    assert!(decode_string_literal("abc").is_err());
    assert!(decode_string_literal("\"abc").is_err());
    assert!(decode_string_literal("\"\"x\"\"").is_err()); // open==2 with body
    assert_eq!(decode_string_literal("\"\"").unwrap(), "");
    assert_eq!(decode_string_literal("\"hi\"").unwrap(), "hi");

    // Multi-quote too short / mismatched closer
    assert!(decode_string_literal("\"\"\"").is_err());
    assert!(decode_string_literal("\"\"\"abc\"\"").is_err());

    // Dedent failure: shallower line than closer
    let bad_dedent = "\"\"\"\n  a\nb\n  \"\"\"";
    assert!(decode_string_literal(bad_dedent).is_err());

    // Closer shares line with content (no indent baseline)
    let shared = "\"\"\"hello\"\"\"";
    assert_eq!(decode_string_literal(shared).unwrap(), "hello");

    // CR/LF normalize inside multi
    let crlf = "\"\"\"\r\na\r\nb\r\n\"\"\"";
    assert_eq!(decode_string_literal(crlf).unwrap(), "a\nb");

    assert_eq!(virtual_close_delimiter(""), None);
    assert_eq!(virtual_close_delimiter("abc"), None);
    assert_eq!(virtual_close_delimiter("\"\""), None);
    assert_eq!(virtual_close_delimiter("\"\"x"), Some("\"".into()));
    assert_eq!(virtual_close_delimiter("\"\"\""), Some("\"\"\"".into()));
    assert_eq!(virtual_close_delimiter("\"\"\"abc"), Some("\"\"\"".into()));
    assert_eq!(
        virtual_close_delimiter("\"\"\"abc\"\""),
        Some("\"\"\"".into())
    );
    assert_eq!(virtual_close_delimiter("\"ok\""), None);
}

#[test]
fn string_lit_encode_and_specials() {
    assert_eq!(encode_string_literal(""), "\"\"");
    assert_eq!(encode_string_literal("plain"), "\"plain\"");
    let multi = encode_string_literal("a\"b\nc");
    assert!(multi.starts_with("\"\"\""));
    assert_eq!(decode_string_literal(&multi).unwrap(), "a\"b\nc");

    // Force longer delimiter when content contains """
    let needs4 = encode_string_literal("say \"\"\" hi");
    assert!(needs4.starts_with("\"\"\"\""));
    assert_eq!(decode_string_literal(&needs4).unwrap(), "say \"\"\" hi");

    assert_eq!(special_char_value("tab"), Some("\t"));
    assert_eq!(special_char_value("carriage-return"), Some("\r"));
    assert_eq!(special_char_value("nul"), Some("\0"));
    assert_eq!(special_char_value("nope"), None);
    assert!(unicode_scalar_value(f64::NAN).is_err());
    assert!(unicode_scalar_value(1.5).is_err());
    assert_eq!(unicode_scalar_value(0.0).unwrap(), "\0");
}

#[test]
fn ident_validate_residual_matrix() {
    assert!(is_operator_ident("+"));
    assert!(is_wildcard_ident("_"));
    assert_eq!(normalize_ident("a"), "a");

    assert!(validate_ident("...").is_ok());
    assert!(validate_ident("...+").is_ok());
    assert!(validate_ident("$").is_err());
    assert!(validate_ident("").is_err());
    assert!(validate_ident("1abc").is_err());
    assert!(validate_ident("?x").is_err());
    assert!(validate_ident("!x").is_err());
    assert!(validate_ident("a-?").is_err());
    assert!(validate_ident("a?!").is_err());
    assert!(validate_ident("a?b").is_err());
    assert!(validate_package_path("").is_err());
    assert!(validate_package_path("/a").is_err());
    assert!(validate_package_path("a/").is_err());
    assert!(validate_package_path("A/b").is_err());
    assert!(validate_ident(&format!("a{}", '\u{200b}')).is_err());
}
