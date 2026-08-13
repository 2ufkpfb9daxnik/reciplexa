//! Wave 17 K3 tip coverage: EstimateStyle + math-box style field.

use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_identity::document::StableNodeId;
use reciplexa_std::math::{EstimateStyle, MathAtom, MathClass, SCRIPT_SCALE, SCRIPT_SCALE_TEXT};

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

#[test]
fn tip_wave17_estimate_style_and_math_box() {
    assert!(SCRIPT_SCALE_TEXT < SCRIPT_SCALE);
    let base = MathAtom::symbol(StableNodeId::new(0), "x", MathClass::Ordinary);
    let sup = MathAtom::symbol(StableNodeId::new(1), "2", MathClass::Ordinary);
    let scripts = MathAtom::scripts(StableNodeId::new(2), base, Some(sup), None);
    let d = scripts.estimate_box_with_style(EstimateStyle::Display);
    let t = scripts.estimate_box_with_style(EstimateStyle::Text);
    assert!(t.width < d.width);

    let v = eval_source(
        r#"(val main (math-box (record (tag "math-scripts") (base (record (tag "math-symbol") (glyph "x") (class "ord"))) (superscript (record (tag "math-symbol") (glyph "2") (class "ord"))) (style "text"))))"#,
    )
    .unwrap();
    assert_eq!(field(&v, "style"), &RuntimeValue::String("text".into()));
}
