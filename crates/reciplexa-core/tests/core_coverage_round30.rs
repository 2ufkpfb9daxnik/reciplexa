//! Round-30: final push past 98.5% — try/check-cast type `?`, handler,
//! local/rec/var nests, pattern payload Err, ErrorNode expr.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn elaborate_round30_cast_handler_local_pattern() {
    for src in [
        // try/check-cast target parse_type_atom Err (2478 / 2505)
        "(val main (try-cast 1 (val x 1)))",
        "(val main (check-cast 1 (val x 1)))",
        "(val main (try-cast 1 1e9999))",
        "(val main (check-cast 1 1e9999))",
        "(val main (try-cast 1 \"unterminated\n))",
        "(val main (check-cast 1 \"unterminated\n))",
        "(val main (as (val x 1) 1))",
        "(val main (try-cast (val x 1) int))",
        // handler clause Err (2592)
        "(val main (handler ask (val x 1)))",
        "(val main (handler ask 1e9999))",
        "(val main (with (handler ask (val x 1)) 1))",
        // local / rec / var / letrec body Err
        "(val main (local (val x (val y 1)) x))",
        "(val main (local (var x (val y 1)) x))",
        "(val main (local (rec (val f (fn (x) (val y 1)))) (f 1)))",
        "(val main (local (type t int) (val t (val y 1)) t))",
        "(val main (rec (val f (fn (x) x)) (val y 1)))",
        "(val main (rec (type T int) (val f (fn (x) (val y 1))) (f 1)))",
        "(val main (var x (val y 1) x))",
        "(val main (var x 1 (val y 1)))",
        "(val main (letrec ((f (fn (x) (val y 1)))) (f 1)))",
        "(val main (let ((x 1)) (val y 1)))",
        // pattern payload nested Err (2154 / 2218 / 2240)
        "(val main (match (tuple 1 2) ((tuple 1e9999 b) -> 0) (_ -> 1)))",
        "(val main (match (tuple 1 2) ((tuple \"unterminated\n b) -> 0) (_ -> 1)))",
        "(val main (match (record (a 1)) ((record (a 1e9999)) -> 0) (_ -> 1)))",
        "(data w (mk int string))\n(val main (match (mk 1 \"a\") ((mk 1e9999 y) -> 0) (_ -> 1)))",
        "(data w (mk int))\n(val main (match (mk 1) ((mk \"unterminated\n) -> 0) (_ -> 1)))",
        // ErrorNode / incomplete
        "(val main (",
        "(val main )",
        "(",
        // data payload nested type Err (457)
        "(data t (c (val x 1)))\n(val main 1)",
        "(data t (c 1e9999))\n(val main 1)",
        // forall / type head Node
        "(type t (forall (val x 1) a))\n(val main 1)",
        // binder_name in local type / rec annotations
        "(val main (local (type Foo int) 1))",
        "(val main (rec (type Foo (fn int int)) (val f (fn (x) x)) (f 1)))",
        "(val main (rec (val Foo (fn (x) x)) (Foo 1)))",
        // row last + empty param still
        "(type t (record (a int) (row r) (b int)))\n(val main 1)",
        "(data t (()) )\n(val main 1)",
        // ambient perform arg Err already; body Ok nearby
        "(val main (log \"x\"))",
        "(val main (seq 1 2 3))",
    ] {
        tip(src);
    }
}
