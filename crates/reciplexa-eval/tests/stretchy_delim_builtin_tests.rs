//! Language builtin `stretchy-delim` + math-delimiter `stretch-factor` field.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_identity::document::StableNodeId;
use reciplexa_std::math::{MathAtom, MathClass};

fn field_num(fields: &[(String, RuntimeValue)], name: &str) -> f64 {
    fields
        .iter()
        .find(|(k, _)| k == name)
        .and_then(|(_, v)| match v {
            RuntimeValue::Number(n) => Some(*n),
            RuntimeValue::F64(n) => Some(*n),
            RuntimeValue::Int(n) => Some(*n as f64),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing numeric field `{name}`"))
}

#[test]
fn stretchy_delim_builtin_matches_estimate() {
    let body = MathAtom::symbol(StableNodeId::new(0), "□", MathClass::Ordinary);
    let atom = MathAtom::delimiter_with_stretch(StableNodeId::new(1), "(", ")", body, 2.5);
    let expected = atom.estimate_box();
    let v = eval_source(r#"(val main (stretchy-delim "(" ")" 2.5))"#).unwrap();
    let RuntimeValue::Record(fields) = v else {
        panic!("expected record, got {v:?}");
    };
    assert_eq!(
        fields.iter().find(|(k, _)| k == "tag").map(|(_, v)| v),
        Some(&RuntimeValue::String("stretchy-delim".into()))
    );
    assert!((field_num(&fields, "stretch-factor") - 2.5).abs() < 1e-9);
    assert!((field_num(&fields, "width") - expected.width).abs() < 1e-9);
    assert!((field_num(&fields, "height") - expected.height).abs() < 1e-9);
    assert!((field_num(&fields, "depth") - expected.depth).abs() < 1e-9);
}

#[test]
fn stretchy_delim_rejects_bad_args() {
    let err = eval_source(r#"(val main (stretchy-delim 1 ")" 1))"#).unwrap_err();
    assert!(err.message.contains("string"));
    let err = eval_source(r#"(val main (stretchy-delim "(" ")" "x"))"#).unwrap_err();
    assert!(err.message.contains("numeric"));
    let err = eval_source(r#"(val main (stretchy-delim "(" ")"))"#).unwrap_err();
    assert!(err.message.contains("expects 3"));
}

#[test]
fn stretchy_delim_typechecks_lightly() {
    let ty = typecheck_language_source(r#"(val main (stretchy-delim "(" ")" 1.5))"#).unwrap();
    assert!(matches!(ty, CoreType::Dynamic(_) | CoreType::Any) || ty == CoreType::dyn_any());
}

#[test]
fn math_box_honors_delimiter_stretch_factor_field() {
    let flat = eval_source(
        r#"(val main (math-box (record (tag "math-delimiter") (left "(") (right ")")
          (body (record (tag "math-symbol") (glyph "x") (class "ord"))))))"#,
    )
    .unwrap();
    let tall = eval_source(
        r#"(val main (math-box (record (tag "math-delimiter") (left "(") (right ")")
          (body (record (tag "math-symbol") (glyph "x") (class "ord")))
          (stretch-factor 3.0))))"#,
    )
    .unwrap();
    let RuntimeValue::Record(flat_f) = flat else {
        panic!("flat");
    };
    let RuntimeValue::Record(tall_f) = tall else {
        panic!("tall");
    };
    assert!(field_num(&tall_f, "height") > field_num(&flat_f, "height"));
}
