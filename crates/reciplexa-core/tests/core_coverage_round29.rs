//! Round-29: nested elaborate `?` Err using forms that actually fail in
//! expression position (`val`/`data`/`1e9999`/`unterminated`), not quarantined
//! heads (those only reject at top-level).

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn elaborate_round29_expr_position_err_nests() {
    let bad_forms = [
        "(val x 1)",
        "(data t (c))",
        "(type t int)",
        "1e9999",
        "170141183460469231731687303715884105728",
        "\"unterminated\n",
        "(",
        ")",
    ];
    let templates = [
        "(val main (record (a {b})))",
        "(val main (record-update (record (a 1)) (a {b})))",
        "(val main (record-extend (record (a 1)) (b {b})))",
        "(val main (field {b} a))",
        "(val main (record-update {b} (a 1)))",
        "(val main (record-extend {b} (b 1)))",
        "(val main (list {b}))",
        "(val main (list 1 {b}))",
        "(val main (seq {b}))",
        "(val main (seq 1 {b}))",
        "(val main (tuple {b} 1))",
        "(val main (tuple 1 {b}))",
        "(val main (match {b} (_ -> 0)))",
        "(val main (match 1 (_ -> {b})))",
        "(val main (match 1 (1 -> {b}) (_ -> 0)))",
        "(val main (perform ask {b}))",
        "(val main (raise {b}))",
        "(val main (or-raise {b}))",
        "(val main (as-result {b}))",
        "(val main (try-cast {b} int))",
        "(val main (check-cast {b} int))",
        "(val main (as int {b}))",
        "(val main (if {b} 1 0))",
        "(val main (if true {b} 0))",
        "(val main (if true 1 {b}))",
        "(val main (set x {b}))",
        "(val main (let ((x {b})) x))",
        "(val main (let ((x 1)) {b}))",
        "(val main (local (val x {b}) x))",
        "(val main (local (val x 1) {b}))",
        "(val main (handle ask (fn (m k) (k m)) {b}))",
        "(val main (with (handler ask (fn (m k) (k m))) {b}))",
        "(val main (log {b}))",
        "(val main (unicode {b}))",
        "(val main (bytes {b}))",
    ];
    for b in bad_forms {
        for t in templates {
            tip(&t.replace("{b}", b));
        }
        // ctor arity nests
        tip(&format!(
            "(data t (b int))\n(val main (b {b}))"
        ));
        tip(&format!(
            "(data t (c int string))\n(val main (c 1 {b}))"
        ));
        tip(&format!(
            "(data t (c int string))\n(val main (c {b} \"x\"))"
        ));
    }
}

#[test]
fn elaborate_round29_pattern_and_rec_local_err() {
    for src in [
        // pattern payload Expr-like Err via nested list that's not a valid pattern
        "(data w (mk int))\n(val main (match (mk 1) ((mk (val x 1)) -> 0) (_ -> 1)))",
        "(val main (match (tuple 1 2) ((tuple (val x 1) b) -> 0) (_ -> 1)))",
        "(val main (match (record (a 1)) ((record (a (val x 1))) -> 0) (_ -> 1)))",
        // sole-pattern String decode / Number overflow
        "(val main (match 1 (\"unterminated\n -> 0) (_ -> 1)))",
        "(val main (match 1 (1e9999 -> 0) (_ -> 1)))",
        // rec / local / letrec nests
        "(val main (rec (val f (fn (x) (val y 1))) (f 1)))",
        "(val main (letrec ((f (fn (x) (data t (c))))) (f 1)))",
        "(val main (local (type t int) (val t (val x 1)) t))",
        // handle handler clause Err
        "(val main (handle ask (val x 1) 1))",
        "(val main (handle ask (fn (m k) (val x 1)) (perform ask 1)))",
        "(val main (with (val x 1) 1))",
        // as type atom Err
        "(val main (as 1e9999 1))",
        "(val main (as \"unterminated\n 1))",
        "(val main (as (val x 1) 1))",
        // row last Err already; empty data param via direct still
        "(data t (()))\n(val main 1)",
        "(data t ((1)))\n(val main 1)",
        // ErrorNode as expr via incomplete
        "(val main (record (a )))",
        "(val main (seq ))",
        "(val main (list ))",
        // forward / ambient
        "(val main (forward))",
        "(val main (log))",
        "(val main (log 1 2))",
        "(val main (random 1))",
        // match incomplete arm body
        "(val main (match 1 (1 ->) (_ -> 0)))",
        "(val main (match 1 (() -> 0) (_ -> 1)))",
    ] {
        tip(src);
    }
}
