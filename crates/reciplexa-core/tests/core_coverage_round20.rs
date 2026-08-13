//! Round-20 core: remaining unique elaborate miss neighborhoods (data ctor edges,
//! type syntax residuals, match/record/local/rec error arms, ambient multi-arg).

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn elaborate_round20_data_ctor_and_param_errors() {
    let cases = [
        // data param / ctor shape residuals
        "(data t ((a type)))\n(val main 1)",
        "(data t ((a type) (b type)) (mk a))\n(val main 1)",
        "(data t (() ))\n(val main 1)",
        "(data t ((1)))\n(val main 1)",
        "(data t ((1 x)))\n(val main 1)",
        "(data t (\"bad\"))\n(val main 1)",
        "(data t (c ( )))\n(val main 1)",
        "(data t (c (1)))\n(val main 1)",
        "(data t (c (fn)))\n(val main 1)",
        "(data t (c (fn int)))\n(val main 1)",
        // type-param pair shape
        "(data t ((a)) (mk))\n(val main 1)",
        "(data t (((a) type)) (mk))\n(val main 1)",
        "(data t ((a (type))) (mk))\n(val main 1)",
        "(data t ((1 type)) (mk))\n(val main 1)",
        "(data t ((a kind)) (mk))\n(val main 1)",
        "(data t ((a type) a) (mk))\n(val main 1)",
        // nullary + payload ctor mix / reserved binder
        "(data flag on off)\n(val main on)",
        "(data box (mk x y))\n(val main (mk 1 2))",
        "(fn if (x) x)\n(val main 1)",
        "(val (if x) 1)\n(val main 1)",
        "(val (1 x) 1)\n(val main 1)",
        "(val ((f x)) 1)\n(val main 1)",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round20_type_syntax_residuals() {
    let cases = [
        // named type atoms / singletons / dynamic / diff / intersect
        "(type t dynamic)\n(val t 1)\n(val main t)",
        "(type t false)\n(val t false)\n(val main t)",
        "(type t true)\n(val t true)\n(val main t)",
        "(type t \"hi\")\n(val t \"hi\")\n(val main t)",
        "(type t 42)\n(val t 42)\n(val main t)",
        "(type t (dynamic int))\n(val t 1)\n(val main t)",
        "(type t (diff number int))\n(val t 1.5)\n(val main t)",
        "(type t (intersect int number))\n(val t 1)\n(val main t)",
        "(type t (intersect))\n(val main 1)",
        // forall / record / effect-row edges
        "(type t (forall ((a type)) a))\n(val t 1)\n(val main t)",
        "(type t (forall () int))\n(val main 1)",
        "(type t (forall ((a type))))\n(val main 1)",
        "(type t (record 1 2))\n(val main 1)",
        "(type t (record a))\n(val main 1)",
        "(type t (record (a int) (row r) (b int)))\n(val main 1)",
        "(type t (record (row (r))))\n(val main 1)",
        "(type t (record (optional a int)))\n(val t (record))\n(val main t)",
        "(type t (fn int))\n(val main 1)",
        "(type t (effects (log string) int))\n(val main 1)",
        "(type t (effects ((log)) int))\n(val main 1)",
        "(type t (effects (1) int))\n(val main 1)",
        // type-alias / bad heads
        "(type-alias t int)\n(val t 1)\n(val main t)",
        "(type (t) int)\n(val main 1)",
        "(type 1 int)\n(val main 1)",
        "(type t t)\n(val main 1)",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round20_expr_match_record_local() {
    let cases = [
        // seq / perform / ambient arity
        "(val main (seq 1 2 3))",
        "(val main (perform log \"a\"))",
        "(val main (perform log))",
        "(val main (perform log \"a\" \"b\"))",
        "(val main (random))",
        "(val main (failure \"x\"))",
        "(val main (forward k))",
        // record forms
        "(val main (record (a 1) (b 2)))",
        "(val main (record a))",
        "(val main (record (1 2)))",
        "(val main (record-update (record (a 1)) (a 2)))",
        "(val main (record-update (record (a 1)) a))",
        "(val main (record-update (record (a 1)) (1 2)))",
        "(val main (record-extend (record (a 1)) (b 2)))",
        "(val main (field (record (a 1)) a))",
        "(val main (field (record (a 1)) 1))",
        // list / tuple / bytes
        "(val main (list 1 2))",
        "(val main (tuple 1 2 3))",
        "(val main (bytes 0 1 255))",
        "(val main (bytes 1.5))",
        // match residuals
        "(val main (match 1 (1 -> 0) (_ -> 1)))",
        "(val main (match 1 1))",
        "(val main (match true (true -> 1) (false -> 0)))",
        "(val main (match false (false -> 1) (_ -> 0)))",
        "(val main (match unit (unit -> 1) (_ -> 0)))",
        "(val main (match (tuple 1 2) ((tuple a b) -> a) (_ -> 0)))",
        "(val main (match (record (a 1)) ((record (a x)) -> x) (_ -> 0)))",
        "(val main (match (record (a 1)) ((record a) -> 0) (_ -> 1)))",
        "(val main (match on ((on) -> 1) (_ -> 0)))",
        "(data wrap (on x))\n(val main (match (on 1) ((on y) -> y) (_ -> 0)))",
        // cast / as-result / handler / with
        "(val main (try-cast 1 int))",
        "(val main (check-cast 1 int))",
        "(val main (as-result (fn () 1)))",
        "(val main (handler ask (fn (m k) (k m))))",
        "(val main (with (handler ask (fn (m k) (k m))) (perform ask 1)))",
        // let / letrec / local / var / set / if
        "(val main (let ((x 1)) x))",
        "(val main (let x 1))",
        "(val main (let (x) 1))",
        "(val main (let ((1 2)) 1))",
        "(val main (letrec ((f (fn (x) (f x)))) (f 1)))",
        "(val main (letrec ((f 1)) f))",
        "(val main (local (val x 1) x))",
        "(val main (local x 1))",
        "(val main (local (1) 1))",
        "(val main (local (type t int) (val t 1) t))",
        "(val main (local (type (t) int) 1))",
        "(val main (local (var x 1) (set x 2) x))",
        "(val main (local (var if 1) 1))",
        "(val main (local (rec (val f (fn (x) x))) (f 1)))",
        "(val main (rec (val f (fn (x) x)) (f 1)))",
        "(val main (rec (val f (fn (x) x))))",
        "(val main (rec (type t int) (val f (fn (x) x)) (f 1)))",
        "(val main (rec (val f 1) 1))",
        "(val main (rec 1))",
        "(val main (if true 1 0))",
        "(val main (set x 1))",
        "(val main (var x 1 x))",
        // path / operator idents
        "(val main +)",
        "(val main (fn (x) x))",
        "(val (+) 1)\n(val main +)",
    ];
    for src in cases {
        tip(src);
    }
}
