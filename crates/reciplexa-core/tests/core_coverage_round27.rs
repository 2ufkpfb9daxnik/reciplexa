//! Round-27 core: break elaborate plateau via Number/String tokens the lexer
//! still emits (`1e9999`, overflow ints, unterminated strings) plus pattern /
//! unicode / bytes / type residual matrix that hits `?` after kind guards.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn elaborate_round27_number_string_parse_err_after_kind() {
    // Lexer emits Number for non-finite / overflow spellings; parse_* Err follows.
    let huge = "170141183460469231731687303715884105728";
    let cases = [
        // unicode: numeric_literal_from_token `?` (f64 non-finite) + Int/F64 lit arms
        "(val main (unicode 65))",
        "(val main (unicode 65.0))",
        "(val main (unicode 1e9999))",
        "(val main (unicode 1.0e999))",
        // bytes: parse_number_literal Err after Number kind
        "(val main (bytes 1e9999))",
        "(val main (bytes 1.0e999))",
        // type/as singleton String decode Err — string must end at newline so `)`
        // is not swallowed into the token (short strings close at `\n`).
        "(type t \"unterminated\n)\n(val main 1)",
        "(val main (as \"unterminated\n 1))",
        "(val main (as \"\"\"\n 1))",
        "(type t (singleton \"unterminated\n))\n(val main 1)",
        // type singleton Number `?`
        "(type t 1e9999)\n(val main 1)",
        "(type t 65.0)\n(val main 1)",
        "(type t (singleton 1e9999))\n(val main 1)",
        // pattern_literal_token: overflow int Err + string decode Err + Ok lit
        &format!("(val main (match {huge} ({huge} -> 0) (_ -> 1)))"),
        "(val main (match 1 (\"unterminated\n -> 0) (_ -> 1)))",
        // sole-pattern String before `->` on the next line → decode Err (2265)
        "(val main (match 1 (\"unterminated\n-> 0) (_ -> 1)))",
        "(val main (match 1.5 (1.5 -> 0) (_ -> 1)))",
        "(val main (match 1e9999 (1e9999 -> 0) (_ -> 1)))",
        // expr-position string decode Err
        "(val main \"unterminated\n)",
        // bool/unit Ident lit arms (no longer shadowed by sole-atom early path)
        "(val main (match true (true -> 1) (false -> 0)))",
        "(val main (match false (false -> 0) (true -> 1)))",
        "(val main (match unit (unit -> 0) (_ -> 1)))",
        "(val main (match true (true x -> 0) (_ -> 1)))",
    ];
    for src in cases {
        tip(src);
    }
    // Overflow int as bare expr / bytes / unicode / singleton
    tip(&format!("(val main {huge})"));
    tip(&format!("(val main (unicode {huge}))"));
    tip(&format!("(val main (bytes {huge}))"));
    tip(&format!("(type t {huge})\n(val main 1)"));
    tip(&format!("(type t (singleton {huge}))\n(val main 1)"));
    tip(&format!("(val main (match 1 ({huge} -> 0) (_ -> 1)))"));
}

