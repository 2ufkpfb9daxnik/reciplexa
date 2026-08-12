//! Round-3 graphics_value: paint/cons/tag residual region ends.

use reciplexa_eval::graphics_value::{
    document_from_graphics_value, page_from_graphics_value, shape_from_graphics_value,
};
use reciplexa_eval::RuntimeValue;
use reciplexa_scene::Shape;

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

fn cons_list(items: Vec<RuntimeValue>) -> RuntimeValue {
    let mut acc = RuntimeValue::Variant {
        tag: "nil".into(),
        payload: None,
    };
    for item in items.into_iter().rev() {
        acc = RuntimeValue::Variant {
            tag: "cons".into(),
            payload: Some(Box::new(rec(vec![("head", item), ("tail", acc)]))),
        };
    }
    acc
}

fn page(content: RuntimeValue) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        (
            "size",
            rec(vec![("width", num(210.0)), ("height", num(297.0))]),
        ),
        ("content", content),
    ])
}

#[test]
fn paint_missing_fields_and_fill_only_fallback() {
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", circle(1.0, 2.0, 3.0)),
        ("stroke-width", num(0.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", circle(1.0, 2.0, 3.0)),
        ("fill", rgb(1.0, 0.0, 0.0)),
        ("stroke-width", num(1.0)),
    ]))
    .is_err());

    let fill_only = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", circle(1.0, 2.0, 3.0)),
        ("fill", rgb(0.2, 0.4, 0.6)),
        ("stroke-width", num(0.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]))
    .unwrap();
    assert!(matches!(fill_only, Shape::Circle(_)));
}

#[test]
fn polyline_collinear_nondrawable_via_stroke() {
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        (
            "shape",
            rec(vec![
                ("tag", RuntimeValue::String("polyline".into())),
                (
                    "points",
                    cons_list(vec![num(0.0), num(0.0), num(0.0), num(0.0)]),
                ),
            ]),
        ),
        ("width", num(1.0)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
}

#[test]
fn cons_malformed_and_tag_mismatch() {
    let broken_cons = RuntimeValue::Variant {
        tag: "cons".into(),
        payload: Some(Box::new(rec(vec![("head", num(1.0))]))),
    };
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("polyline".into())),
        ("points", broken_cons),
    ]))
    .is_err());

    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("circle".into())),
        ("size", rec(vec![("width", num(1.0)), ("height", num(1.0))]),),
        ("content", circle(1.0, 2.0, 3.0)),
    ]))
    .is_err());

    let doc = document_from_graphics_value(&page(circle(1.0, 2.0, 3.0))).unwrap();
    assert_eq!(doc.pages.len(), 1);
}

#[test]
fn text_partial_box_and_stroke_circle_ok() {
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("text".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("size", num(8.0)),
        ("content", RuntimeValue::String("x".into())),
        ("w", num(10.0)),
    ]))
    .is_err());

    let ring = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", circle(1.0, 2.0, 3.0)),
        ("width", num(0.5)),
        ("color", rgb(1.0, 0.0, 0.0)),
    ]))
    .unwrap();
    assert!(matches!(ring, Shape::Ring(_)));
}
