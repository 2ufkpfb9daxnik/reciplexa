//! Round-4 core: elaborate Err sweep + remaining check/cast/unify arms.

use reciplexa_core::cast::{
    decide_subtype, is_runtime_checkable, is_subtype, plan_cast_evidence, DecideResult,
};
use reciplexa_core::check::{infer_expr, infer_with_effects, typecheck_language_source, TypeEnv};
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::expr::{
    first_unreachable_arm, CoreExpr, CoreLiteral, CorePattern, MatchArm,
};
use reciplexa_core::ty::{CoreType, EffectRow};
use reciplexa_core::unify::{unify, Subst};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn elaborate_err_sweep_top_and_data() {
    for src in [
        "",
        "(page a4)",
        "(type orphan int)",
        "(val x 1)(val x 2)",
        "(rec (val main 1) (val main 2))",
        "(rec (val main 1))",
        "(data)",
        "(data onlyname)",
        "(data 1 (c))",
        "(data t)",
        "(data t (()))",
        "(data t (1))",
        "(data t ((1)))",
        "(data t 1)",
        "(data t foo/bar)",
        "(data t [ctor])",
        "(data t ((a type)))",
        "(data t (( )) (c))",
        "(data t ((a kind)) (c))",
        "(data t ((a type)(a type)) (c))",
        "(rec (data a (x)) (val y 1))",
        "(rec)",
        "(rec (data a))",
        "(type-alias)",
        "(type-alias t)",
        "(type-alias t int extra)",
        "(type t)",
        "(type 1 int)",
        "(type t bad)",
        "(val main (fn int))",
        "true",
        "1",
        "\"hi\"",
        "; just comment\n",
        "(val main 1)\n(type leftover int)",
    ] {
        let _ = elaborate_source(src);
    }
}

#[test]
fn elaborate_err_sweep_types_and_expr_forms() {
    for src in [
        "(val main (as (not) 1))",
        "(val main (as (diff int) 1))",
        "(val main (as (effects io) 1))",
        "(val main (as foo/bar 1))",
        "(val main (as (fn) 1))",
        "(val main (as (fn int) 1))",
        "(val main (as (fn int int (effects)) 1))",
        "(val main (as (fn int int (effects io)) 1))",
        "(val main (as (record (1 int)) 1))",
        "(val main (as (record (optional a)) 1))",
        "(val main (as (record (row)) 1))",
        "(val main (as (union) 1))",
        "(val main (as (intersect) 1))",
        "(type t (singleton))",
        "(type t (singleton \"x\"))\n(val main 1)",
        "(val main (record-update))",
        "(val main (record-update 1))",
        "(val main (record-extend 1))",
        "(val main (field))",
        "(val main (field 1))",
        "(val main (try-cast))",
        "(val main (try-cast 1))",
        "(val main (check-cast 1))",
        "(val main (cast))",
        "(val main (as))",
        "(val main (local))",
        "(val main (local (val x 1)))",
        "(val main (set))",
        "(val main (set x))",
        "(val main (if))",
        "(val main (if true))",
        "(val main (if true 1))",
        "(val main (match))",
        "(val main (match 1))",
        "(val main (match 1 (-> 1)))",
        "(val main (match 1 (true)))",
        "(val main (let))",
        "(val main (let ()))",
        "(val main (let ((x)) x))",
        "(val main (letrec))",
        "(val main (letrec ((f 1)) f))",
        "(val main (var))",
        "(val main (var x))",
        "(val main (var x 1))",
        "(val main (fn))",
        "(val main (fn (x)))",
        "(val main (bytes))",
        "(val main (bytes 1))",
        "(val main (list))",
        "(val main (tuple))",
        "(val main (raise))",
        "(val main (raise 1 2))",
        "(val main (or-raise))",
        "(val main (or-raise 1 2))",
        "(val main (as-result))",
        "(val main (as-result 1))",
        "(val main (as-result (fn (x) x)))",
        "(val main (forward))",
        "(val main (forward k extra))",
        "(val main (handler))",
        "(val main (handler log))",
        "(val main (with))",
        "(val main (with h))",
        "(val main (perform))",
        "(val main (perform log))",
        "(val main (handle))",
        "(val main (handle log))",
        "(val main (handle log (fn (m) m)))",
        "(fn if (x) x)",
        "(val if 1)",
        "(var top 1)",
        "(val (1 x) 1)",
        "(val main (unicode))",
        "(val main (match 1 ((bind) -> 0)))",
        "(val main (match 1 ((bind x y) -> 0)))",
        "(val main (match 1 (_ x -> 0)))",
        "(val main (match 1 (true x -> 0)))",
        "(val main (match 1 ((record) -> 0)))",
        "(val main (match 1 ((record a) -> 0)))",
        "(val main (match 1 ((tuple a) -> 0)))",
        "(data box (b (fn box int)))\n(val main 1)",
        "(rec (data nest (n (fn nest int))) (data wrap (w nest)))\n(val main 1)",
    ] {
        let _ = elaborate_source(src);
    }
}