#[test]
fn elaborate_round27_expr_pattern_data_residual() {
    let cases = [
        // seq Ok + empty Err
        "(val main (seq 1))",
        "(val main (seq 1 2))",
        "(val main (seq))",
        // ctor arity 0/1/multi
        "(data t (a) (b int) (c int string))\n(val main (a))",
        "(data t (a) (b int) (c int string))\n(val main (b 1))",
        "(data t (a) (b int) (c int string))\n(val main (c 1 \"x\"))",
        // ambient log arity 1 Ok
        "(val main (perform log \"x\"))",
        "(val main (perform random))",
        // record / field / update / extend / list Ok nests
        "(val main (record (a 1) (b 2)))",
        "(val main (field (record (a 1)) a))",
        "(val main (record-update (record (a 1)) (a 2)))",
        "(val main (record-extend (record (a 1)) (b 2)))",
        "(val main (list 1 2))",
        // match arms Ok + nested payload
        "(val main (match 1 (1 -> 0) (_ -> 1)))",
        "(val main (match \"ok\" (\"ok\" -> 1) (_ -> 0)))",
        "(val main (match (tuple 1 2) ((tuple a b) -> a) (_ -> 0)))",
        "(val main (match (record (a 1)) ((record (a x)) -> x) (_ -> 0)))",
        "(data w (mk int))\n(val main (match (mk 1) ((mk x) -> x) (_ -> 0)))",
        "(data w (mk int string))\n(val main (match (mk 1 \"a\") ((mk x y) -> x) (_ -> 0)))",
        // payload pattern non-ident Err
        "(data w (mk int))\n(val main (match (mk 1) ((mk 1.5) -> 0) (_ -> 1)))",
        // perform / raise / or-raise / as-result / casts
        "(val main (perform ask \"q\"))",
        "(val main (raise \"e\"))",
        "(val main (or-raise (as-result (fn () 1))))",
        "(val main (as-result (fn () 1)))",
        "(val main (try-cast 1 int))",
        "(val main (check-cast 1 int))",
        "(val main (as int 1))",
        // handle / with
        "(val main (handle ask (fn (m k) (k m)) (perform ask 1)))",
        "(val main (with (handler ask (fn (m k) (k m))) 1))",
        // local / rec / var / set / if
        "(val main (local (val x 1) x))",
        "(val main (local (type t int) (val t 1) t))",
        "(val main (local (var x 1) (set x 2) x))",
        "(val main (local (rec (val f (fn (x) x))) (f 1)))",
        "(val main (rec (val f (fn (x) x)) (f 1)))",
        "(val main (var x 1 x))",
        "(val main (if true 1 0))",
        "(val main (set x 1))",
        // let / letrec
        "(val main (let ((x 1)) x))",
        "(val main (letrec ((f (fn (x) x))) (f 1)))",
        // data polarity / positivity / empty param section nearby
        "(data box ((a type)) (mk (fn a int)))\n(val main 1)",
        "(data box ((a type)) (mk ()))\n(val main 1)",
        "(data box ((a type)) (mk (1 a)))\n(val main 1)",
        "(data box ((a type)) (mk foo/bar))\n(val main 1)",
        // forall binder BracketList / row last Err
        "(type t (forall [a type] a))\n(val main 1)",
        "(type t (record (row r) (a int)))\n(val main 1)",
        // type head Node Err
        "((type) t int)\n(val main 1)",
        // val binder Node Err
        "(val (1) 2)",
        "(val ((x)) 1)",
        // named fn top decl close
        "(fn f (x) x)\n(val main (f 1))",
        // rec data token / empty
        "(rec)",
        "(rec foo)",
        "(rec (data t (c)) foo)\n(val main 1)",
        // quarantined
        "(page a4)",
        "(circle 1)",
        // Error recovery
        "(val main )",
        "(",
        ")",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn check_round27_open_record_match_cast() {
    let cases = [
        "(type r (record (a int) (row rho)))\n(val main (fn (x) (.: x a)))",
        "(type r (record (optional a int)))\n(val main (fn (x) (.: x a)))",
        "(data t (a) (b int))\n(val main (match (a) (a -> 1) ((b x) -> x)))",
        "(data t (a) (b))\n(val main (match (a) (a -> 1)))",
        "(val main (as (union int string) 1))",
        "(val main (check-cast (intersect int number) 1))",
        "(val main (try-cast (diff int string) 1))",
        "(val main (fn (x) (if (number? x) (+ x 1) 0)))",
        "(val main (fn (x) (if (string? x) x \"\")))",
        "(val main (var s 1 (fn () s)))",
        "(val main (handle ask (fn (m k) (k m)) (perform ask 1)))",
        "(val main (with (handler ask (fn (m k) (k m))) 1))",
        "(val main (letrec ((f (fn (x) (f x)))) f))",
        "(type F (fn int int))\n(val main (letrec ((f (fn (x) x))) (f 1)))",
        "(val main (record-update (record (a 1)) (b 2)))",
        "(data pair (mk int string))\n(val main (match (mk 1 \"x\") ((mk a b) -> a) (_ -> 0)))",
        "(data opt (none) (some int))\n(val main (match none ((some x) -> x) (none -> 0)))",
    ];
    for src in cases {
        tip(src);
    }
}
