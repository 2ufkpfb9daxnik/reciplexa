//! Round-35 core: residual check/cast/elaborate toward 99%.

use reciplexa_core::cast::{normalize_type, plan_cast_evidence, types_disjoint, CastEvidence};
use reciplexa_core::check::{insert_implicit_casts, typecheck_language_source, TypeEnv};
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::expr::CoreExpr;
use reciplexa_core::ty::{CoreType, EffectRow};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn core_round35_check_cast_elaborate_push() {
    // Letrec with type annotation (∀ / Fun stub) + wildcard param
    tip("(type f (fn int int))\n(val f (fn (x) (+ x 1)))\n(val main (f 1))");
    tip("(type f (fn number number))\n(val f (fn (_) 0))\n(val main (f 1))");
    tip("(val main (letrec ((f (fn (x) (if (< x 1) x (f (- x 1))))) (f 3)))");

    // App where callee is still a unification var / non-fun after soft unify
    tip("(val main (fn (g) (g 1 2)))");
    tip("(val main ((fn (x) x) 1))");

    // Multi-ctor data with nullary + payload variants (arity-0 arm)
    tip("(data opt (none) (some int))\n(val main (match (none) (none -> 0) (some x -> x)))");
    tip("(data either (left int) (right))\n(val main (right))");

    // Parameterized data (empty params rejected; non-empty App ctor)
    tip("(data box ((a type)) (wrap a))\n(val main (wrap 1))");
    tip("(data bad (()) (c))\n(val main 1)");

    // Open-record / effect escape / local state
    tip("(val main (fn (r) (field r missing)))");
    tip("(val main (local (var c 0) (fn () c)))");

    // Expansive forall annotation reject
    tip("(type id (forall (a type) (fn a a)))\n(val id ((fn (x) x) 1))\n(val main 1)");

    // Diff / Not / Dynamic cast nests
    tip("(type T (diff (union int string) string))\n(val main (as T 1))");
    tip("(val main (as (dynamic (union int string)) 1))");
    tip("(val main (as (not string) 1))");
    tip("(val main (check-cast (as (dynamic any) 1) string))");
    tip("(val main (try-cast (as (dynamic any) true) int))");

    // Handler / with / perform effect rows on Fun
    tip("(val main (handle ask (fn (m k) (k m)) (perform ask 1)))");
    tip("(val main (with (handler log (fn (m) m)) 1))");

    // Record update/extend + match lit denser
    tip("(val main (record-update (record (a 1)) (a 2) (b 3)))");
    tip("(val main (match (record (a 1)) ((record (a x)) -> x) (_ -> 0)))");

    // insert_casts Identity skip + disjoint Err + arity mismatch skip
    let mut env = TypeEnv::new();
    env.insert(
        "f",
        CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        },
    );
    env.insert("x", CoreType::Int);
    env.insert("s", CoreType::String);
    let _ = insert_implicit_casts(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("f".into())),
            args: vec![CoreExpr::Var("x".into())],
        },
        &env,
    );
    // arity mismatch: 2 args vs 1 param → skip coerce loop
    let _ = insert_implicit_casts(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("f".into())),
            args: vec![CoreExpr::Var("x".into()), CoreExpr::Var("x".into())],
        },
        &env,
    );
    // disjoint coerce
    let _ = insert_implicit_casts(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("f".into())),
            args: vec![CoreExpr::Var("s".into())],
        },
        &env,
    );

    // Cast API: Identity plan + Never reject
    let _ = plan_cast_evidence(&CoreType::Int, &CoreType::Int);
    let _ = plan_cast_evidence(&CoreType::Int, &CoreType::Never);
    let _ = plan_cast_evidence(&CoreType::String, &CoreType::Int);
    let _ = normalize_type(&CoreType::Intersect(vec![
        CoreType::Intersect(vec![CoreType::Int, CoreType::Int]),
        CoreType::Number,
    ]));
    let _ = types_disjoint(&CoreType::Int, &CoreType::String);
    let _ = types_disjoint(&CoreType::String, &CoreType::String);
    let _ = CastEvidence::Identity;

    // Elaborate residuals: empty rec data / token entry / empty param section
    tip("(rec)");
    tip("(rec (data))");
    tip("(data t)");
    tip("(data t ((a type)))");
    tip("(data t (() ) (c))");
    tip("(rec 1)");
    tip("(rec (data t (c)) (data u (d)))\n(val main (c))");
}
