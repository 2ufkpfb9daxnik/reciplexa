//! Round-18 core: large unique miss clusters in elaborate/check (rec/letrec/let
//! path binders, match payload/record patterns, forall, perform/handle, var).

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn elaborate_round18_rec_expression_error_matrix() {
    let cases = [
        // elaborate_rec: too short / result is decl / empty after type-only
        "(val main (rec f))",
        "(val main (rec (val f (fn (x) x))))",
        "(val main (rec (type t int) 1))",
        // parse_rec_val_bindings: non-list entry, bad binder, reserved, duplicate
        "(val main (rec 1 1))",
        "(val main (rec foo/bar 1))",
        "(val main (rec (1 2) 1))",
        "(val main (rec (val) 1))",
        "(val main (rec (val (f) (fn (x) x)) 1))",
        "(val main (rec (val if (fn (x) x)) 1))",
        "(val main (rec (val f (fn (x) x)) (val f (fn (y) y)) 1))",
        "(val main (rec (val f 1) 1))",
        // annotation without matching val (pending_annotations)
        "(val main (rec (type g int) (val f (fn (x) x)) 1))",
        // happy-ish rec expression (hit success path too)
        "(val main (rec (val f (fn (x) (f x))) f))",
        "(val main (rec (type f (fn int int)) (val f (fn (x) x)) f))",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round18_let_letrec_var_path_and_token_bindings() {
    let cases = [
        // let: binding atom is Token / Path (not a pair list)
        "(val main (let (x) 1))",
        "(val main (let (foo/bar) 1))",
        "(val main (let ((foo/bar 1)) 1))",
        "(val main (let ((1 1)) 1))",
        // letrec: Token / Path binding slots + empty / non-fn
        "(val main (letrec (f) f))",
        "(val main (letrec (foo/bar) 1))",
        "(val main (letrec ((foo/bar (fn (x) x))) 1))",
        "(val main (letrec (() (fn (x) x)) 1))",
        "(val main (letrec ((f)) f))",
        // var: name must be identifier
        "(val main (var))",
        "(val main (var 1 0 1))",
        "(val main (var (x) 0 1))",
        "(val main (var foo/bar 0 1))",
        "(val main (var x 0 x))",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round18_match_payload_record_and_perform() {
    let cases = [
        // payload pattern: bad token / path / nested non-list
        "(data box (mk x))\n(val main (match (mk 1) ((mk 1.5) -> 0) (_ -> 1)))",
        "(data box (mk x))\n(val main (match (mk 1) ((mk foo/bar) -> 0) (_ -> 1)))",
        // record patterns: optional decompose, arity, non-ident label
        "(val main (match (record (a 1)) ((record (optional a x)) -> 0) (_ -> 1)))",
        "(val main (match (record (a 1)) ((record (a)) -> 0) (_ -> 1)))",
        "(val main (match (record (a 1)) ((record ((a) x)) -> 0) (_ -> 1)))",
        "(val main (match (record (a 1)) ((record (1 x)) -> 0) (_ -> 1)))",
        // perform / handle / handler ops not identifiers
        "(val main (perform (op) 1))",
        "(val main (perform 1 1))",
        "(val main (perform foo/bar 1))",
        "(val main (handle (op) (fn (x) x) 1))",
        "(val main (handle 1 (fn (x) x) 1))",
        "(val main (handler (op) (fn (x) x)))",
        "(val main (handler 1 (fn (x) x)))",
        // bind pattern binder not ident
        "(val main (match 1 ((bind (x)) -> 0) (_ -> 1)))",
        "(val main (match 1 ((bind foo/bar) -> 0) (_ -> 1)))",
        // unicode non-number
        "(val main (unicode \"hi\"))",
        "(val main (unicode true))",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round18_forall_data_params_record_local_type() {
    let cases = [
        // forall binder list / pair shape errors
        "(type t (forall bad int))\n(val main 1)",
        "(type t (forall (x) int))\n(val main 1)",
        "(type t (forall (x type) int))\n(val main 1)",
        "(type t (forall ((x)) int))\n(val main 1)",
        "(type t (forall (((x) type)) int))\n(val main 1)",
        "(type t (forall ((x (type))) int))\n(val main 1)",
        "(type t (forall ((foo/bar type)) int))\n(val main 1)",
        // data type params
        "(data t foo/bar (c))\n(val main 1)",
        "(data t (a) (c))\n(val main 1)",
        "(data t ((a)) (c))\n(val main 1)",
        "(data t (((a) type)) (c))\n(val main 1)",
        // record / record-extend / record-update field not pair
        "(val main (record a))",
        "(val main (record foo/bar))",
        "(val main (record-extend (record (a 1)) b))",
        "(val main (record-update (record (a 1)) b))",
        // local type name not ident / orphan annotation
        "(val main (local (type (t) int) 1))",
        "(val main (local (type foo/bar int) (val x 1) 1))",
        "(val main (local (type t int) 1))",
        // polarity / data with fn payload (walk_param_polarity fn branch)
        "(data box ((a type)) (mk (fn a a)))\n(val main 1)",
        "(data box ((a type)) (mk a))\n(val main (mk 1))",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn check_round18_numeric_intersect_let_principle_and_casts() {
    let cases = [
        // Let principle-type early return when body is binder
        "(val main (let ((x 1)) x))",
        "(val main (let ((f (fn (x) x))) f))",
        "(val nested (let ((x 1)) x))\n(val main nested)",
        // number? refine then numeric binop (Intersect + Number)
        "(val main (let ((x (dynamic 1))) (if (number? x) (+ x 1) 0)))",
        "(val main (let ((x (dynamic 1.5))) (if (number? x) (* x 2.0) 0.0)))",
        "(val main (let ((x (dynamic 1))) (if (number? x) (< x 2) false)))",
        // union numeric class
        "(type n (union int f64))\n(val main (fn (x) (+ (as n x) 1)))",
        // variant match payload binding edges
        "(data opt (none) (some x))\n(val main (match none (none y -> 0) (some z -> z)))",
        "(data opt (none) (some x))\n(val main (match (some 1) (none -> 0) (some (bind w) -> w)))",
        "(val main (match 1 (foo x -> x) (_ -> 0)))",
        // if Never branch preference
        "(val main (if true 1 (raise \"x\")))",
        "(val main (if false (raise \"x\") 1))",
        // cast / try-cast / check-cast residual
        "(val main (as string 1))",
        "(val main (try-cast 1 string))",
        "(val main (check-cast 1 int))",
        "(val main (as (union int string) 1))",
        "(val main (as (intersect int string) 1))",
        // open record / multi-arg app
        "(val main ((fn (a b) (+ a b)) 1 2))",
        "(val main (record-extend (record (a 1)) (b 2) (c 3)))",
    ];
    for src in cases {
        tip(src);
    }
}
