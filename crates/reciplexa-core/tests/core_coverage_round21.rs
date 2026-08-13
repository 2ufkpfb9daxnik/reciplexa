//! Round-21 core: elaborate residual neighborhoods (variance/positivity, match
//! exhaustiveness, type/effect syntax, local/rec edges) + unify Any/Any and
//! same-label Lacks where the second `enforce_lacks` fails.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::ty::CoreType;
use reciplexa_core::unify::{unify, Subst};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn unify_any_any_and_lacks_second_enforce_err() {
    assert!(unify(&CoreType::Any, &CoreType::Any, &mut Subst::new()).is_ok());

    let lacks_ok = CoreType::Lacks {
        label: "a".into(),
        row: Box::new(CoreType::Unit),
    };
    let lacks_bad = CoreType::Lacks {
        label: "a".into(),
        row: Box::new(CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        }),
    };
    assert!(unify(&lacks_ok, &lacks_bad, &mut Subst::new()).is_err());
    assert!(unify(&lacks_bad, &lacks_ok, &mut Subst::new()).is_err());
}

#[test]
fn elaborate_round21_data_variance_positivity_rec() {
    let cases = [
        // Variance walks: phantom / covariant / contravariant / invariant
        "(data box ((a type)) (mk int))\n(val main 1)",
        "(data box ((a type)) (mk a))\n(val main 1)",
        "(data box ((a type)) (mk (fn a int)))\n(val main 1)",
        "(data box ((a type)) (mk (fn a a)))\n(val main 1)",
        "(data box ((a type)) (mk (list a)))\n(val main 1)",
        // walk_param_polarity: empty / non-ident heads
        "(data box ((a type)) (mk ()))\n(val main 1)",
        "(data box ((a type)) (mk (1 a)))\n(val main 1)",
        "(data box ((a type)) (mk (\"x\" a)))\n(val main 1)",
        // Strict positivity violation (group type in negative position)
        "(data t (mk (fn t int)))\n(val main 1)",
        "(rec (data a (mk (fn b int))) (data b (nk int)))\n(val main 1)",
        // Mutual data rec (happy)
        "(rec (data a (wrap b)) (data b (wrap a)))\n(val main 1)",
        "(rec (data a ((t type)) (mk t)) (data b (nk)))\n(val main 1)",
        // Path / bad ctor atoms
        "(data t a/b)\n(val main 1)",
        "(data t (c a/b))\n(val main 1)",
        // Duplicate type param
        "(data t ((a type) (a type)) (mk a))\n(val main 1)",
        // Payload type path / bad node
        "(data t (mk [1]))\n(val main 1)",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round21_type_effect_syntax() {
    let cases = [
        // not / diff / union / intersect / forall / tuple / record residuals
        "(type t (not int))\n(val t 1)\n(val main t)",
        "(type t (not))\n(val main 1)",
        "(type t (diff number int))\n(val t 1.0)\n(val main t)",
        "(type t (diff int))\n(val main 1)",
        "(type t (union int string))\n(val t 1)\n(val main t)",
        "(type t (union))\n(val t 1)\n(val main t)",
        "(type t (intersect int))\n(val t 1)\n(val main t)",
        "(type t (tuple))\n(val t unit)\n(val main t)",
        "(type t (tuple int))\n(val main 1)",
        "(type t (tuple int string))\n(val t (tuple 1 \"a\"))\n(val main t)",
        "(type t (forall ((a type) (b type)) a))\n(val t 1)\n(val main t)",
        "(type t (forall ((a type) (a type)) a))\n(val main 1)",
        "(type t (forall ((a row)) a))\n(val main 1)",
        "(type t (forall ((a type) (b effect-row)) a))\n(val t 1)\n(val main t)",
        "(type t (record (a int) (row r)))\n(val main 1)",
        "(type t (record (row r) (a int)))\n(val main 1)",
        "(type t (record (row)))\n(val main 1)",
        "(type t (record (optional a int) (b int)))\n(val t (record (b 1)))\n(val main t)",
        // effects on fn types
        "(type t (fn int int (effects log)))\n(val t (fn (x) x))\n(val main t)",
        "(type t (fn int int (effects (log string))))\n(val t (fn (x) x))\n(val main t)",
        "(type t (fn int int (effects (log string) (log string))))\n(val main 1)",
        "(type t (fn int int (effects (1))))\n(val main 1)",
        "(type t (fn int int (effects ())))\n(val main 1)",
        "(type t (effects log))\n(val main 1)",
        // dynamic / singleton / app / aliases
        "(type t (dynamic))\n(val t 1)\n(val main t)",
        "(type t (dynamic int string))\n(val main 1)",
        "(type t 1.5)\n(val main 1)",
        "(type-alias u int)\n(type-alias u string)\n(val main 1)",
        "(type t (box int))\n(val main 1)",
        "(data box ((a type)) (mk a))\n(type t (box int))\n(val t (mk 1))\n(val main t)",
        "(type t (fn))\n(val main 1)",
        "(type t (fn int int int))\n(val t (fn (x y) x))\n(val main t)",
        // unknown / path type atoms
        "(type t unknown-ty)\n(val main 1)",
        "(type t a/b)\n(val main 1)",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round21_match_patterns_exhaustiveness() {
    let cases = [
        // Non-exhaustive / unreachable
        "(data flag on off)\n(val main (match on (on -> 1)))",
        "(data flag on off)\n(val main (match on (on -> 1) (_ -> 0) (off -> 2)))",
        "(data flag on off)\n(val main (match on (on -> 1) (on -> 2) (off -> 0)))",
        "(data flag on off)\n(val main (match on (on -> 1) (off -> 0) (on -> 2)))",
        // optional field in record pattern
        "(val main (match (record (a 1)) ((record (optional a x)) -> x) (_ -> 0)))",
        // bind / literal arity / f64 lit reject
        "(val main (match 1 ((bind) -> 0) (_ -> 1)))",
        "(val main (match 1 ((bind x y) -> 0) (_ -> 1)))",
        "(val main (match 1 ((bind 1) -> 0) (_ -> 1)))",
        "(val main (match 1 (1.0 -> 0) (_ -> 1)))",
        "(val main (match 1 (1 2 -> 0) (_ -> 1)))",
        "(val main (match \"a\" (\"a\" \"b\" -> 0) (_ -> 1)))",
        "(val main (match true (true x -> 0) (_ -> 1)))",
        "(val main (match unit (unit x -> 0) (_ -> 1)))",
        "(val main (match 1 (_ x -> 0)))",
        // tuple / record / multi-payload pattern edges
        "(val main (match (tuple 1 2) ((tuple) -> 0) (_ -> 1)))",
        "(val main (match (tuple 1 2) ((tuple a) -> 0) (_ -> 1)))",
        "(val main (match (tuple 1 2) ((tuple a b c) -> 0) (_ -> 1)))",
        "(val main (match (record (a 1)) ((record) -> 0) (_ -> 1)))",
        "(val main (match (record (a 1)) ((record (a x) (a y)) -> 0) (_ -> 1)))",
        "(val main (match (record (a 1)) ((record (1 x)) -> 0) (_ -> 1)))",
        "(data wrap (mk a b))\n(val main (match (mk 1 2) ((mk x y) -> x) (_ -> 0)))",
        "(data wrap (mk a b))\n(val main (match (mk 1 2) ((mk x y z) -> x) (_ -> 0)))",
        // arm shape errors
        "(val main (match 1 (-> 0)))",
        "(val main (match 1 (1 ->)))",
        "(val main (match 1 (1 -> 0 1)))",
        "(val main (match 1 1))",
        // nested payload patterns
        "(data wrap (mk x))\n(val main (match (mk 1) ((mk _) -> 0) (_ -> 1)))",
        "(data wrap (mk x))\n(val main (match (mk 1) ((mk (bind y)) -> y) (_ -> 0)))",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round21_expr_local_rec_casts_ambient() {
    let cases = [
        // unicode / bytes / tuple residuals
        "(val main (unicode 65))",
        "(val main (unicode 0))",
        "(val main (unicode 1114112))",
        "(val main (unicode (tuple)))",
        "(val main (unicode))",
        "(val main (unicode 1 2))",
        "(val main (bytes 256))",
        "(val main (bytes -1))",
        "(val main (bytes 1.5))",
        "(val main (tuple 1))",
        // record-update / extend field shape
        "(val main (record-update (record (a 1))))",
        "(val main (record-extend (record (a 1))))",
        "(val main (record-update (record (a 1)) (a 2) (a 3)))",
        "(val main (record-extend (record (a 1)) (b 2) (b 3)))",
        "(val main (record (a 1) (a 2)))",
        // perform / forward / handle / handler / with
        "(val main (perform 1 2))",
        "(val main (perform log))",
        "(val main (forward))",
        "(val main (forward 1))",
        "(val main (forward k extra))",
        "(val main (handle log (fn () 1) 1))",
        "(val main (handle log (fn (m) m) (perform log 1)))",
        "(val main (handle 1 (fn (m) m) 1))",
        "(val main (handler log (fn () 1)))",
        "(val main (handler log (fn (m) m)))",
        "(val main (handler 1 (fn (m) m)))",
        "(val main (with (handler log (fn (m k) (k m))) (perform log 1) 2))",
        "(val main (with 1))",
        // try/check-cast incompatible
        "(val main (try-cast 1 never))",
        "(val main (check-cast 1 never))",
        "(val main (try-cast 1))",
        "(val main (check-cast 1))",
        // local / rec edges
        "(val main (local (fn x) 1))",
        "(val main (local (import x) 1))",
        "(val main (local (type t int) (val t 1) t))",
        "(val main (local (type-alias u int) (val x 1) x))",
        "(val main (local (type t int) 1))",
        "(val main (local (var x 1) (set x 2) x))",
        "(val main (local (var (x) 1) 1))",
        "(val main (local (val if 1) 1))",
        "(val main (local (var if 1) 1))",
        "(val main (local (rec (val f (fn (x) x))) (f 1)))",
        "(val main (local (rec) 1))",
        "(val main (rec (val f (fn (x) x))))",
        "(val main (rec (val f (fn (x) x)) (val g (fn (y) y))))",
        "(val main (rec (type t int) 1))",
        "(val main (rec (val f 1) (f)))",
        "(val main (rec 1 2))",
        "(val main (let ((x 1) (x 2)) x))",
        "(val main (letrec ((f (fn (x) x)) (f (fn (y) y))) (f 1)))",
        "(val main (letrec ((f 1)) f))",
        // top-level / quarantine / named fn
        "(page a4 (circle 1 2 3))",
        "(fn add (x y) (+ x y))\n(val main (add 1 2))",
        "(fn (x) x)\n(val main 1)",
        "(val (f 1) 1)",
        "(val (if x) 1)",
        "(val main (as-result (fn () 1)))",
        "(val main (seq))",
        "(val main (seq 1))",
        // field / set / if residuals
        "(val main (field (record (a 1))))",
        "(val main (if true 1))",
        "(val main (set))",
        "(val main (var x))",
    ];
    for src in cases {
        tip(src);
    }
}
