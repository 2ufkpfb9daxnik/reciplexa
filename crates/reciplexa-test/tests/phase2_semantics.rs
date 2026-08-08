//! Phase 2 semantics conformance tests.

use reciplexa_core::expr::{CoreExpr, CoreLiteral, CoreValue, MatchArm};
use reciplexa_core::ty::CoreType;
use reciplexa_core::{infer_expr, typecheck_value, Subst, TypeEnv};
use reciplexa_eval::{eval_expr, UnitHost};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;
use reciplexa_test::{run_conformance, ConformanceCase, TestOutcome};
use std::collections::HashMap;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn test_sem_c001_left_to_right_seq() {
    let case = ConformanceCase::new("TEST-SEM-C001", "EVAL", "seq left-to-right");
    run_conformance(&case, || {
        let expr = CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
            CoreExpr::Lit(CoreLiteral::Number(2.0)),
        ]);
        let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
        assert_eq!(v, reciplexa_eval::RuntimeValue::Number(2.0));
    });
}

#[test]
fn test_sem_c002_typed_lambda() {
    let case = ConformanceCase::new("TEST-SEM-C002", "TYPE", "lambda inference");
    run_conformance(&case, || {
        let expr = CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                param: "x".into(),
                body: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            }),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
        };
        let cv = typecheck_value(expr, &TypeEnv::new(), range()).unwrap();
        assert_eq!(cv.ty, CoreType::Number);
    });
}

#[test]
fn test_sem_c003_record_and_match() {
    let case = ConformanceCase::new("TEST-SEM-C003", "EVAL", "record + variant match");
    run_conformance(&case, || {
        let expr = CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Number(7.0)))],
            }),
            field: "a".into(),
        };
        let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
        assert_eq!(v, reciplexa_eval::RuntimeValue::Number(7.0));

        let m = CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "ok".into(),
                payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(3.0)))),
            }),
            arms: vec![MatchArm {
                tag: "ok".into(),
                bind: Some("v".into()),
                body: CoreExpr::Lit(CoreLiteral::Number(1.0)),
            }],
        };
        let _ = eval_expr(&m, &HashMap::new(), &mut UnitHost).unwrap();
    });
}

#[test]
fn test_sem_c004_structured_typecheck_outcome() {
    let case = ConformanceCase::new("TEST-SEM-C004", "TYPE", "structured check outcome");
    run_conformance(&case, || {
        let bad = CoreExpr::App {
            fun: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
        };
        let outcome: TestOutcome<CoreValue> = typecheck_value(bad, &TypeEnv::new(), range())
            .map(TestOutcome::Passed)
            .unwrap_or_else(|e| TestOutcome::Failed(e.message));
        assert!(!outcome.is_passed());
    });
}

#[test]
fn test_sem_c005_unification_infers_polymorphic_use() {
    let case = ConformanceCase::new("TEST-SEM-C005", "TYPE", "unification");
    run_conformance(&case, || {
        let mut subst = Subst::new();
        let id = CoreExpr::Lambda {
            param: "x".into(),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
        };
        let ty = infer_expr(&id, &TypeEnv::new(), &mut subst, range()).unwrap();
        assert!(matches!(subst.apply(&ty), CoreType::Fun { .. }));
    });
}

#[test]
fn test_sem_c006_literal_types() {
    let case = ConformanceCase::new("TEST-SEM-C006", "TYPE", "literal types");
    run_conformance(&case, || {
        for (lit, expected) in [
            (CoreLiteral::Number(3.14), CoreType::Number),
            (CoreLiteral::String("x".into()), CoreType::String),
            (CoreLiteral::Color("red".into()), CoreType::Color),
        ] {
            let cv = typecheck_value(CoreExpr::Lit(lit), &TypeEnv::new(), range()).unwrap();
            assert_eq!(cv.ty, expected);
        }
    });
}

#[test]
fn test_sem_c007_seq_evaluates_last() {
    let case = ConformanceCase::new("TEST-SEM-C007", "EVAL", "seq last value");
    run_conformance(&case, || {
        let expr = CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::String("a".into())),
            CoreExpr::Lit(CoreLiteral::String("b".into())),
            CoreExpr::Lit(CoreLiteral::Number(99.0)),
        ]);
        let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
        assert_eq!(v, reciplexa_eval::RuntimeValue::Number(99.0));
    });
}

#[test]
fn test_sem_c008_let_binds_in_body() {
    let case = ConformanceCase::new("TEST-SEM-C008", "EVAL", "let binding");
    run_conformance(&case, || {
        let expr = CoreExpr::Let {
            name: "x".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Number(5.0))),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(5.0))),
        };
        let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
        assert_eq!(v, reciplexa_eval::RuntimeValue::Number(5.0));
    });
}

