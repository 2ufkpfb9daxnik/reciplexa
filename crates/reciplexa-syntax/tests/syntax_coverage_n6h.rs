//! N6 tip: syntax parse.rs structured-comment / @-expr residuals + number_lit edges.

use reciplexa_syntax::number_lit::{is_f64_literal_form, parse_int_literal, parse_number_literal};
use reciplexa_syntax::parse_source;

#[test]
fn parse_structured_comment_and_at_expr_residuals() {
    for src in [
        // Exact `(//` at EOF → at_structured_comment None arm + unclosed body
        "(//",
        "(// ",
        "(//\n",
        // Trivia between `(` and `//`
        "( //)",
        "( \t// comment-body)",
        // Ident after `(` is not exactly `//` (structured-comment error arm)
        "(//not-a-comment)",
        "(///)",
        "(//x)",
        // Unclosed @-expr
        "@(",
        "@(foo",
        "@(foo bar",
        // Mixed with real forms
        "(// c)\n(val main 1)",
        "(val main (seq 1 (// nest) 2))",
        // Other parse recovery edges nearby
        "(",
        ")",
        "(()",
        "(@)",
    ] {
        let _ = parse_source(src);
    }
}

#[test]
fn number_lit_residual_nonfinite_and_edges() {
    // strip_sign never Err → is_f64 False arm via empty-ish; still poke forms
    assert!(!is_f64_literal_form(""));
    assert!(!is_f64_literal_form("-"));
    assert!(!is_f64_literal_form("+"));
    assert!(is_f64_literal_form("1e1"));
    assert!(is_f64_literal_form("1E1"));
    assert!(is_f64_literal_form("1.0"));
    assert!(!is_f64_literal_form("42"));

    // Out of range / empty cleaned / octal number path
    assert!(parse_int_literal("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").is_err());
    assert!(parse_int_literal("0b").is_err());
    assert!(parse_int_literal("0o").is_err());
    assert!(parse_int_literal("0x_").is_err());
    let _ = parse_int_literal("0o17");
    let _ = parse_number_literal("0o52");
    let _ = parse_number_literal("+0o10");
    let _ = parse_number_literal("-0o7");

    // Non-finite decimal / huge radix iterative path
    assert!(parse_number_literal("1e999999").is_err());
    assert!(parse_number_literal("-1e999999").is_err());
    let huge = format!("0x{}", "f".repeat(80));
    let _ = parse_number_literal(&huge);
    let huge_o = format!("0o{}", "7".repeat(80));
    let _ = parse_number_literal(&huge_o);
    let huge_b = format!("0b{}", "1".repeat(400));
    let _ = parse_number_literal(&huge_b);
}
