//! N6 tip p2: markup Embed / bare @() / scribble residuals.

use reciplexa_syntax::parse_source;

#[test]
fn markup_n6p2_embed_and_at_forms() {
    for src in [
        "(markup hello @world[x] more)",
        "(markup before @(+ 1 2) after)",
        "(markup @title{Hi})",
        "(markup @image[\"a.png\"])",
        "(markup @emph{bold @strong{nested}})",
        "(markup leading @(list) trailing)",
        "(markup @foo[])",
        "(markup @bar{})",
        "(markup text @( ) more)",
        "(markup @name(inner words))",
        "(markup)",
        "(markup   )",
        "(markup @x{a} @y{b})",
    ] {
        let _ = parse_source(src);
    }
}