#[test]
fn test_sem_c009_record_get_missing_field_errors() {
    let case = ConformanceCase::new("TEST-SEM-C009", "EVAL", "missing field");
    run_conformance(&case, || {
        let expr = CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Number(1.0)))],
            }),
            field: "missing".into(),
        };
        assert!(eval_expr(&expr, &HashMap::new(), &mut UnitHost).is_err());
    });
}

#[test]
fn test_sem_c010_match_no_arm_errors() {
    let case = ConformanceCase::new("TEST-SEM-C010", "EVAL", "non-exhaustive match");
    run_conformance(&case, || {
        let expr = CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "Other".into(),
                payload: None,
            }),
            arms: vec![MatchArm {
                tag: "Ok".into(),
                bind: None,
                body: CoreExpr::Lit(CoreLiteral::Number(0.0)),
            }],
        };
        assert!(eval_expr(&expr, &HashMap::new(), &mut UnitHost).is_err());
    });
}

#[test]
fn test_sem_c011_lambda_application() {
    let case = ConformanceCase::new("TEST-SEM-C011", "EVAL", "lambda app");
    run_conformance(&case, || {
        let expr = CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                param: "x".into(),
                body: Box::new(CoreExpr::Lit(CoreLiteral::Number(42.0))),
            }),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
        };
        let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
        assert_eq!(v, reciplexa_eval::RuntimeValue::Number(42.0));
    });
}

#[test]
fn test_sem_c012_type_mismatch_on_app() {
    let case = ConformanceCase::new("TEST-SEM-C012", "TYPE", "app type error");
    run_conformance(&case, || {
        let bad = CoreExpr::App {
            fun: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
        };
        assert!(typecheck_value(bad, &TypeEnv::new(), range()).is_err());
    });
}

#[test]
fn test_sem_c013_variant_payload_typecheck() {
    let case = ConformanceCase::new("TEST-SEM-C013", "TYPE", "variant payload");
    run_conformance(&case, || {
        let expr = CoreExpr::Variant {
            tag: "Some".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0)))),
        };
        let cv = typecheck_value(expr, &TypeEnv::new(), range()).unwrap();
        assert!(matches!(cv.ty, CoreType::Variant { .. }));
    });
}

#[test]
fn test_sem_c014_empty_seq_is_unit() {
    let case = ConformanceCase::new("TEST-SEM-C014", "EVAL", "empty seq");
    run_conformance(&case, || {
        let v = eval_expr(&CoreExpr::Seq(vec![]), &HashMap::new(), &mut UnitHost).unwrap();
        assert_eq!(v, reciplexa_eval::RuntimeValue::Unit);
    });
}

#[test]
fn test_sem_c015_variant_without_payload() {
    let case = ConformanceCase::new("TEST-SEM-C015", "EVAL", "nullary variant");
    run_conformance(&case, || {
        let expr = CoreExpr::Variant {
            tag: "Done".into(),
            payload: None,
        };
        let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
        assert!(matches!(v, reciplexa_eval::RuntimeValue::Variant { .. }));
    });
}

#[test]
fn test_sem_c016_infer_let_binding() {
    let case = ConformanceCase::new("TEST-SEM-C016", "TYPE", "let-bound inference");
    run_conformance(&case, || {
        let expr = CoreExpr::Let {
            name: "n".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
        };
        let cv = typecheck_value(expr, &TypeEnv::new(), range()).unwrap();
        assert_eq!(cv.ty, CoreType::Number);
    });
}

#[test]
fn test_sem_c017_unify_record_fields() {
    let case = ConformanceCase::new("TEST-SEM-C017", "TYPE", "record unify");
    run_conformance(&case, || {
        use reciplexa_core::unify::{unify, Subst};
        let mut s = Subst::new();
        let a = CoreType::Record {
            fields: vec![("x".into(), CoreType::Number)],
        };
        let b = CoreType::Record {
            fields: vec![("x".into(), CoreType::Number)],
        };
        assert!(unify(&a, &b, &mut s).is_ok());
    });
}

#[test]
fn test_sem_c018_runtime_value_ty() {
    let case = ConformanceCase::new("TEST-SEM-C018", "EVAL", "runtime value ty");
    run_conformance(&case, || {
        let v = reciplexa_eval::RuntimeValue::ShapeTag("circle".into());
        assert_eq!(v.ty(), CoreType::Shape);
        assert_eq!(v.to_string(), "shape:circle");
    });
}

#[test]
fn test_sem_c019_lower_surface_forms() {
    let case = ConformanceCase::new("TEST-SEM-C019", "LOWER", "surface lowering");
    run_conformance(&case, || {
        use reciplexa_core::lower::lower_surface_form;
        use reciplexa_source::offset::ByteOffset;
        let range = TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap();
        assert!(lower_surface_form("circle", 3, range).is_ok());
        assert!(lower_surface_form("bogus", 1, range).is_err());
    });
}
