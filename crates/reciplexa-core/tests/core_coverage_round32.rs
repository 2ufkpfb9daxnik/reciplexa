//! Round-32 core: more check/elaborate sources without in-file denom bloat.

use reciplexa_core::cast::{intersect_types, plan_cast_evidence, types_disjoint};
use reciplexa_core::check::{insert_implicit_casts, typecheck_language_source, TypeEnv};
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::{CoreType, EffectRow, SingletonValue};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn core_round32_open_record_handle_letrec_casts() {
    // Open-record unify via ascription + field
    tip(
        r#"
(val main (fn (r)
  (let ((x (as (record (a int) (row rho)) r)))
    (+ (field x a) (field x b)))))
"#,
    );
    tip(
        r#"
(type Point (record (x int) (y int) (row r)))
(val main (fn (p)
  (field (as Point p) z)))
"#,
    );

    // Failure handle 1-param + ask 2-param
    tip("(val main (handle failure (fn (e) 0) (perform failure \"x\")))");
    tip("(val main (handle ask (fn (m k) (k 1)) (perform ask \"q\")))");
    tip("(val main (handle ask (fn (m) m) (perform ask \"q\")))");

    // Letrec annotated / local var expansive init
    tip("(val main (letrec ((f (fn (x) (+ x 1)))) (f 1)))");
    tip("(val main (local (var c (fn (x) x)) (c 1)))");
    tip("(val main (local (rec (val f (fn (x) (f x)))) 1))");

    // Match / if expansive
    tip("(val main (match (fn (x) x) (_ -> 1)))");
    tip("(val main (if (fn (x) true) 1 0))");

    // Cast / try / check nests
    tip("(val main (as number 1))");
    tip("(val main (try-cast 1 number))");
    tip("(val main (check-cast 1 int))");
    tip("(val main (as (dynamic any) 1))");
    tip("(val main (as (dynamic number) 1))");

    // Data payload / forall / row
    tip("(data box (mk (forall (a) a)))\n(val main 1)");
    tip("(type R (record (row r) (a int)))\n(val main 1)");
    tip("(type F (fn int (effects ask) int))\n(val main 1)");

    // Pattern / ErrorNode
    tip("(val main (match 1 (true -> 0) (_ -> 1)))");
    tip("(val main (match unit (unit -> 1) (_ -> 0)))");
    tip("(val main (match false (false -> 1) (_ -> 0)))");

    // insert_casts nested App/Let/If/Seq/Lambda/Casts
    let mut env = TypeEnv::new();
    env.insert(
        "f",
        CoreType::Fun {
            args: vec![CoreType::Number, CoreType::Number],
            ret: Box::new(CoreType::Number),
            effects: EffectRow::default(),
        },
    );
    env.insert("x", CoreType::Int);
    let expr = CoreExpr::Let {
        name: "a".into(),
        value: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("f".into())),
            args: vec![
                CoreExpr::Var("x".into()),
                CoreExpr::Lit(CoreLiteral::Int(2)),
            ],
        }),
        body: Box::new(CoreExpr::Seq(vec![
            CoreExpr::If {
                cond: Box::new(CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("number?".into())),
                    args: vec![CoreExpr::Var("a".into())],
                }),
                then_branch: Box::new(CoreExpr::Cast {
                    expr: Box::new(CoreExpr::Var("a".into())),
                    evidence: reciplexa_core::cast::CastEvidence::Identity,
                    target: CoreType::Number,
                    cast_id: 1,
                }),
                else_branch: Box::new(CoreExpr::TryCast {
                    expr: Box::new(CoreExpr::Var("a".into())),
                    target: CoreType::Int,
                    cast_id: 2,
                }),
            },
            CoreExpr::CheckCast {
                expr: Box::new(CoreExpr::Var("a".into())),
                target: CoreType::Number,
                cast_id: 3,
            },
            CoreExpr::Lambda {
                params: vec!["z".into()],
                body: Box::new(CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("f".into())),
                    args: vec![CoreExpr::Var("z".into()), CoreExpr::Var("a".into())],
                }),
            },
            CoreExpr::Match {
                scrutinee: Box::new(CoreExpr::Var("a".into())),
                arms: vec![MatchArm {
                    pattern: CorePattern::Wildcard,
                    body: CoreExpr::Var("a".into()),
                }],
            },
        ])),
    };
    let _ = insert_implicit_casts(&expr, &env);

    // Cast singleton leftovers
    let s = CoreType::Singleton(SingletonValue::Int(3));
    let _ = intersect_types(&s, &CoreType::Any);
    let _ = intersect_types(&CoreType::Any, &s);
    let _ = types_disjoint(&s, &CoreType::String);
    let _ = plan_cast_evidence(&s, &CoreType::F64);
}
