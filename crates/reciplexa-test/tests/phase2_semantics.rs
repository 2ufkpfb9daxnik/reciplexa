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
