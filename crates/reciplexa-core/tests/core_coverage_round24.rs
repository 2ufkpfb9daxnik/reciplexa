//! Round-24 core: elaborate residual `?` Err nests + Rec/main selection +
//! duplicate-binding clusters still under the live-hash miss map.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn elaborate_round24_type_nested_err_and_ok() {
    let cases = [
        // Nested type `?` Err arms (dynamic / not / diff / tuple / record / forall / effects)
        "(type t (dynamic (1)))\n(val main 1)",
        "(type t (not (1)))\n(val main 1)",
        "(type t (diff int (1)))\n(val main 1)",
        "(type t (diff (1) int))\n(val main 1)",
        "(type t (tuple int (1) string))\n(val main 1)",
        "(type t (record (a (1))))\n(val main 1)",
        "(type t (record (optional a (1))))\n(val main 1)",
        "(type t (record (a int) (row r) (b int)))\n(val main 1)",
        "(type t (forall ((a type)) (1)))\n(val main 1)",
        "(type t (forall ((a type)(b type)) (fn a b)))\n(val main 1)",
        "(type t (fn int int (effects (ask (1)))))\n(val main 1)",
        "(type t (fn int int (effects ask (state int))))\n(val main 1)",
        // BracketList / Node-as-type → "expected a type"
        "(type t [int])\n(val main 1)",
        "(val main (as [int] 1))",
        "(val main (as ((int)) 1))",
        // String singleton decode Err
        "(type t \"bad\\q\")\n(val main 1)",
        "(type t \"unterminated)",
        "(type t (singleton \"bad\\q\"))\n(val main 1)",
        // Numeric singleton Ok + F64 reject
        "(type t 42)\n(val main 1)",
        "(type t 1.5)\n(val main 1)",
        "(type t 0x2A)\n(val main 1)",
        // App type with bad arg
        "(data box ((a type)) (mk a))\n(type t (box (1)))\n(val main 1)",
        // Intersect flatten neighborhood
        "(type t (intersect (intersect int number) string))\n(val main 1)",
        "(type t (intersect any int))\n(val main 1)",
        "(type t (union int (union string bool)))\n(val main 1)",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round24_data_rec_dup_and_main_select() {
    let cases = [
        // Duplicate top-level Single / Rec
        "(val x 1)\n(val x 2)\n(val main 1)",
        "(val f (fn (x) x))\n(rec (val f (fn (y) y)))\n(val main 1)",
        "(rec (val f (fn (x) x)) (val f (fn (y) y)))\n(val main 1)",
        // Rec containing main (L197) vs last-binding fallback (L202)
        "(rec (val main (fn (x) x)))",
        "(rec (val f (fn (x) x)) (val main (fn (y) y)))",
        "(rec (val f (fn (x) x)))",
        "(rec (val a (fn (x) x)) (val b (fn (y) y)))",
        // rec data group Err (token / empty / non-data)
        "(rec (data))",
        "(rec (data t))",
        "(rec (data t (c)) foo)\n(val main 1)",
        "(rec foo (data t (c)))\n(val main 1)",
        // Empty type-param section is guarded; still tip nearby payload `?`
        "(data box ((a type)) (mk (1)))\n(val main 1)",
        "(data box ((a type)) (mk (fn (1) a)))\n(val main 1)",
        "(data t (((tag))) )\n(val main 1)",
        "(data t Foo)\n(val main 1)",
        "(data t foo_bar)\n(val main 1)",
        // Nullary ctor binder_name Err
        "(data t BadTag)\n(val main 1)",
        "(data Bad (c))\n(val main 1)",
        // type / type-alias binder Err + head Node
        "(type Foo int)\n(val main 1)",
        "(type-alias Foo int)\n(val main 1)",
        "(type foo_bar int)\n(val main 1)",
        "((type) t int)\n(val main 1)",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round24_expr_pattern_cast_local() {
    let cases = [
        // unicode via Number token Err + Int/F64/Number atom paths
        "(val main (unicode 0b2))",
        "(val main (unicode 1__0))",
        "(val main (unicode 65))",
        "(val main (unicode 65.0))",
        "(val main (unicode 0x41))",
        "(val main (unicode (+ 1 64)))",
        "(val main (unicode true))",
        // val binder Node Err + named sugar binder_name
        "(val ((x)) 1)",
        "(val (Foo x) x)",
        "(val (ok Foo) 1)",
        // bytes / list / tuple / record nested `?`
        "(val main (bytes 0b2))",
        "(val main (bytes 300))",
        "(val main (list (1 2)))",
        "(val main (tuple 1 ( )))",
        "(val main (record (a (1 2))))",
        "(val main (field (record (a 1)) (a)))",
        "(val main (record-update (record (a 1)) (a (1 2))))",
        "(val main (record-extend (record (a 1)) (b (1 2))))",
        // match pattern literal / payload Err
        "(val main (match 1 (\"bad\\q\" -> 0) (_ -> 1)))",
        "(val main (match 1 (0b2 -> 0) (_ -> 1)))",
        "(val main (match 1 (1.5 -> 0) (_ -> 1)))",
        "(val main (match true (true -> 1) (false -> 0)))",
        "(val main (match false (false -> 0) (true -> 1)))",
        "(val main (match unit (unit -> 0) (_ -> 1)))",
        "(val main (match (tuple 1 2) ((tuple a b) -> a) (_ -> 0)))",
        "(val main (match (record (a 1)) ((record (a x)) -> x) (_ -> 0)))",
        "(data wrap (mk a))\n(val main (match (mk 1) ((mk +) -> 0) (_ -> 1)))",
        "(data wrap (mk a))\n(val main (match (mk 1) ((mk Foo) -> 0) (_ -> 1)))",
        // perform / raise / forward / casts denser Ok + nested Err
        "(val main (perform log \"x\"))",
        "(val main (perform ask (1 2)))",
        "(val main (raise (1 2)))",
        "(val main (forward Foo))",
        "(val main (try-cast (1 2) int))",
        "(val main (check-cast (1 2) int))",
        "(val main (as int (1 2)))",
        "(val main (as (1) 1))",
        "(val main (or-raise (as-result (fn () (1 2)))))",
        "(val main (as-result (1 2)))",
        // handle / with / handler
        "(val main (handle ask (fn (m k) (k m)) (perform ask (1 2))))",
        "(val main (with (handler ask (fn (m k) (k m))) (1 2)))",
        "(val main (handler ask (fn (m k) (1 2))))",
        // local / let / letrec / set / if / var
        "(val main (local (val Foo 1) Foo))",
        "(val main (local (type Foo int) (val Foo 1) Foo))",
        "(val main (local (var Foo 1) Foo))",
        "(val main (local (var x 1) (set Foo 2) x))",
        "(val main (let ((Foo 1)) Foo))",
        "(val main (letrec ((Foo (fn (x) x))) (Foo 1)))",
        "(val main (if (1 2) 1 0))",
        "(val main (if true (1 2) 0))",
        "(val main (if true 1 (1 2)))",
        "(val main (var Foo 1 Foo))",
        "(val main (set Foo 1))",
        // ErrorNode-ish recovery + quarantined
        "(val main )",
        "(page a4)",
        "(circle 1 2 3)",
        // seq ambient
        "(val main (seq 1 2 3))",
        "(val main (seq (// c) 1))",
        // fn param binder Err
        "(val main (fn (Foo) Foo))",
        "(fn Foo (x) x)\n(val main 1)",
        "(fn ok (Foo) Foo)\n(val main 1)",
    ];
    for src in cases {
        tip(src);
    }
}
