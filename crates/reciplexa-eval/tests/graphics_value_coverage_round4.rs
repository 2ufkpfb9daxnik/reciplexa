//! Round-4 graphics_value: fill/stroke/paint field omission matrix.

use reciplexa_eval::graphics_value::shape_from_graphics_value;
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
fn fill_stroke_paint_each_missing_field() {
    let c = circle(1.0, 2.0, 3.0);
    let color = rgb(0.0, 0.0, 0.0);

    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("color", color.clone()),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("shape", c.clone()),
    ]))
    .is_err());

    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", c.clone()),
        ("color", color.clone()),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", c.clone()),
        ("width", num(1.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("width", num(1.0)),
        ("color", color.clone()),
    ]))
    .is_err());

    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("fill", color.clone()),
        ("stroke-width", num(1.0)),
        ("stroke-color", color.clone()),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", c.clone()),
        ("stroke-width", num(1.0)),
        ("stroke-color", color.clone()),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", c.clone()),
        ("fill", color.clone()),
        ("stroke-color", color.clone()),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", c),
        ("fill", color.clone()),
        ("stroke-width", num(1.0)),
    ]))
    .is_err());
}

#[test]
fn shape_inner_missing_coords_and_bad_tag() {
    assert!(
        shape_from_graphics_value(&rec(vec![("tag", RuntimeValue::Int(3)), ("x", num(0.0)),]))
            .is_err()
    );
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        (
            "shape",
            rec(vec![("tag", RuntimeValue::String("ellipse".into()))])
        ),
        ("width", num(1.0)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        (
            "shape",
            rec(vec![("tag", RuntimeValue::String("image".into()))]),
        ),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
}