#[test]
fn elaborate_happy_and_variance_surfaces() {
    let ok = elaborate_source(
        r#"
(data option ((a type)) none (some a))
(type id (forall ((a type)) (fn a a)))
(val id (fn (x) x))
(val main (some 1))
"#,
    );
    assert!(ok.is_ok(), "{ok:?}");

    let _ = elaborate_source(
        r#"
(data phantom ((a type)) (make))
(val main make)
"#,
    );

    let ok = elaborate_source(
        r#"
(rec (val f (fn () 1)) (val g (fn () 2)))
(val main (f))
"#,
    );
    assert!(ok.is_ok(), "{ok:?}");

    // Non-fn rec binding is an error.
    let err = elaborate_source("(rec (val main 1) (val other 2))");
    assert!(err.is_err());

    let ok = elaborate_source("1\n(val main 2)");
    assert!(ok.is_ok());

    let (_expr, data) = elaborate_with_data(
        r#"
(data color red green)
(val main red)
"#,
    )
    .unwrap();
    assert!(data.ctors.contains_key("red"));
}

#[test]
fn check_perform_forward_handle_record_edges() {
    let err = infer_expr(
        &CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("string"));

    let err = infer_expr(
        &CoreExpr::Perform {
            op: "custom".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("string") || err.message.contains("unit"));

    let mut env = TypeEnv::new();
    env.insert("k", CoreType::Int);
    let err = infer_expr(
        &CoreExpr::Forward {
            resume_name: "k".into(),
        },
        &env,
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("resume") || err.message.contains("forward"));

    let err = infer_expr(
        &CoreExpr::Forward {
            resume_name: "gone".into(),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("unbound"));

    let err = infer_expr(
        &CoreExpr::HandlerValue {
            op: "ask".into(),
            handler_params: vec![],
            handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("1 or 2") || err.message.contains("handler"));

    let ty = infer_expr(
        &CoreExpr::With {
            handler: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert_eq!(ty, CoreType::Int);

    let mut env = TypeEnv::new();
    env.data.type_aliases.insert(
        "f".into(),
        CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        },
    );
    let ty = infer_expr(
        &CoreExpr::LetRec {
            bindings: vec![(
                "f".into(),
                CoreExpr::Lambda {
                    params: vec!["x".into()],
                    body: Box::new(CoreExpr::Var("x".into())),
                },
            )],
            body: Box::new(CoreExpr::Var("f".into())),
        },
        &env,
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    let _ = ty;

    let err = infer_expr(
        &CoreExpr::Set {
            name: "missing".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &TypeEnv::new(),
        &mut Subst::new(),
        range(),
    )
    .unwrap_err();
    assert!(err.message.contains("unbound"));

    let mut env = TypeEnv::new();
    env.insert("x", CoreType::Int);
    let (ty, effs) = infer_with_effects(
        &CoreExpr::Set {
            name: "x".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(2))),
        },
        &env,
        &mut Subst::new(),
        range(),
    )
    .unwrap();
    assert_eq!(ty, CoreType::Unit);
    assert!(effs.ops.is_empty());
}

#[test]
fn check_record_match_cast_occurrence() {
    assert!(typecheck_language_source("(val main (field 1 a))").is_err());
    assert!(typecheck_language_source("(val main (record-update 1 (a 2)))").is_err());
    assert!(
        typecheck_language_source("(val main (record-update (record (a 1)) (b 2)))").is_err()
    );
    assert!(
        typecheck_language_source("(val main (record-extend (record (a 1)) (a 2)))").is_err()
    );
    assert!(typecheck_language_source(
        r#"(val main
  (field (record-extend (record (a 1)) (b 2)) b))"#,
    )
    .is_ok());

    let _ = typecheck_language_source(
        r#"
(data option (none) (some x))
(val main
  (match none
    (none -> 0)
    (some x -> x)
    (_ -> 1)))
"#,
    );

    let _ = typecheck_language_source(r#"(val main (fn (x) (if (number? x) (+ x 1) 0)))"#);
    for pred in ["string?", "bool?", "int?"] {
        let src = format!("(val main (fn (x) (if ({pred} x) x 0)))");
        let _ = typecheck_language_source(&src);
    }

    let _ = typecheck_language_source("(val main (try-cast 1 never))");
    let _ = typecheck_language_source("(val main (check-cast 1 never))");
    let _ = typecheck_language_source("(val main (as dynamic 1))");
}

#[test]
fn check_local_var_and_expansive() {
    let ty = typecheck_language_source(
        r#"(val main
  (var x 0
    (seq (set x 1) x)))"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::Int);

    let _ = typecheck_language_source(
        r#"
(type idle (forall ((a type)) a))
(val idle (perform log "x"))
(val main idle)
"#,
    );
}

#[test]
fn cast_unify_expr_helpers() {
    let _ = plan_cast_evidence(&CoreType::Int, &CoreType::F64);
    let _ = plan_cast_evidence(&CoreType::dyn_any(), &CoreType::String);
    let _ = plan_cast_evidence(
        &CoreType::Union(vec![CoreType::Int, CoreType::String]),
        &CoreType::Int,
    );
    let _ = plan_cast_evidence(
        &CoreType::Intersect(vec![CoreType::Int, CoreType::dyn_any()]),
        &CoreType::Int,
    );
    assert_eq!(
        decide_subtype(&CoreType::Never, &CoreType::Int),
        DecideResult::Proved
    );
    let _ = decide_subtype(&CoreType::Int, &CoreType::Never);
    let _ = is_subtype(&CoreType::Int, &CoreType::Number);
    let _ = is_runtime_checkable(&CoreType::Fun {
        args: vec![CoreType::Int],
        ret: Box::new(CoreType::Int),
        effects: EffectRow::default(),
    });
    assert!(!is_runtime_checkable(&CoreType::Not(Box::new(
        CoreType::Int
    ))));
    let _ = is_runtime_checkable(&CoreType::Diff(
        Box::new(CoreType::Any),
        Box::new(CoreType::Int),
    ));

    let mut subst = Subst::new();
    let v = CoreType::Var(subst.fresh_var());
    let _ = unify(
        &CoreType::OpenRecord {
            fields: vec![("a".into(), CoreType::Int)],
            row: Box::new(v),
        },
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        },
        &mut subst,
    );
    let mut subst = Subst::new();
    let _ = unify(
        &CoreType::Lacks {
            label: "a".into(),
            row: Box::new(CoreType::Record {
                fields: vec![("a".into(), CoreType::Int)],
            }),
        },
        &CoreType::Record {
            fields: vec![("b".into(), CoreType::Int)],
        },
        &mut subst,
    );
    let mut subst = Subst::new();
    let _ = unify(
        &CoreType::Not(Box::new(CoreType::Int)),
        &CoreType::Not(Box::new(CoreType::String)),
        &mut subst,
    );
    let mut subst = Subst::new();
    let id = subst.fresh_var();
    let _ = unify(
        &CoreType::Var(id),
        &CoreType::Fun {
            args: vec![CoreType::Var(id)],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        },
        &mut subst,
    );

    let arms = vec![
        MatchArm {
            pattern: CorePattern::Variant {
                tag: "none".into(),
                payload: None,
            },
            body: CoreExpr::Lit(CoreLiteral::Int(0)),
        },
        MatchArm {
            pattern: CorePattern::Variant {
                tag: "none".into(),
                payload: Some(Box::new(CorePattern::Wildcard)),
            },
            body: CoreExpr::Lit(CoreLiteral::Int(1)),
        },
    ];
    assert!(first_unreachable_arm(&arms, &["none", "some"]).is_some());

    let arms = vec![MatchArm {
        pattern: CorePattern::Variant {
            tag: "some".into(),
            payload: Some(Box::new(CorePattern::Variant {
                tag: "inner".into(),
                payload: None,
            })),
        },
        body: CoreExpr::Lit(CoreLiteral::Int(0)),
    }];
    assert!(first_unreachable_arm(&arms, &["some"]).is_none());
}

#[test]
fn typecheck_parameterized_adt() {
    let ty = typecheck_language_source(
        r#"
(data option ((a type)) none (some a))
(val main (some 1))
"#,
    );
    assert!(ty.is_ok(), "{ty:?}");

    assert!(typecheck_language_source(
        r#"
(data option ((a type)) none (some a))
(val main (none 1))
"#,
    )
    .is_err());

    assert!(typecheck_language_source(
        r#"
(data option ((a type)) none (some a))
(val main (some))
"#,
    )
    .is_err());
}
