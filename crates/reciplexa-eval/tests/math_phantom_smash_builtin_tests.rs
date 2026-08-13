//! Language builtins `math-phantom` / `math-smash` via phantom_box / smash_box.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_identity::document::StableNodeId;
use reciplexa_std::math::{phantom_box, smash_box, MathAtom, MathClass};

fn field<'a>(rec: &'a RuntimeValue, name: &str) -> &'a RuntimeValue {
    match rec {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("missing field {name}")),
        other => panic!("expected record, got {other}"),
    }
}

fn num(v: &RuntimeValue) -> f64 {
    match v {
        RuntimeValue::Number(n) => *n,
        RuntimeValue::F64(n) => *n,
        RuntimeValue::Int(n) => *n as f64,
        other => panic!("expected number, got {other}"),
    }
}

#[test]
fn math_phantom_zeros_width_from_symbol_record() {
    let v = eval_source(
        r#"(val main (math-phantom (record (tag "math-symbol") (glyph "Σ") (class "op"))))"#,
    )
    .unwrap();
    let expected = phantom_box(
        MathAtom::symbol(StableNodeId::new(0), "Σ", MathClass::Operator).estimate_box(),
    );
    assert_eq!(
        field(&v, "tag"),
        &RuntimeValue::String("math-phantom".into())
    );
    assert!((num(field(&v, "width")) - expected.width).abs() < 1e-9);
    assert!((num(field(&v, "height")) - expected.height).abs() < 1e-9);
    assert!((num(field(&v, "depth")) - expected.depth).abs() < 1e-9);
    assert!((num(field(&v, "width")) - 0.0).abs() < 1e-9);
}

#[test]
fn math_smash_zeros_height_depth_from_string() {
    let v = eval_source(r#"(val main (math-smash "x"))"#).unwrap();
    let expected =
        smash_box(MathAtom::symbol(StableNodeId::new(0), "x", MathClass::Ordinary).estimate_box());
    assert_eq!(field(&v, "tag"), &RuntimeValue::String("math-smash".into()));
    assert!((num(field(&v, "width")) - expected.width).abs() < 1e-9);
    assert!((num(field(&v, "height")) - 0.0).abs() < 1e-9);
    assert!((num(field(&v, "depth")) - 0.0).abs() < 1e-9);
}

#[test]
fn math_phantom_accepts_math_box_metrics_record() {
    let v = eval_source(
        r#"(val main (math-phantom (math-box (record (tag "math-symbol") (glyph "x") (class "ord")))))"#,
    )
    .unwrap();
    assert!((num(field(&v, "width")) - 0.0).abs() < 1e-9);
    assert!(num(field(&v, "height")) > 0.0 || num(field(&v, "depth")) >= 0.0);
}

#[test]
fn math_phantom_smash_reject_bad_args() {
    let err = eval_source(r#"(val main (math-phantom 1))"#).unwrap_err();
    assert!(err.message.contains("math-phantom"));
    let err = eval_source(r#"(val main (math-smash))"#).unwrap_err();
    assert!(err.message.contains("expects 1"));
}

#[test]
fn math_phantom_typechecks_as_dyn_any() {
    let ty = typecheck_language_source(r#"(val main (math-phantom "x"))"#).unwrap();
    assert_eq!(ty, CoreType::dyn_any());
    let ty = typecheck_language_source(r#"(val main (math-smash "x"))"#).unwrap();
    assert_eq!(ty, CoreType::dyn_any());
}
