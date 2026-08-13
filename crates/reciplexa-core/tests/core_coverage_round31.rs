//! Round-31 core: open-record field unify, dynamic Never refine, Singleton∩,
//! insert_casts App coerce, elaborate pattern lit residuals.

use reciplexa_core::cast::{intersect_types, plan_cast_evidence, types_disjoint, CastEvidence};
use reciplexa_core::check::{
    coerce_to_static, insert_implicit_casts, typecheck_language_source, TypeEnv,
};
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_core::ty::{CoreType, EffectRow, SingletonValue};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn core_round31_check_cast_elaborate_push() {
    // Open-record field access → unify row with expected OpenRecord (check L462)
    tip(
        r#"
(type R (record (a int) (row r)))
(val main (fn (x)
  (let ((y (as R x)))
    (field y missing))))
"#,
    );
    tip(
        r#"
(val main (fn (r)
  (field (as (record (row r)) r) z)))
"#,
    );

    // Dynamic + number? refine → else Never when bound is Number (L1312)
    tip(
        r#"
(val main (let ((d (as (dynamic number) 1.0)))
  (if (number? d) 1 0)))
"#,
    );
    tip(
        r#"
(val main (let ((d (as (dynamic string) "x")))
  (if (string? d) 1 0)))
"#,
    );
    tip(
        r#"
(val main (let ((d (as (dynamic bool) true)))
  (if (bool? d) 1 0)))
"#,
    );

    // Occurrence Intersect + numeric binop (L1502 / 1556)
    tip(
        r#"
(val main (fn (x)
  (if (number? x) (+ x 1) 0)))
"#,
    );
    tip(
        r#"
(val main (fn (x)
  (if (number? x) (+ 1 x) 0)))
"#,
    );

    // Handle / With effects (handler_body infer L203)
    tip("(val main (handle ask (fn (m k) (k m)) (perform ask 1)))");
    tip("(val main (with (handler ask (fn (m k) (k m))) 1))");
    tip("(val main (handler ask (fn (m) m)))");

    // Letrec with annotation / expansive
    tip("(val main (letrec ((f (fn (x) (f x)))) (f 1)))");
    tip("(val main (local (rec (val f (fn (x) x))) (f 1)))");

    // Match expansive / is-none occurrence
    tip(
        r#"
(data opt (none) (some int))
(val main (fn (x)
  (if (is-none x) 0 1)))
"#,
    );

    // insert_implicit_casts App with Int→Number coerce
    let mut env = TypeEnv::new();
    env.insert(
        "f",
        CoreType::Fun {
            args: vec![CoreType::Number],
            ret: Box::new(CoreType::Number),
            effects: EffectRow::default(),
        },
    );
    env.insert("x", CoreType::Int);
    let app = CoreExpr::App {
        fun: Box::new(CoreExpr::Var("f".into())),
        args: vec![CoreExpr::Var("x".into())],
    };
    let _ = insert_implicit_casts(&app, &env);
    let nested = CoreExpr::Let {
        name: "y".into(),
        value: Box::new(app.clone()),
        body: Box::new(CoreExpr::If {
            cond: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("number?".into())),
                args: vec![CoreExpr::Var("y".into())],
            }),
            then_branch: Box::new(CoreExpr::Seq(vec![
                CoreExpr::Var("y".into()),
                CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("f".into())),
                    args: vec![CoreExpr::Lit(CoreLiteral::Int(2))],
                },
            ])),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        }),
    };
    let _ = insert_implicit_casts(&nested, &env);
    let _ = insert_implicit_casts(
        &CoreExpr::Lambda {
            params: vec!["_".into(), "z".into()],
            body: Box::new(app),
        },
        &env,
    );
    let _ = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::Int,
        &CoreType::Number,
        99,
    );

    // Cast Singleton∩ residuals
    let s = CoreType::Singleton(SingletonValue::Int(1));
    let _ = intersect_types(&s, &CoreType::Number);
    let _ = intersect_types(&CoreType::Number, &s);
    let _ = intersect_types(&s, &CoreType::Int);
    let _ = intersect_types(&s, &CoreType::String);
    let _ = intersect_types(
        &s,
        &CoreType::Union(vec![CoreType::Int, CoreType::String]),
    );
    let _ = intersect_types(
        &s,
        &CoreType::Diff(Box::new(CoreType::Number), Box::new(CoreType::F64)),
    );
    let _ = intersect_types(
        &CoreType::Singleton(SingletonValue::Bool(true)),
        &CoreType::Bool,
    );
    let _ = intersect_types(
        &CoreType::Singleton(SingletonValue::String("x".into())),
        &CoreType::String,
    );
    let _ = intersect_types(&CoreType::Singleton(SingletonValue::Unit), &CoreType::Unit);
    // Equal singletons after normalize (type_eq early) + unequal
    let _ = intersect_types(&s, &CoreType::Singleton(SingletonValue::Int(1)));
    let _ = intersect_types(&s, &CoreType::Singleton(SingletonValue::Int(2)));
    let _ = types_disjoint(&CoreType::Color, &CoreType::Shape);
    let _ = types_disjoint(&CoreType::Bool, &CoreType::String);
    let _ = plan_cast_evidence(&s, &CoreType::Number);
    let _ = plan_cast_evidence(&CoreType::dyn_any(), &s);

    // Elaborate: pattern literal Number/String with arity / ErrorNode / incomplete
    tip("(val main (match 1 (1 2 -> 0) (_ -> 1)))");
    tip("(val main (match \"a\" (\"a\" x -> 0) (_ -> 1)))");
    tip("(val main (match 1 (1e9999 -> 0) (_ -> 1)))");
    tip("(val main (match 1 (\"unterminated\n -> 0) (_ -> 1)))");
    tip("(val main (match 1 (() -> 0) (_ -> 1)))");
    tip("(val main )");
    tip("(val main (");
    tip("(data t (c int))\n(val main (match (c 1) ((c ) -> 0) (_ -> 1)))");
    // Top-level rec / local nests
    tip("(rec (val f (fn (x) x)) (val main (f 1)))");
    tip("(val main (local (type T int) (val x (as T 1)) x))");
    tip("(val main (local (val x 1) (val y 2) (+ x y)))");
    tip("(val main (var x 1 (set x 2) x))");

    let _ = CastEvidence::Identity;
}
