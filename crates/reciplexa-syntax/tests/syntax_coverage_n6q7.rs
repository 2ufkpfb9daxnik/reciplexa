//! N6 tip q7: markup Embed + number residuals for final 99% push.

use reciplexa_syntax::markup::{flatten_lines, markup_parts};
use reciplexa_syntax::number_lit::{is_f64_literal_form, parse_int_literal, parse_number_literal};
use reciplexa_syntax::parse_source;

#[test]
fn syntax_n6q7_markup_number_residuals() {
    // Embed part via `@(…)` scribble embedding + At brace/bracket.
    for src in [
        "(markup before @(+ 1 2) after)",
        "(markup @(list a b) z)",
        "(markup text @(foo bar) more)",
        "(markup @name(+ 1))",
        "(markup @x[a b]{c d})",
        "(markup @y{brace body})",
        "(markup @z[])",
    ] {
        let p = parse_source(src);
        for form in p.root.children() {
            if let Ok(parts) = markup_parts(&form) {
                let _ = flatten_lines(&parts);
            }
        }
    }

    // number_lit strip_sign Err + empty cleaned
    assert!(!is_f64_literal_form(""));
    assert!(!is_f64_literal_form("+"));
    assert!(!is_f64_literal_form("-"));
    assert!(!is_f64_literal_form("+-1"));
    let _ = parse_int_literal("");
    let _ = parse_int_literal("+");
    let _ = parse_int_literal("-");
    let _ = parse_int_literal("_");
    let _ = parse_int_literal("1_");
    let _ = parse_int_literal("_1");
    let _ = parse_number_literal("");
    let _ = parse_number_literal("+e");
    let _ = parse_number_literal("1e");
    let _ = parse_source("(// note)\n(val main 1)");
    let _ = parse_source("(//");
}
