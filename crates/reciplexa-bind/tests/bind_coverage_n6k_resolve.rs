//! N6 tip k: resolve.rs residual soft continues + brace/path/handle edges.

use reciplexa_bind::{resolve_language_source, resolve_source};

#[test]
fn resolve_n6k_brace_path_and_handle_edges() {
    for src in [
        // Non-list expr node walk (BraceList / nested)
        "(val main {})",
        "(val main { a b })",
        "(val main { (x) })",
        "(val main [{ a }])",
        // Path soft-continue when already bound
        "(val color/black 1)\n(val main color/black)",
        "(val graphics/shapes 1)\n(val main graphics/shapes)",
        // Unbound path
        "(val main missing/path)",
        // Pattern other / nested list
        "(val main (match 1 ((tuple (a) (b)) -> 0) (_ -> 1)))",
        "(val main (match 1 ((some (record (a x))) -> x) (_ -> 0)))",
        // resolve_source handle children
        "(src (handle log (circle 1 2 3 red)))",
        "(src (handle ask (handle log (perform ask 1))))",
        "(src (handle log (page a4)))",
        // Markup skip + type/val declare
        "(markup @heading(Hi))",
        "(type title str)\n(val title \"x\")\n(page a4)",
    ] {
        let _ = resolve_language_source(src);
        let _ = resolve_source(src);
    }
}
