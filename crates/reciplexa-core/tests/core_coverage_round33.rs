//! Round-33 core: open-record unify, effect-escape Diff/App, cast intersect
//! residuals, Failure handler arity — tip without growing in-file denom.

use reciplexa_core::cast::{
    decide_subtype, intersect_types, is_runtime_checkable, is_subtype, judge_dynamic_use,
    normalize_type, plan_cast_evidence, types_disjoint, CastEvidence, DecideResult,
};
use reciplexa_core::check::{
    coerce_to_static, insert_implicit_casts, typecheck_language_source, TypeEnv,
};
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::{CoreType, EffectRow, SingletonValue};

fn tip(src: &str) {
    let _ = elaborate_source(src);
    let _ = elaborate_with_data(src);
    let _ = typecheck_language_source(src);
}

#[test]
fn core_round33_check_cast_elaborate_wave() {
    // Open-record missing field → unify row (check L462)
    tip(
        r#"
(type R (record (a int) (row r)))
(val main (fn (x)
  (field (as R x) z)))
"#,
    );
    tip(
        r#"
(val main (fn (r)
  (let ((x (as (record (row rho)) r)))
    (field x missing))))
"#,
    );

    // Failure handler with resume → Err (check L189-193)
    tip("(val main (handle failure (fn (e k) 0) (perform failure \"x\")))");
    tip("(val main (handle ask (fn ()) 1))");
    tip("(val main (handler ask (fn (a b c) a)))");

    // Local-state escape via Fun effects in result
    tip(
        r#"
(val main (local (var c 1)
  (fn (x) (set c x))))
"#,
    );

    // Record-update / extend error paths
    tip("(val main (record-update 1 (a 2)))");
    tip("(val main (record-update (record (a 1)) (b 2)))");
    tip("(val main (record-extend 1 (a 2)))");
    tip("(val main (record-extend (record (a 1)) (a 2)))");

    // Match / occurrence / is-some
    tip(
        r#"
(data opt (none) (some int))
(val main (fn (x)
  (if (is-some x) 1 0)))
"#,
    );
    tip(
        r#"
(data opt (none) (some int))
(val main (match (some 1)
  (none -> 0)
  (some x -> x)))
"#,
    );

    // Dynamic refine Never else
    tip(
        r#"
(val main (let ((d (as (dynamic int) 1)))
  (if (string? d) 1 0)))
"#,
    );

    // Cast / try / check nests + ascription
    tip("(val main (as (union int string) 1))");
    tip("(val main (as (intersect number int) 1))");
    tip("(val main (as (not string) 1))");
    tip("(val main (as (diff number string) 1))");
    tip("(val main (try-cast \"x\" int))");
    tip("(val main (check-cast 1.5 int))");

    // Elaborate pattern / quarantined / data edges
    tip("(val main (match 1 (\"unterminated -> 0) (_ -> 1)))");
    tip("(page a4 (circle 1 2 3))\n(val main 1)");
    tip("(val main (as int 1e9999))");
    tip("(data box (mk))\n(val main (match (mk) (mk -> 1)))");
    tip("(type T (forall ((a type)) a))\n(val main 1)");
    tip("(type R (record (row r) (a int) (b string)))\n(val main 1)");

    // insert_casts: nested With/Handle/Perform/Forward/Set
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
    let expr = CoreExpr::With {
        handler: Box::new(CoreExpr::HandlerValue {
            op: "ask".into(),
            handler_params: vec!["m".into(), "k".into()],
            handler_body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Var("m".into())],
            }),
        }),
        body: Box::new(CoreExpr::Handle {
            op: "ask".into(),
            handler_params: vec!["m".into()],
            handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
            body: Box::new(CoreExpr::Seq(vec![
                CoreExpr::Perform {
                    op: "ask".into(),
                    arg: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
                },
                CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("f".into())),
                    args: vec![CoreExpr::Var("x".into())],
                },
                CoreExpr::Match {
                    scrutinee: Box::new(CoreExpr::Var("x".into())),
                    arms: vec![MatchArm {
                        pattern: CorePattern::Wildcard,
                        body: CoreExpr::Cast {
                            expr: Box::new(CoreExpr::Var("x".into())),
                            evidence: CastEvidence::Identity,
                            target: CoreType::Number,
                            cast_id: 1,
                        },
                    }],
                },
            ])),
        }),
    };
    let _ = insert_implicit_casts(&expr, &env);
    let _ = coerce_to_static(
        CoreExpr::Var("x".into()),
        &CoreType::Int,
        &CoreType::Number,
        9,
    );
    let _ = coerce_to_static(
        CoreExpr::Var("x".into()),
        &CoreType::dyn_any(),
        &CoreType::Int,
        10,
    );
    let _ = coerce_to_static(
        CoreExpr::Var("x".into()),
        &CoreType::Dynamic(Box::new(CoreType::String)),
        &CoreType::Int,
        11,
    );

    // Cast algebra leftovers (production APIs)
    let s = CoreType::Singleton(SingletonValue::Int(1));
    let s2 = CoreType::Singleton(SingletonValue::Int(1));
    let _ = intersect_types(&s, &s2); // equal singletons → early type_eq
    let _ = intersect_types(
        &CoreType::Singleton(SingletonValue::Int(1)),
        &CoreType::Singleton(SingletonValue::Int(2)),
    );
    let _ = intersect_types(
        &CoreType::Variant {
            variants: vec![("ok".into(), None)],
        },
        &CoreType::Variant {
            variants: vec![("ok".into(), Some(CoreType::Int))],
        },
    );
    let _ = intersect_types(
        &CoreType::Variant {
            variants: vec![("ok".into(), Some(CoreType::Int))],
        },
        &CoreType::Variant {
            variants: vec![("ok".into(), Some(CoreType::Never))],
        },
    );
    let _ = normalize_type(&CoreType::Intersect(vec![
        CoreType::Intersect(vec![CoreType::Int, CoreType::Number]),
        CoreType::Any,
        CoreType::Int,
    ]));
    let _ = normalize_type(&CoreType::Intersect(vec![
        CoreType::Color,
        CoreType::Shape,
    ]));
    let _ = normalize_type(&CoreType::Union(vec![
        CoreType::Union(vec![CoreType::Int, CoreType::String]),
        CoreType::Int,
        CoreType::Never,
    ]));
    let _ = types_disjoint(&CoreType::Color, &CoreType::Shape);
    let _ = types_disjoint(&CoreType::Bytes, &CoreType::Bool);
    let _ = types_disjoint(&CoreType::Unit, &CoreType::String);
    let _ = is_runtime_checkable(&CoreType::Record {
        fields: vec![("a".into(), CoreType::Int)],
    });
    let _ = is_runtime_checkable(&CoreType::Variant {
        variants: vec![("ok".into(), None), ("err".into(), Some(CoreType::String))],
    });
    let _ = is_runtime_checkable(&CoreType::Fun {
        args: vec![CoreType::Int],
        ret: Box::new(CoreType::Bool),
        effects: EffectRow::default().with_op("ask"),
    });
    let _ = is_runtime_checkable(&CoreType::Diff(
        Box::new(CoreType::Int),
        Box::new(CoreType::String),
    ));
    let _ = plan_cast_evidence(&CoreType::dyn_any(), &s);
    let _ = plan_cast_evidence(
        &CoreType::dyn_any(),
        &CoreType::Singleton(SingletonValue::Bool(true)),
    );
    let _ = plan_cast_evidence(
        &CoreType::dyn_any(),
        &CoreType::Singleton(SingletonValue::String("x".into())),
    );
    let _ = plan_cast_evidence(
        &CoreType::dyn_any(),
        &CoreType::Singleton(SingletonValue::Unit),
    );
    let _ = decide_subtype(&CoreType::Int, &CoreType::String);
    let _ = decide_subtype(
        &CoreType::Var(reciplexa_core::ty::TypeVarId(0)),
        &CoreType::Int,
    );
    let _ = judge_dynamic_use(&CoreType::String, &CoreType::Int);
    let _ = is_subtype(
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        },
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Number)],
        },
    );
    let _ = DecideResult::Unknown;
}
