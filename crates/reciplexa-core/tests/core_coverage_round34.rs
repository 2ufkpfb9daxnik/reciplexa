//! Round-34 core: more check/cast/elaborate sources toward 98.75%.

use reciplexa_core::cast::{
    cast_success_type, compose_evidence, intersect_types, is_runtime_checkable, is_subtype,
    normalize_type, plan_cast_evidence, simplify_evidence, types_disjoint, CastEvidence,
};
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
fn core_round34_dense_residual_matrix() {
    // Open record + row unify denser
    tip("(val main (fn (r) (field (as (record (a int) (row r)) r) b)))");
    tip("(type R (record (x f64) (y f64) (row rest)))\n(val main (fn (p) (+ (field (as R p) x) (field (as R p) z))))");

    // Handler / with / failure
    tip("(val main (handle failure (fn (e) e) (perform failure \"boom\")))");
    tip("(val main (with (handler log (fn (m) m)) (perform log \"x\")))");
    tip("(val main (handler failure (fn (e) e)))");

    // Record update/extend ok paths
    tip("(val main (record-update (record (a 1) (b 2)) (a 3)))");
    tip("(val main (record-extend (record (a 1)) (b 2)))");

    // Match / lit patterns
    tip("(val main (match 1 (1 -> 0) (_ -> 1)))");
    tip("(val main (match \"x\" (\"x\" -> 1) (_ -> 0)))");
    tip("(val main (match true (true -> 1) (false -> 0)))");
    tip("(val main (match unit (unit -> 1) (_ -> 0)))");

    // Cast nests
    tip("(val main (as (dynamic (union int string)) 1))");
    tip("(val main (as (optional-field int) 1))");
    tip("(val main (try-cast (as (dynamic any) 1) string))");
    tip("(val main (check-cast (as (dynamic number) 1.5) f64))");

    // Letrec / local / set
    tip("(val main (letrec ((f (fn (x) (if (< x 1) x (f (- x 1))))) (f 3)))");
    tip("(val main (local (var c 0) (seq (set c 1) c)))");

    // Data / type residuals
    tip("(data either (left int) (right string))\n(val main (match (left 1) (left x -> x) (right _ -> 0)))");
    tip("(type N (not (union string bool)))\n(val main 1)");
    tip("(type D (diff (union int string) string))\n(val main 1)");

    // Quarantined / overflow
    tip("(circle 1 2 3)");
    tip("(val main (as number 1e99999))");

    // insert_casts Match/Variant/RecordUpdate
    let mut env = TypeEnv::new();
    env.insert("x", CoreType::Int);
    env.insert(
        "f",
        CoreType::Fun {
            args: vec![CoreType::Number, CoreType::Number],
            ret: Box::new(CoreType::Number),
            effects: EffectRow::default(),
        },
    );
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(CoreExpr::Var("x".into()))),
        }),
        arms: vec![
            MatchArm {
                pattern: CorePattern::Variant {
                    tag: "some".into(),
                    payload: Some(Box::new(CorePattern::Bind("y".into()))),
                },
                body: CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("f".into())),
                    args: vec![
                        CoreExpr::Var("y".into()),
                        CoreExpr::Lit(CoreLiteral::Int(1)),
                    ],
                },
            },
            MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(0)),
            },
        ],
    };
    let _ = insert_implicit_casts(&expr, &env);

    // Cast API denser
    let _ = cast_success_type(&CoreType::Int, &CoreType::Number);
    let _ = compose_evidence(vec![
        CastEvidence::Identity,
        CastEvidence::Compose(vec![CastEvidence::TagCheck { tag: "int".into() }]),
        CastEvidence::NumericPromote,
    ]);
    let _ = simplify_evidence(CastEvidence::Compose(vec![
        CastEvidence::Compose(vec![CastEvidence::Identity]),
        CastEvidence::Identity,
    ]));
    let _ = normalize_type(&CoreType::Union(vec![
        CoreType::Never,
        CoreType::Union(vec![CoreType::Int, CoreType::Int]),
        CoreType::String,
    ]));
    let _ = normalize_type(&CoreType::Intersect(vec![
        CoreType::Any,
        CoreType::Intersect(vec![CoreType::Number, CoreType::Int]),
    ]));
    let _ = intersect_types(
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        },
        &CoreType::Record {
            fields: vec![("a".into(), CoreType::Never)],
        },
    );
    let _ = types_disjoint(&CoreType::Shape, &CoreType::Color);
    let _ = is_subtype(
        &CoreType::Singleton(SingletonValue::Int(1)),
        &CoreType::Number,
    );
    let _ = is_runtime_checkable(&CoreType::App {
        ctor: "box".into(),
        args: vec![CoreType::Int],
    });
    let _ = plan_cast_evidence(&CoreType::dyn_any(), &CoreType::Bytes);
    let _ = plan_cast_evidence(&CoreType::dyn_any(), &CoreType::Unit);
    let _ = plan_cast_evidence(&CoreType::dyn_any(), &CoreType::Bool);
}
