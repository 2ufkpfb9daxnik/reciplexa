//! Wave 9 C5 tip coverage: fraction/radical stubs, hang-width, graphics wrap-em.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_eval::graphics_value::shape_from_graphics_value;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::Shape;
use reciplexa_std::japanese::{hang_width_em_char, HANG_WIDTH_EM};
use reciplexa_std::math::{
    fraction_rule_metrics, radical_vinculum_index_offsets, MathAtom, MathBox, MathClass,
    FRAC_RULE_THICKNESS_EM, RADICAL_VINCULUM_CLEARANCE_EM,
};

fn id(n: u64) -> StableNodeId {
    StableNodeId::new(n)
}

fn rec(fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
    RuntimeValue::Record(
        fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

#[test]
fn tip_wave9_fraction_radical_and_hang_width() {
    let (rule, _, _) = fraction_rule_metrics();
    assert!((rule - FRAC_RULE_THICKNESS_EM).abs() < 1e-9);

    let body = MathBox::new(1.0, 0.7, 0.2);
    let (vy, _, _) = radical_vinculum_index_offsets(body, None);
    assert!((vy - (body.height + RADICAL_VINCULUM_CLEARANCE_EM)).abs() < 1e-9);

    let frac = MathAtom::fraction(
        id(1),
        MathAtom::symbol(id(2), "a", MathClass::Ordinary),
        MathAtom::symbol(id(3), "b", MathClass::Ordinary),
    );
    assert!(frac.estimate_box().total_height() > 1.0);

    let hang = eval_source(r#"(val main (hang-width "、"))"#).unwrap();
    assert_eq!(hang, RuntimeValue::Number(HANG_WIDTH_EM));
    assert!((hang_width_em_char('、') - HANG_WIDTH_EM).abs() < 1e-9);
    let _ = typecheck_language_source(r#"(val main (hang-width "。"))"#).unwrap();
}

#[test]
fn tip_wave9_graphics_text_wrap_em() {
    let shape = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("text".into())),
        ("x", RuntimeValue::F64(0.0)),
        ("y", RuntimeValue::F64(0.0)),
        ("size", RuntimeValue::F64(10.0)),
        (
            "content",
            RuntimeValue::String("あいうえおかきくけこ".into()),
        ),
        ("wrap-em", RuntimeValue::F64(4.0)),
    ]))
    .expect("wrap-em text");
    match shape {
        Shape::Group { children, .. } => {
            assert!(children.len() >= 2);
            assert!(children.iter().all(|c| matches!(c, Shape::Text(_))));
        }
        Shape::Text(_) => panic!("expected soft-wrapped Group"),
        other => panic!("unexpected shape {other:?}"),
    }
}
