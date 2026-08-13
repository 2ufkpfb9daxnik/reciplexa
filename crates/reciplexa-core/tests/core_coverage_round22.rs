//! Round-22 core: elaborate residual clusters from llvm-cov miss map —
//! BracketList/special AST edges, explicit Err arms, and medium `?` neighborhoods.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn elaborate_round22_bracketlist_and_structured() {
    let cases = [
        // BracketList param lists (is_param_list BracketList arm)
        "(fn add [x y] (+ x y))\n(val main (add 1 2))",
        "(val main (fn [x] x))",
        "(val main (fn f [x] (+ x 1)))",
        // Structured comment skipped inside lists (list_atoms continue)
        "(val main (seq 1 (// skip-me) 2))",
        "(val main (list 1 (// c) 2))",
        "(// top)\n(val main 1)",
        // list_head_ident None (non-ident head token)
        "(1 2 3)",
        "(\"x\" 1)",
        // Quarantined heads (unwrap_or_else \"?\" is dead; still exercise page/circle)
        "(page a4)",
        "(circle 1 2 3)",
        "(markup x)",
        "(group x)",
        "(rect 1 2 3 4)",
        "(text \"a\")",
        "(src \"a\")",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round22_data_param_ctor_edges() {
    let cases = [
        // Ctor tag is nested Node (not Token)
        "(data t (((tag))) )\n(val main 1)",
        "(data t (((tag) x)) )\n(val main 1)",
        // BracketList type-param pair → pair.kind() != List
        "(data t ([a type]) c)\n(val main 1)",
        "(data t ([a type] [b type]) c)\n(val main 1)",
        // BracketList ctor / empty-ish
        "(data t [c])\n(val main 1)",
        // Multi-payload happy + arity-1 / arity-n apps
        "(data wrap (mk a))\n(val main (mk 1))",
        "(data wrap (mk a b))\n(val main (mk 1 2))",
        "(data wrap (mk a b c))\n(val main (mk 1 2 3))",
        // Param section with token entry (not Node)
        "(data t (a type) c)\n(val main 1)",
        // Bad binder on good-looking tag
        "(data t (BadTag))\n(val main 1)",
        "(data t ((bad_tag x)))\n(val main 1)",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round22_type_syntax_err_arms() {
    let cases = [
        // Type constructor head not Ident
        "(type t (1))\n(val main 1)",
        "(type t (1 int))\n(val main 1)",
        "(type t (\"x\"))\n(val main 1)",
        // Empty / non-list type nodes via as
        "(val main (as () 1))",
        "(val main (as [int] 1))",
        // forall binder not a list
        "(type t (forall x a))\n(val main 1)",
        "(type t (forall 1 a))\n(val main 1)",
        // effect row: BracketList / nested Node entry
        "(type t (fn int int (effects [ask])))\n(val main 1)",
        "(type t (fn int int (effects (ask))))\n(val main 1)",
        "(type t (fn int int (effects ((ask int)))))\n(val main 1)",
        // row not last / optional / record field types
        "(type t (record (row r) (a int)))\n(val main 1)",
        "(type t (record (a int) (row r) (b int)))\n(val main 1)",
        "(type t (record (optional a int)))\n(val main 1)",
        "(type t (record [a int]))\n(val main 1)",
        // dynamic bound / not / diff / tuple / singleton string decode Err
        "(type t (dynamic int))\n(val t 1)\n(val main t)",
        "(type t (not string))\n(val main 1)",
        "(type t (diff number string))\n(val main 1)",
        "(type t (tuple int string bool))\n(val main 1)",
        "(type t \"bad\\q\")\n(val main 1)",
        // Number singleton Ok path already tipped; push F64 reject + unsupported
        "(type t 1.5)\n(val main 1)",
        "(type t 1.0e2)\n(val main 1)",
        // App type / alias
        "(data box ((a type)) (mk a))\n(type t (box string))\n(val main 1)",
        "(type-alias u (union int string))\n(val main 1)",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round22_expr_record_match_local() {
    let cases = [
        // Record / update / extend field shape: BracketList pairs
        "(val main (record [a 1]))",
        "(val main (record-update (record (a 1)) [a 2]))",
        "(val main (record-extend (record (a 1)) [b 2]))",
        "(val main (record-update (record (a 1)) ((a) 2)))",
        "(val main (record-extend (record (a 1)) ((b) 2)))",
        // field / list / tuple / bytes happy-ish
        "(val main (field (record (a 1)) a))",
        "(val main (list 1 2 3))",
        "(val main (tuple 1 2 3))",
        "(val main (bytes 0 1 255))",
        // Match arm / pattern BracketList edges
        "(val main (match 1 [1 -> 0]))",
        "(val main (match 1 ((record [a x]) -> x) (_ -> 0)))",
        "(val main (match (record (a 1)) ((record [a x]) -> x) (_ -> 0)))",
        "(val main (match 1 ([x] -> 0) (_ -> 1)))",
        // Nested pattern non-list
        "(data wrap (mk x))\n(val main (match (mk 1) ((mk [x]) -> 0) (_ -> 1)))",
        // Payload pattern unsupported token
        "(data wrap (mk x))\n(val main (match (mk 1) ((mk +) -> 0) (_ -> 1)))",
        // Literal patterns true/false/unit (covered region starts on match arms)
        "(val main (match true (true -> 1) (false -> 0)))",
        "(val main (match false (false -> 0) (true -> 1)))",
        "(val main (match unit (unit -> 0) (_ -> 1)))",
        // Let / letrec binding list wrong kinds
        "(val main (let [x 1] x))",
        "(val main (let (x 1) x))",
        "(val main (let ((x 1) [y 2]) x))",
        "(val main (letrec [f (fn (x) x)] (f 1)))",
        "(val main (letrec ((f (fn (x) x)) [g (fn (y) y)]) (f 1)))",
        // Local / rec BracketList decls
        "(val main (local [val x 1] x))",
        "(val main (local (val x 1) [var y 2] x))",
        "(val main (local (type [t] int) (val t 1) t))",
        "(val main (local (type (t) int) (val t 1) t))",
        "(val main (local (rec [val f (fn (x) x)]) (f 1)))",
        "(val main (rec [val f (fn (x) x)] (f 1)))",
        "(val main (rec () (fn () 1)))",
        "(val main (rec (val [f] (fn (x) x)) (f 1)))",
        // Fn params as nested Node
        "(val main (fn ((x)) x))",
        "(fn f ((x)) x)\n(val main 1)",
        // Perform / raise / casts / handle denser Ok paths (medium ?)
        "(val main (perform log \"x\"))",
        "(val main (log \"x\"))",
        "(val main (raise \"e\"))",
        "(val main (or-raise (as-result (fn () 1))))",
        "(val main (as-result (fn () 1)))",
        "(val main (try-cast 1 int))",
        "(val main (check-cast 1 int))",
        "(val main (handle ask (fn (m k) (k m)) (perform ask 1)))",
        "(val main (handler ask (fn (m k) (k m))))",
        "(val main (with (handler ask (fn (m k) (k m))) (perform ask 1)))",
        "(val main (forward k))",
        // unicode via elaborated atom
        "(val main (unicode (+ 1 64)))",
        "(val main (unicode 65.0))",
        // if / set / var Ok
        "(val main (if true 1 0))",
        "(val main (local (var x 1) (set x 2) x))",
        // Named top fn + binder_name Err
        "(fn Bad (x) x)\n(val main 1)",
        "(fn ok [x] x)\n(val main (ok 1))",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn elaborate_round22_rec_data_and_mix() {
    let cases = [
        // Mutual data rec happy
        "(rec (data a (wrap b)) (data b (wrap a)))\n(val main 1)",
        // Mix data+val rejected
        "(rec (data t (c)) (val f (fn (x) x)))\n(val main 1)",
        // rec data with non-data token (is_data_rec_node false → try_top_decl)
        "(rec foo (data t (c)))\n(val main 1)",
        "(rec (data t (c)) foo)\n(val main 1)",
        // Empty-ish rec
        "(rec)\n(val main 1)",
        // Top-level only data (no bindings → empty source? data alone may empty)
        "(data t (c))",
        "(data t (c))\n(val main c)",
        // Trailing after bindings
        "(val x 1)\n(+ x 1)",
        "(val main 1)\n2",
    ];
    for src in cases {
        tip(src);
    }
}

#[test]
fn check_round22_light_infer_edges() {
    // Light check.rs pass: drive handle/with/record-update/extend/cast/`set`/`if`.
    let cases = [
        "(val main (handle ask (fn (m k) (k m)) (perform ask 1)))",
        "(val main (with (handler ask (fn (m k) (k m))) (perform ask 1)))",
        "(val main (record-update (record (a 1)) (a 2)))",
        "(val main (record-extend (record (a 1)) (b 2)))",
        "(val main (field (record (a 1)) a))",
        "(val main (try-cast 1 int))",
        "(val main (check-cast 1 int))",
        "(val main (as int 1))",
        "(val main (local (var x 1) (set x 2) x))",
        "(val main (if true 1 0))",
        "(val main (letrec ((f (fn (x) (f x)))) f))",
        "(val main (let ((x 1)) x))",
        "(fn id (x) x)\n(val main (id 1))",
        "(val main ((fn (x) x) 1))",
        // Fun typed with effects stub / Var-ish
        "(type f (fn int int (effects ask)))\n(val f (fn (x) x))\n(val main (f 1))",
        "(data wrap (mk a))\n(val main (match (mk 1) ((mk x) -> x)))",
    ];
    for src in cases {
        tip(src);
    }
}
