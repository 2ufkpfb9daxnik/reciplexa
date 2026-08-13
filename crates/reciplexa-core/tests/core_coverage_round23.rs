//! Round-23 core: light elaborate residual clusters still under ~96% after round22.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn elaborate_round23_unicode_singleton_errornode() {
    let cases = [
        // unicode: Int / F64 / Number / non-lit Err / arity
        "(val main (unicode 65))",
        "(val main (unicode 65.0))",
        "(val main (unicode 0x41))",
        "(val main (unicode (+ 60 5)))",
        "(val main (unicode))",
        "(val main (unicode 1 2))",
        "(val main (unicode \"x\"))",
        "(val main (unicode true))",
        // singleton type edges
        "(type s (singleton 1))\n(val main 1)",
        "(type s (singleton 1.5))\n(val main 1)",
        "(type s (singleton true))\n(val main 1)",
        "(type s (singleton false))\n(val main 1)",
        "(type s (singleton unit))\n(val main 1)",
        "(type s (singleton \"x\"))\n(val main 1)",
        "(type s (singleton \"bad\\q\"))\n(val main 1)",
        "(val main (as (singleton 1) 1))",
        "(val main (as (singleton 1.5) 1))",
        // dynamic / not / diff / optional-field Ok paths near type ?
        "(type t (dynamic string))\n(val main 1)",
        "(type t (not int))\n(val main 1)",
        "(type t (diff number int))\n(val main 1)",
        "(type t (optional-field int))\n(val main 1)",
        "(type t (tuple int string bool))\n(val main 1)",
        // forall / effects denser
        "(type id (forall ((a type)) (fn a a)))\n(val id (fn (x) x))\n(val main (id 1))",
        "(type f (fn int int (effects ask log)))\n(val main 1)",
        "(type f (fn int int (effects (state int))))\n(val main 1)",
        // val binder Node Err + named sugar
        "(val (1) 2)",
        "(val (Bad x) x)",
        "(val (ok x) x)\n(val main (ok 1))",
        // perform arity-0 / arity-1 / arity-n ambient
        "(val main (perform random))",
        "(val main (perform log \"x\"))",
        "(val main (random))",
        "(val main (log \"x\"))",
        // ErrorNode via bad tokens nearby in list
        "(val main (+))",
        "(val main (seq))",
        // record / field / list denser Ok for medium ?
        "(val main (record (a 1) (b 2) (c 3)))",
        "(val main (field (record (a 1) (b 2)) b))",
        "(val main (list 1 2 3 4))",
        "(val main (tuple 1 2 3))",
        "(val main (bytes 0 128 255))",
        // match denser patterns
        "(data wrap (mk a b))\n(val main (match (mk 1 2) ((mk x y) -> x) (_ -> 0)))",
        "(val main (match (record (a 1) (b 2)) ((record (a x) (b y)) -> y) (_ -> 0)))",
        "(val main (match (tuple 1 2 3) ((tuple a b c) -> c) (_ -> 0)))",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round23_local_rec_handle_casts() {
    let cases = [
        "(val main (local (val x 1) (val y 2) (+ x y)))",
        "(val main (local (var x 1) (set x 2) x))",
        "(val main (local (type t int) (val t 1) t))",
        "(val main (local (rec (val f (fn (x) x))) (f 1)))",
        "(rec (val f (fn (x) x)) (val main (f 1)))",
        "(val main (handle ask (fn (m k) (k m)) (perform ask 1)))",
        "(val main (with (handler ask (fn (m k) (k m))) (perform ask 1)))",
        "(val main (handler ask (fn (m k) (k m))))",
        "(val main (try-cast 1 int))",
        "(val main (check-cast 1 int))",
        "(val main (as int 1))",
        "(val main (as number 1.5))",
        "(val main (raise \"e\"))",
        "(val main (or-raise (as-result (fn () 1))))",
        "(val main (as-result (fn () 1)))",
        "(val main (forward k))",
        "(val main (if true 1 0))",
        "(val main (if false 1 0))",
        "(fn add (x y) (+ x y))\n(val main (add 1 2))",
        "(fn add [x y] (+ x y))\n(val main (add 1 2))",
        // BracketList / structured leftovers
        "(val main (fn [x] x))",
        "(val main (seq 1 (// c) 2))",
        "(// top)\n(val main 1)",
    ];
    for src in cases {
        tip(src);
    }
}
