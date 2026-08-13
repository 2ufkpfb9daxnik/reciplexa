//! N6 tip l: resolve.rs residual payload BracketList / path soft / handle / data.

use reciplexa_bind::{resolve_language_source, resolve_source};

#[test]
fn resolve_n6l_payload_bracket_path_handle_data() {
    for src in [
        // Payload pattern Atom::Node non-List → other arm (BracketList / BraceList)
        "(data opt (some x) (none))\n(val main (match (some 1) ((some [x]) -> x) (none -> 0)))",
        "(data opt (some x) (none))\n(val main (match (some 1) ((some {x}) -> x) (none -> 0)))",
        "(val main (match 1 ((tuple [a] [b]) -> 0) (_ -> 1)))",
        "(val main (match 1 ((record (a [x])) -> 0) (_ -> 1)))",
        // Nested sole pattern atom that is BracketList (not List)
        "(val main (match 1 ([x] -> 0) (_ -> 1)))",
        "(val main (match 1 ({x} -> 0) (_ -> 1)))",
        // Path soft-continue when already bound (lookup Some)
        "(val color/red 1)\n(val main color/red)",
        "(val a/b 1)\n(val main a/b)",
        // Unbound path Err
        "(val main no/such/path)",
        // Invalid path normalize
        "(val main Foo/Bar)",
        // data with type-param section Node + nullary/list ctors
        "(data box ((a type)) (mk a) nullary)\n(val main nullary)",
        "(data t ((a type)(b type)) (c a b))\n(val main 1)",
        "(data t (Foo) (bar))\n(val main 1)",
        // handle in resolve_source: skip(1) child walk
        "(src (handle log (perform log \"x\")))",
        "(src (handle ask (fn (m k) (k m)) (perform ask 1)))",
        "(src (handle log (circle 1 2 3)))",
        "(src (handle log (page a4 (circle 1 2 3))))",
        // type/val declare stubs under src
        "(src (type t int) (val t 1))",
        "(src (val x 1) (markup @heading(Hi)))",
        // let soft continues: non-list bindings / short pairs / non-ident binder
        "(val main (let 1 2))",
        "(val main (let (x) x))",
        "(val main (let ((1 2)) 1))",
        "(val main (let (((x) 1)) x))",
        "(val main (let ((foo/bar 1)) 1))",
        // bind pattern / wildcard / literals
        "(val main (match 1 ((bind x) -> x)))",
        "(val main (match 1 ((bind _) -> 0) (_ -> 1)))",
        "(val main (match true (true -> 1) (false -> 0)))",
        "(val main (match 1 (_ -> 0)))",
        // record / tuple nested declare
        "(val main (match (tuple 1 2) ((tuple a b) -> a)))",
        "(val main (match (record (a 1)) ((record (a x) (b y)) -> x) (_ -> 0)))",
        // fn top-level named
        "(fn f (x) x)\n(val main (f 1))",
        "(fn f (x y) (+ x y))\n(val main (f 1 2))",
        // local / rec / var
        "(val main (local (val x 1) x))",
        "(val main (local (var x 1) (set x 2) x))",
        "(val main (rec (val f (fn (x) x)) (f 1)))",
        // perform / forward / with / handler
        "(val main (perform log \"x\"))",
        "(val main (forward k))",
        "(val main (with (handler ask (fn (m) m)) 1))",
        "(val main (handler ask (fn (m k) (k m))))",
    ] {
        let _ = resolve_language_source(src);
        let _ = resolve_source(src);
    }
}
