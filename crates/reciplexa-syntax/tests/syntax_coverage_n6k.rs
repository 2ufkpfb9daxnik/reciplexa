//! N6 tip k: number_lit / ident residual edges toward 99%.

use reciplexa_syntax::number_lit::{is_f64_literal_form, parse_int_literal, parse_number_literal};
use reciplexa_syntax::{parse_source, validate_package_path};

#[test]
fn number_lit_n6k_sign_radix_and_separators() {
    for bad in ["", "+", "-", "0x", "0b", "0o", "0x_", "0b_", "0o_", "_1", "1_", "1__2"] {
        let _ = parse_int_literal(bad);
        let _ = parse_number_literal(bad);
        let _ = is_f64_literal_form(bad);
    }
    for ok in [
        "0", "+0", "-0", "42", "+42", "-42", "1_000", "0xFF", "+0xff", "-0x10", "0b1010", "0o77",
        "1.5", "+1.5", "-1e3", "1E-2", "0.0",
    ] {
        let _ = parse_number_literal(ok);
        let _ = is_f64_literal_form(ok);
    }
    for ok in ["0", "42", "0xFF", "0b101", "0o7", "1_2_3", "+7", "-7"] {
        let _ = parse_int_literal(ok);
    }
    // Leading-zero reject / f64-as-int reject
    assert!(parse_int_literal("01").is_err());
    assert!(parse_int_literal("1.0").is_err());
    assert!(parse_int_literal("1e2").is_err());
    assert!(parse_number_literal("01").is_err());
    assert!(parse_number_literal("01.5").is_err());
    // i128 overflow
    assert!(parse_int_literal(&format!("0x{}", "f".repeat(40))).is_err());
    // Huge radix → iterative f64 path (+ non-finite if possible)
    let huge_x = format!("0x{}", "f".repeat(80));
    let _ = parse_number_literal(&huge_x);
    let huge_b = format!("0b{}", "1".repeat(200));
    let _ = parse_number_literal(&huge_b);
    let huge_inf = format!("0x{}", "f".repeat(500));
    let _ = parse_number_literal(&huge_inf);
    // Decimal parse Err neighborhood
    let _ = parse_number_literal("1.2.3");
    let _ = parse_number_literal("1e");
    let _ = parse_number_literal("1e+");
}

#[test]
fn ident_and_package_path_n6k_edges() {
    use reciplexa_syntax::{decode_string_literal, validate_ident};

    for p in [
        "a",
        "a/b",
        "graphics/shapes",
        "Foo",
        "a_b",
        "",
        "/",
        "a/",
        "/a",
        "a//b",
        "a--b",
        "a-",
        "-a",
        "foo-bar",
        "a1",
    ] {
        let _ = validate_package_path(p);
        let _ = validate_ident(p);
    }
    for src in [
        "(val main 1)",
        "(val main-foo 1)",
        "(val Main 1)",
        "(import graphics/shapes)",
        "(// comment)\n(val main 1)",
    ] {
        let _ = parse_source(src);
    }
    // string_lit open==0 / multi-quote edges
    let _ = decode_string_literal("not-a-string");
    let _ = decode_string_literal("\"\"");
    let _ = decode_string_literal("\"hi\"");
    let _ = decode_string_literal("\"\"\"\n  line\n  \"\"\"");
    let _ = decode_string_literal("\"\"\"line\n\"\"\"");
    let _ = decode_string_literal("\"\"\"\r\n\tx\r\n\t\"\"\"");
}
