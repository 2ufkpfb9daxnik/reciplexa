//! Round-19 core: remaining unique elaborate miss neighborhoods (quarantine
//! top-level heads, rec-only body pick, ambient ops, match lit patterns,
//! try/check-cast never, multi-payload ctors, local type+val).

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn elaborate_round19_top_quarantine_and_rec_only_body() {
    let cases = [
        // Top-level quarantine heads (try_top_decl None → is_quarantined)
        "(page a4)",
        "(markup (text \"x\"))",
        "(src \"x.rpx\")",
        "(circle 1 2 3)",
        "(rect 0 0 1 1)",
        "(text \"hi\")",
        "(group (circle 1 2 3))",
        // Rec-only top bindings, no trailing expr → body from last/main Rec binder
        "(rec (val f (fn (x) x)) (val g (fn (y) y)))",
        "(rec (val f (fn (x) x)) (val main f))",
        "(rec (type f (fn int int)) (val f (fn (x) x)) (val main f))",
        // Named top-level fn reserved + happy named fn
        "(fn if (x) x)",
        "(fn add (x y) (+ x y))\n(val main (add 1 2))",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round19_ambient_ctors_list_tuple_bytes() {
    let cases = [
        // Ambient effect ops (arity + happy)
        "(val main (random))",
        "(val main (random 1))",
        "(val main (log \"hi\"))",
        "(val main (log))",
        "(val main (log \"a\" \"b\"))",
        "(val main (read-file \"a\" \"b\"))",
        // Multi-payload / nullary constructors
        "(data pair (mk a b))\n(val main (mk 1 2))",
        "(data flag (on) (off))\n(val main on)",
        "(data box (mk x))\n(val main (mk 1))",
        "(data bad (mk a))\n(val main (mk 1 2))",
        // list / tuple / bytes happy + residual Err
        "(val main (list 1 2 3))",
        "(val main (list))",
        "(val main (tuple))",
        "(val main (tuple 1 2))",
        "(val main (tuple 1 2 3))",
        "(val main (bytes 0 255))",
        "(val main (bytes (list 1)))",
        "(val main (bytes true))",
        // record-extend / field happy
        "(val main (record-extend (record (a 1)) (b 2)))",
        "(val main (field (record (a 1)) a))",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round19_match_lits_cast_handler_local() {
    let cases = [
        // Literal / bind / unit / bool match patterns (happy + arity Err)
        "(val main (match true (true -> 1) (_ -> 0)))",
        "(val main (match false (false -> 1) (_ -> 0)))",
        "(val main (match unit (unit -> 1) (_ -> 0)))",
        "(val main (match 1 ((bind x) -> x) (_ -> 0)))",
        "(val main (match \"hi\" (\"hi\" -> 1) (_ -> 0)))",
        "(val main (match true (true x -> 0) (_ -> 1)))",
        "(val main (match unit (unit x -> 0) (_ -> 1)))",
        "(val main (match 1 ((_ x) -> 0) (_ -> 1)))",
        // f64 literal pattern / type singleton reject
        "(val main (match 1 (1.5 -> 0) (_ -> 1)))",
        "(type t 1.5)\n(val t 1)\n(val main t)",
        "(type t (singleton 1.5))\n(val t 1)\n(val main t)",
        // try-cast / check-cast never-ish targets
        "(val main (try-cast 1 (intersect)))",
        "(val main (check-cast 1 (intersect)))",
        "(val main (try-cast 1 never))",
        "(val main (check-cast 1 never))",
        "(val main (try-cast 1 int))",
        "(val main (check-cast \"x\" string))",
        // as-result / handler / with residuals
        "(val main (as-result (fn () 1)))",
        "(val main (as-result (fn (x) x)))",
        "(val main (handler ask (fn (m) m)))",
        "(val main (handler ask (fn (m k) (k m))))",
        "(val main (handler ask (fn () 1)))",
        "(val main (handler ask (fn (a b c) a)))",
        "(val main (with (handler ask (fn (m) m)) (perform ask 1)))",
        // local type annotation + matching val
        "(val main (local (type u int) (val u 1) u))",
        "(val main (local (type-alias u int) (val x 1) x))",
        "(val main (local (type u int) 1))",
        // positivity / variance-ish data
        "(data box ((a type)) (mk))\n(val main (mk))",
        "(data box ((a type)) (mk (fn a int)))\n(val main 1)",
        "(data nest ((a type)) (mk (fn (fn a int) int)))\n(val main 1)",
    ];
    for src in cases {
        tip(src);
    }
}
