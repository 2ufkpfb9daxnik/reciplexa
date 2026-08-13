//! N6 tip i: syntax parse.rs — @-expr optional paren-body unclosed arm
//! (prior n6h used `@(…)` which parses as the *form*, not the markup body).

use reciplexa_syntax::parse_source;

#[test]
fn parse_n6i_at_expr_paren_body_unclosed() {
    for src in [
        // Form is Ident, then unclosed markup `(…)` body → L438–445
        "@foo(",
        "@foo(bar",
        "@foo(bar baz",
        "@em(hi",
        "@section(title",
        // With bracket args then unclosed paren body
        "@foo[1](",
        "@foo[a b](c",
        // Inside markup / mixed
        "(markup @em(hi)",
        "(markup @section(title)",
        "(markup @foo[1](x)",
        // Closed controls (ensure happy path still nearby)
        "@foo(bar)",
        "@foo[1](bar)",
        "(markup @em(hi))",
        // Structured-comment lookahead edges that still enter via eat_trivia
        "(//)",
        "(// )",
        "( // )",
        "(// nested (x) y)",
        // EOF after structured open
        "(//",
        "(// ",
    ] {
        let _ = parse_source(src);
    }
}
