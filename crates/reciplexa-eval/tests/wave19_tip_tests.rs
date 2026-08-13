//! Wave 19 M1 tip: phantom_box / smash_box on estimated math boxes.

use reciplexa_eval::{estimate_math_box_from_value, eval_source, RuntimeValue};
use reciplexa_std::math::{phantom_box, smash_box, MathAtom, MathClass};
use reciplexa_identity::document::StableNodeId;

fn field<'a>(rec: &'a RuntimeValue, name: &str) -> &'a RuntimeValue {
    match rec {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("missing {name}")),
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
fn tip_wave19_phantom_smash_on_estimate() {
    let atom = MathAtom::symbol(StableNodeId::new(1), "∑", MathClass::Operator);
    let inner = atom.estimate_box();
    assert!(inner.width > 0.0);
    assert!(inner.total_height() > 0.0);

    let ph = phantom_box(inner);
    assert!((ph.width - 0.0).abs() < 1e-9);
    assert!((ph.height - inner.height).abs() < 1e-9);
    assert!((ph.depth - inner.depth).abs() < 1e-9);

    let sm = smash_box(inner);
    assert!((sm.width - inner.width).abs() < 1e-9);
    assert_eq!(sm.total_height(), 0.0);

    // Host path: package math record → estimate → phantom/smash.
    let v = eval_source(
        r#"(val main (math-box (record (tag "math-symbol") (glyph "x") (class "ord"))))"#,
    )
    .unwrap();
    assert_eq!(field(&v, "tag"), &RuntimeValue::String("math-box".into()));
    let w = num(field(&v, "width"));
    let h = num(field(&v, "height"));
    let d = num(field(&v, "depth"));
    let via = estimate_math_box_from_value(&RuntimeValue::Record(vec![
        ("tag".into(), RuntimeValue::String("math-symbol".into())),
        ("glyph".into(), RuntimeValue::String("x".into())),
        ("class".into(), RuntimeValue::String("ord".into())),
    ]))
    .unwrap();
    assert!((via.width - w).abs() < 1e-9);
    assert!((phantom_box(via).width - 0.0).abs() < 1e-9);
    assert!((smash_box(via).height - 0.0).abs() < 1e-9);
    assert!((smash_box(via).depth - 0.0).abs() < 1e-9);
    assert!((phantom_box(via).height - h).abs() < 1e-9);
    assert!((phantom_box(via).depth - d).abs() < 1e-9);
}
