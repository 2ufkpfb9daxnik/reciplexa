//! Round-25 core: elaborate residual ErrorNode / pattern / binder clusters
//! still under the live-hash miss map after round24.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn elaborate_round25_error_node_and_patterns() {
    let cases = [
        // Error recovery / ErrorNode neighborhood
        "(val main (",
        "(val main )",
        "(match 1 (",
        // Literal pattern arity Err (number/string with args)
        "(val main (match 1 (1 x -> 0) (_ -> 1)))",
        "(val main (match \"a\" (\"a\" x -> 0) (_ -> 1)))",
        // true/false/unit with extra atoms (literal arity Err before Lit arms)
        "(val main (match true (true x -> 0) (_ -> 1)))",
        "(val main (match false (false x -> 0) (_ -> 1)))",
        "(val main (match unit (unit x -> 0) (_ -> 1)))",
        // Nested payload patterns with literals
        "(val main (match (tuple 1 true) ((tuple 1 true) -> 1) (_ -> 0)))",
        "(val main (match (tuple false unit) ((tuple false unit) -> 1) (_ -> 0)))",
        // Unsupported literal pattern neighborhood + f64 reject
        "(val main (match 1.5 (1.5 -> 0) (_ -> 1)))",
        "(val main (match 1e2 (1e2 -> 0) (_ -> 1)))",
        // String decode Err in pattern
        r#"(val main (match "x" ("bad\q" -> 0) (_ -> 1)))"#,
        // Quarantined / bad heads
        "(circle 1 2 3)",
        "(page a4)",
        // val binder Node form
        "(val (1) 2)",
        "(val main (as (1) 1))",
        // Color-byte / number literal Err nests
        "(val main (color-byte 1.5 0 0))",
        "(val main (color-byte 999 0 0))",
        "(val main (color-byte -1 0 0))",
        // List / seq / perform / forward / fail / try / cast nests
        "(val main (begin 1 2 3))",
        "(val main (perform log 1))",
        "(val main (forward k))",
        "(val main (fail \"x\"))",
        "(val main (try (fn () 1)))",
        "(val main (as int 1))",
        "(val main (check-cast int 1))",
        "(val main (try-cast int 1))",
        // record / update / extend / list sugar
        "(val main (record (a 1) (b 2)))",
        "(val main (record-update (record (a 1)) (a 2)))",
        "(val main (record-extend (record (a 1)) (b 2)))",
        "(val main (list 1 2 3))",
        // if / set / var / let / letrec / local
        "(val main (if true 1 0))",
        "(val main (let ((x 1)) x))",
        "(val main (letrec ((f (fn (x) x))) (f 1)))",
        "(val main (var x 1 x))",
        "(val main (set x 2))",
        // match nested ctors / record patterns
        "(val main (match (some 1) ((some x) -> x) (_ -> 0)))",
        "(val main (match (record (a 1)) ((record (a x)) -> x) (_ -> 0)))",
        // data / type residual nearby
        "(data t (c int))\n(val main (c 1))",
        "(type t (record (a int) (optional b string)))\n(val main 1)",
        "(type t (forall ((a type)) a))\n(val main 1)",
        "(type t (fn int int (effects ask)))\n(val main 1)",
        // forall binder list shape Err
        "(type t (forall a a))\n(val main 1)",
        "(type t (forall (a) a))\n(val main 1)",
        // empty rec data / non-data in rec group
        "(rec)",
        "(rec (val f (fn (x) x)))",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn check_round25_residual_matrix() {
    let cases = [
        // Multi-payload ctor missing field / arity mismatch
        "(data pair (mk int string))\n(val main (match (mk 1) ((mk a b) -> a) (_ -> 0)))",
        "(data pair (mk int string))\n(val main (match (mk 1 \"x\") ((mk a) -> a) (_ -> 0)))",
        "(data box (mk))\n(val main (match (mk) ((mk x) -> 1) (_ -> 0)))",
        // Variant pattern nested bind when tag/payload diverge
        "(data opt (none) (some int))\n(val main (match none ((some (tuple x y)) -> x) (none -> 0) (_ -> 1)))",
        "(val main (match 1 ((some x) -> x) (_ -> 0)))",
        // Optional field access → option type
        "(type r (record (optional a int)))\n(val main (fn (x) (.: x a)))",
        // number? occurrence + numeric binop
        "(val main (fn (x) (if (number? x) (+ x 1) 0)))",
        "(val main (fn (x) (if (string? x) x \"\")))",
        // Local state escape
        "(val main (var s 1 (fn () s)))",
        "(val main (var s 1 (handler ask (fn (m k) s))))",
        // letrec with type annotation alias
        "(type F (fn int int))\n(val main (letrec ((f (fn (x) x))) (f 1)))",
        // Open record / row unify neighborhood
        "(type r (record (a int) (row rho)))\n(val main (fn (x) (.: x a)))",
        "(val main (record-update (record (a 1)) (b 2)))",
        // Cast / as / check-cast type stage
        "(val main (as (union int string) 1))",
        "(val main (check-cast (intersect int number) 1))",
        "(val main (try-cast (diff int string) 1))",
        // Match exhaustiveness / wildcard
        "(data t (a) (b))\n(val main (match (a) (a -> 1)))",
        "(data t (a int) (b))\n(val main (match (a 1) ((a x) -> x) (b -> 0)))",
    ];
    for src in cases {
        tip(src);
    }
}
