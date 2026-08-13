//! Round-15 eval: graphics paint fallbacks and cons-list error paths.

use reciplexa_eval::graphics_value::{document_from_graphics_value, shape_from_graphics_value};
use reciplexa_eval::RuntimeValue;

fn rec(fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
    RuntimeValue::Record(
        fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

fn num(n: f64) -> RuntimeValue {
    RuntimeValue::F64(n)
}

fn circle(x: f64, y: f64, r: f64) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("circle".into())),
        ("x", num(x)),
        ("y", num(y)),
        ("r", num(r)),
    ])
}

fn rgb(r: f64, g: f64, b: f64) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("rgb".into())),
        ("r", num(r)),
        ("g", num(g)),
        ("b", num(b)),
    ])
}

#[test]
fn paint_fallback_fill_only_and_stroke_only() {
    let c = circle(3.0, 3.0, 3.0);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", c.clone()),
        ("fill", rgb(1.0, 0.0, 0.0)),
        ("stroke-width", num(0.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_ok());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", c),
        ("fill", rgb(1.0, 0.0, 0.0)),
        ("stroke-width", num(1.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_ok());
}

#[test]
fn document_rejects_non_record_page_in_list() {
    let bad = RuntimeValue::Variant {
        tag: "cons".into(),
        payload: Some(Box::new(rec(vec![
            ("head", RuntimeValue::String("not-page".into())),
            (
                "tail",
                RuntimeValue::Variant {
                    tag: "nil".into(),
                    payload: None,
                },
            ),
        ]))),
    };
    assert!(document_from_graphics_value(&bad).is_err());
}
