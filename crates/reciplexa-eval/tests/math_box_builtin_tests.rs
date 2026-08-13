//! Language builtin `math-box` via math_value + estimate_box.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{estimate_math_box_from_value, eval_source, RuntimeValue};
use reciplexa_identity::document::StableNodeId;
use reciplexa_std::math::{MathAtom, MathClass};

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

#[test]
fn math_box_builtin_from_symbol_record() {
    let v = eval_source(
        r#"(val main (math-box (record (tag "math-symbol") (glyph "x") (class "ord"))))"#,
    )
    .unwrap();
    let expected = estimate_math_box_from_value(&RuntimeValue::Record(vec![
        ("tag".into(), RuntimeValue::String("math-symbol".into())),
        ("glyph".into(), RuntimeValue::String("x".into())),
        ("class".into(), RuntimeValue::String("ord".into())),
    ]))
    .unwrap();
    assert_eq!(
        field(&v, "tag"),
        &RuntimeValue::String("math-box".into())
    );
    match (field(&v, "width"), field(&v, "height"), field(&v, "depth")) {
        (RuntimeValue::Number(w), RuntimeValue::Number(h), RuntimeValue::Number(d)) => {
            assert!((w - expected.width).abs() < 1e-9);
            assert!((h - expected.height).abs() < 1e-9);
            assert!((d - expected.depth).abs() < 1e-9);
        }
        other => panic!("expected width/height/depth numbers, got {other:?}"),
    }
}

#[test]
fn math_box_builtin_from_symbol_string() {
    let v = eval_source(r#"(val main (math-box "x"))"#).unwrap();
    let expected =
        MathAtom::symbol(StableNodeId::new(0), "x", MathClass::Ordinary).estimate_box();
    match field(&v, "width") {
        RuntimeValue::Number(w) => assert!((w - expected.width).abs() < 1e-9),
        other => panic!("width number, got {other}"),
    }
}

#[test]
fn math_box_builtin_rejects_bad_args() {
    let err = eval_source(r#"(val main (math-box 1))"#).unwrap_err();
    assert!(err.message.contains("record") || err.message.contains("string"));
    let err = eval_source(r#"(val main (math-box))"#).unwrap_err();
    assert!(err.message.contains("expects 1"));
    let err = eval_source(
        r#"(val main (math-box (record (tag "math-absent"))))"#,
    )
    .unwrap_err();
    assert!(err.message.contains("math-box"));
}

#[test]
fn math_box_typechecks_as_dyn_any() {
    let ty = typecheck_language_source(
        r#"(val main (math-box (record (tag "math-symbol") (glyph "x") (class "ord"))))"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::dyn_any());
}
