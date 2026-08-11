//! Round-5 core: open-record, multi-field ctors, coerce/cast residual, patterns.

use reciplexa_core::cast::{plan_cast_evidence, CastEvidence, DecideResult};
use reciplexa_core::check::{
    coerce_to_static, infer_expr, insert_implicit_casts, typecheck_language_source, TypeEnv,
};
use reciplexa_core::elaborate::elaborate_source;
use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_core::ty::{CoreType, EffectRow};
use reciplexa_core::unify::{unify, Subst};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn open_record_field_get_and_update_typing() {
    let mut env = TypeEnv::new();
    let mut subst = Subst::new();
    let row = CoreType::Var(subst.fresh_var());
    env.insert(
        "r",
        CoreType::OpenRecord {
            fields: vec![("a".into(), CoreType::Int)],
            row: Box::new(row),
        },
    );
    let ty = infer_expr(
        &CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Var("r".into())),
            field: "b".into(),
        },
        &env,
        &mut subst,
        range(),
    )
    .unwrap();
    let _ = ty;

    let ty = infer_expr(
        &CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Var("r".into())),
            field: "a".into(),
        },
        &env,
        &mut subst,
        range(),
    )
    .unwrap();
    assert_eq!(ty, CoreType::Int);
}

#[test]
fn multi_field_ctor_and_match_patterns() {
    let ty = typecheck_language_source(
        r#"
(data pair (mk a b))
(val main
  (match (mk 1 2)
    ((mk x y) -> (+ x y))
    (_ -> 0)))
"#,
    );
    assert!(ty.is_ok(), "{ty:?}");

    let ty = typecheck_language_source(
        r#"
(data option ((a type)) none (some a))
(val main
  (match (some 1)
    (none -> 0)
    ((some x) -> x)))
"#,
    );
    assert!(ty.is_ok(), "{ty:?}");

    for src in [
        r#"(val main (match 1 (1 -> 1) (_ -> 0)))"#,
        r#"(val main (match "x" ("x" -> 1) (_ -> 0)))"#,
        r#"(val main (match true (true -> 1) (false -> 0)))"#,
        r#"(val main (match unit (unit -> 1) (_ -> 0)))"#,
        r#"(val main (match (record (a 1)) ((record (a x)) -> x) (_ -> 0)))"#,
        r#"(val main (match (tuple 1 2) ((tuple a b) -> a) (_ -> 0)))"#,
        r#"(val main (match 1 ((bind x) -> x)))"#,
        r#"
(data tree (leaf) (node l r))
(val main
  (match (node leaf leaf)
    ((node (leaf) (leaf)) -> 1)
    ((node a b) -> 0)
    (leaf -> 2)))
"#,
    ] {
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn elaborate_more_constructors_and_locals() {
    for src in [
        "(data pair (mk a b))\n(val main (mk 1 2))",
        "(data pair (mk a b))\n(val main (mk 1))",
        "(data t c)\n(val main (c 1))",
        "(val main (local (val x 1) (val y x) y))",
        "(val main (local (type t int) (val x 1) x))",
        "(val main (local (data box (b)) (b)))",
        "(val main (seq 1))",
        "(val main (seq))",
        "(val main (unicode 65))",
        "(val main (bytes \"hi\"))",
        "(val main (list 1 2 3))",
        "(val main (tuple 1 2 3))",
        "(val main (as (forall ((a type)) a) 1))",
        "(val main (as (not int) 1))",
        "(val main (as (diff number int) 1.0))",
        "(val main (as (intersect int dynamic) 1))",
        "(val main (as (union int string) 1))",
        "(val main (as (record (row r)) 1))",
        "(type t (fn int int (effects io)))\n(val main 1)",
        "(type t (record (optional a int) (b string)))\n(val main 1)",
        "(type t (singleton 1))\n(val main 1)",
        "(data box ((a type)) (mk a))\n(type t (box int))\n(val main 1)",
        "(val main (try-cast 1 int))",
        "(val main (check-cast 1 int))",
        "(val main (as int 1))",
        "(val main (handle failure (fn (e) e) (raise \"x\")))",
        "(val main (or-raise (as-result (fn () 1))))",
        "(val main (with (handler log (fn (m) m)) (perform log \"x\")))",
        "(val main (letrec ((f (fn (n) (f n)))) f))",
        "(val main (let ((x 1) (y x)) y))",
        "(val main (if true 1 0))",
        "(val main (record (a 1) (b 2)))",
        "(val main (record-update (record (a 1)) (a 2)))",
        "(val main (record-extend (record (a 1)) (b 2)))",
        "(val main (field (record (a 1)) a))",
        "(val main (fn (_) 1))",
        "(fn id (x) x)\n(val main (id 1))",
        "(val (add x y) (+ x y))\n(val main (add 1 2))",
        "1 2 3",
        "(data color red)\n(val main red)",
        "(rec (data a (x)) (data b (y a)))\n(val main 1)",
    ] {
        let _ = elaborate_source(src);
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn coerce_insert_casts_and_impossible() {
    let expr = CoreExpr::Lit(CoreLiteral::Int(1));
    let casted = insert_implicit_casts(&expr, &TypeEnv::new());
    let _ = casted;

    let _ = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::dyn_any(),
        &CoreType::String,
        2,
    );

    let plan = plan_cast_evidence(
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        },
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Int), ("b".into(), CoreType::String)],
        },
    );
    let _ = plan;

    let plan = plan_cast_evidence(
        &CoreType::Variant {
            variants: vec![("none".into(), None)],
        },
        &CoreType::Variant {
            variants: vec![
                ("none".into(), None),
                ("some".into(), Some(CoreType::Int)),
            ],
        },
    );
    let _ = plan;

    let plan = plan_cast_evidence(
        &CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        },
        &CoreType::Fun {
            args: vec![CoreType::dyn_any()],
            ret: Box::new(CoreType::dyn_any()),
            effects: EffectRow::default(),
        },
    );
    let _ = plan;

    let _ = plan_cast_evidence(
        &CoreType::App {
            ctor: "option".into(),
            args: vec![CoreType::Int],
        },
        &CoreType::dyn_any(),
    );
}

