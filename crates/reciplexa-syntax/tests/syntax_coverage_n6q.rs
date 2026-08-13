//! N6 tip q: lexer / number / string / markup residual tips.

use reciplexa_syntax::ident::validate_package_path;
use reciplexa_syntax::lexer::Lexer;
use reciplexa_syntax::number_lit::{is_f64_literal_form, parse_int_literal, parse_number_literal};
use reciplexa_syntax::parse_source;
use reciplexa_syntax::string_lit::decode_string_literal;

#[test]
fn syntax_n6q_lexer_number_string_residuals() {
    // Trailing ?/! not glued when followed by ident-continue (`foo?x`).
    let _ = Lexer::new("foo?x foo!y <=a +=b").tokenize_all();
    // Fraction consume fail / scientific edges
    let _ = Lexer::new("1._ 1. 1e 1e+ 1e- 0b 0o 0x 007").tokenize_all();
    // Multi-quote / empty string
    let _ = Lexer::new("\"\" \"hi\" \"\"\"\n  body\n  \"\"\"").tokenize_all();
    // Operators freestanding vs glued
    let _ = Lexer::new("<= >= != < > + - * / //").tokenize_all();

    let _ = is_f64_literal_form("+");
    let _ = is_f64_literal_form("1.5");
    let _ = is_f64_literal_form("1e3");
    let _ = parse_int_literal("");
    let _ = parse_int_literal("_");
    let _ = parse_int_literal("1__0");
    let _ = parse_number_literal("1e99999");
    let _ = parse_number_literal("+");
    let _ = decode_string_literal("\"");
    let _ = decode_string_literal("\"hi");
    let _ = decode_string_literal("\"\"");
    let _ = decode_string_literal("\"\"x\"\"");
    let _ = decode_string_literal("\"\"\"\n a\n\"\"\"");
    let _ = validate_package_path("ab");
    let _ = validate_package_path("a-b");
    let _ = validate_package_path("a--b");
    let _ = validate_package_path("a-");
    let _ = validate_package_path("Foo");
    let _ = validate_package_path("a_b");
    let _ = validate_package_path("graphics/shapes");

    for src in [
        "(markup @foo[args only])",
        "(markup @foo{brace})",
        "(markup before @(+ 1 2) after)",
        "(markup @x[])",
        "(markup   spaced   )",
    ] {
        let _ = parse_source(src);
    }
}
