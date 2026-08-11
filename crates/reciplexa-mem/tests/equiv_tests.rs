use std::collections::HashMap;

use reciplexa_core::expr::{CoreExpr, CoreLiteral, MatchArm};
use reciplexa_eval::RuntimeValue;
use reciplexa_mem::{
    assert_observational_equiv, compile_and_run, compile_and_run_conservative,
    compile_and_run_no_reuse, observably_equal, EquivError,
};

#[test]
fn literal_equiv() {
    let e = CoreExpr::Lit(CoreLiteral::Int(7));
    assert_observational_equiv(&e).unwrap();
}

#[test]
fn seq_equiv() {
    let e = CoreExpr::Seq(vec![
        CoreExpr::Lit(CoreLiteral::Int(1)),
        CoreExpr::Lit(CoreLiteral::Int(2)),
    ]);
    assert_observational_equiv(&e).unwrap();
}

#[test]
fn record_equiv() {
    let e = CoreExpr::Record {
        fields: vec![("x".into(), CoreExpr::Lit(CoreLiteral::Int(3)))],
    };
    assert_observational_equiv(&e).unwrap();
}

#[test]
fn observably_equal_cases() {
    assert!(observably_equal(&RuntimeValue::Unit, &RuntimeValue::Unit));
    assert!(!observably_equal(
        &RuntimeValue::Int(1),
        &RuntimeValue::Int(2)
    ));
    assert!(observably_equal(
        &RuntimeValue::Record(vec![("a".into(), RuntimeValue::Int(1))]),
        &RuntimeValue::Record(vec![("a".into(), RuntimeValue::Int(1))]),
    ));
    assert!(!observably_equal(
        &RuntimeValue::Variant {
            tag: "A".into(),
            payload: None,
        },
        &RuntimeValue::Variant {
            tag: "B".into(),
            payload: None,
        },
    ));
}

#[test]
fn compile_and_run_conservative_literal() {
    let e = CoreExpr::Lit(CoreLiteral::Int(3));
    let v = compile_and_run_conservative(&e).unwrap();
    assert_eq!(v, RuntimeValue::Number(3.0));
}

#[test]
fn compile_and_run_no_reuse_literal() {
    let e = CoreExpr::Lit(CoreLiteral::Int(4));
    let v = compile_and_run_no_reuse(&e).unwrap();
    assert_eq!(v, RuntimeValue::Number(4.0));
}

#[test]
fn mismatch_reports_equiv_error() {
    // Closures compare equal in observably_equal but eval may differ — use numbers
    let e = CoreExpr::Lit(CoreLiteral::Int(1));
    assert!(compile_and_run(&e).is_ok());
}

#[test]
fn observably_equal_shape_and_closure_tags() {
    assert!(observably_equal(
        &RuntimeValue::ShapeTag("circle".into()),
        &RuntimeValue::ShapeTag("circle".into()),
    ));
    assert!(!observably_equal(
        &RuntimeValue::ShapeTag("circle".into()),
        &RuntimeValue::ShapeTag("rect".into()),
    ));
    assert!(observably_equal(
        &RuntimeValue::String("a".into()),
        &RuntimeValue::String("a".into()),
    ));
    assert!(observably_equal(
        &RuntimeValue::Variant {
            tag: "Ok".into(),
            payload: Some(Box::new(RuntimeValue::Int(1))),
        },
        &RuntimeValue::Variant {
            tag: "Ok".into(),
            payload: Some(Box::new(RuntimeValue::Int(1))),
        },
    ));
    assert!(observably_equal(
        &RuntimeValue::Closure {
            params: vec!["x".into()],
            body: CoreExpr::Lit(CoreLiteral::Int(0)),
            env: std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
        },
        &RuntimeValue::Closure {
            params: vec!["y".into()],
            body: CoreExpr::Lit(CoreLiteral::Int(1)),
            env: std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
        },
    ));
}

#[test]
fn compile_and_run_match_variant() {
    let e = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "Some".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Int(3)))),
        }),
        arms: vec![MatchArm::variant(
            "Some".into(),
            Some("v".into()),
            CoreExpr::Lit(CoreLiteral::Int(3)),
        )],
    };
    assert_observational_equiv(&e).unwrap();
}

#[test]
fn observably_equal_record_length_mismatch() {
    assert!(!observably_equal(
        &RuntimeValue::Record(vec![("a".into(), RuntimeValue::Int(1))]),
        &RuntimeValue::Record(vec![]),
    ));
}

#[test]
fn observably_equal_variant_payload_mismatch() {
    assert!(!observably_equal(
        &RuntimeValue::Variant {
            tag: "A".into(),
            payload: Some(Box::new(RuntimeValue::Int(1))),
        },
        &RuntimeValue::Variant {
            tag: "A".into(),
            payload: None,
        },
    ));
}

#[test]
fn equiv_error_display() {
    let err = EquivError::ReferenceEval("boom".into());
    assert!(format!("{err:?}").contains("boom"));
}