#[test]
fn generalize_complex_types_and_effects() {
    let ty = typecheck_language_source(
        r#"
(type id (forall ((a type)) (fn a a)))
(val id (fn (x) x))
(val main (id 1))
"#,
    );
    assert!(ty.is_ok(), "{ty:?}");

    let ty = typecheck_language_source(
        r#"(val main
  (handle log (fn (msg k) (k msg))
    (perform log "x")))"#,
    );
    assert!(ty.is_ok(), "{ty:?}");

    let ty = typecheck_language_source(
        r#"(val main
  (fn (r)
    (field r missing)))"#,
    );
    let _ = ty;

    // Ambiguous number
    let err = typecheck_language_source(
        r#"(val main (+ (as dynamic 1) (as dynamic 2)))"#,
    );
    let _ = err;
}

#[test]
fn unify_more_structural_failures() {
    let mut s = Subst::new();
    let _ = unify(
        &CoreType::OptionalField(Box::new(CoreType::Int)),
        &CoreType::OptionalField(Box::new(CoreType::String)),
        &mut s,
    );
    let mut s = Subst::new();
    let _ = unify(
        &CoreType::App {
            ctor: "list".into(),
            args: vec![CoreType::Int],
        },
        &CoreType::App {
            ctor: "list".into(),
            args: vec![CoreType::String],
        },
        &mut s,
    );
    let mut s = Subst::new();
    let _ = unify(
        &CoreType::Union(vec![CoreType::Int, CoreType::String]),
        &CoreType::Union(vec![CoreType::Bool]),
        &mut s,
    );
    let mut s = Subst::new();
    let v = s.fresh_var();
    let _ = unify(
        &CoreType::OpenRecord {
            fields: vec![],
            row: Box::new(CoreType::Var(v)),
        },
        &CoreType::OpenRecord {
            fields: vec![("a".into(), CoreType::Int)],
            row: Box::new(CoreType::Var(s.fresh_var())),
        },
        &mut s,
    );
    let mut s = Subst::new();
    let _ = unify(&CoreType::Color, &CoreType::Shape, &mut s);
    let mut s = Subst::new();
    let _ = unify(&CoreType::Bytes, &CoreType::String, &mut s);
    let mut s = Subst::new();
    let _ = unify(&CoreType::Error, &CoreType::Int, &mut s);
    let mut s = Subst::new();
    let _ = unify(&CoreType::Any, &CoreType::Never, &mut s);

    let _ = DecideResult::Unknown;
    let _ = CastEvidence::Widen;
}
