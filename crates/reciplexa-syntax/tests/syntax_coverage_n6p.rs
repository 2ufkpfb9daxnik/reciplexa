//! N6 tip p: parse UnexpectedToken / structured comment / unparse edges.

use reciplexa_syntax::{parse_source, unparse};

#[test]
fn syntax_n6p_parse_recovery_and_unparse() {
    for src in [
        "(val main 1)",
        "(// comment body)\n(val main 1)",
        "(// nested (still) ok)\n(val main 1)",
        "(// unclosed",
        ")",
        "(",
        "[1 2 3]",
        "{a: 1}",
        "@markup[hi]",
        "(val main \"unterminated)",
        "(val main 1e)",
        "(((val main 1)))",
        "(val main [1 2)",
        "(val main {a 1)",
        "unexpected",
        "1 2 3",
        "(val 1 main)",
        "(import graphics/shapes only circle)\n(val main circle)",
    ] {
        let parsed = parse_source(src);
        let _ = unparse(&parsed.root);
    }

    // Recovery: stray closers / unexpected tokens inside lists
    for src in [
        "(val main 1))",
        "((val main 1)",
        "(val main ])",
        "(val main })",
        "[(val main 1]",
        "{(val main 1}",
    ] {
        let _ = parse_source(src);
    }
}
