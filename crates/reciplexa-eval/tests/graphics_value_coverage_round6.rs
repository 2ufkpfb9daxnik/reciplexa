//! Round-6 graphics_value: ShapeTag, srgb, stroke line/polyline, paint single-child,
//! empty lists, cons malformed, opacity bounds, optional text dims.

use reciplexa_eval::graphics_value::{
    color_from_graphics_value, document_from_graphics_value, page_from_graphics_value,
    shape_from_graphics_value,
};
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

fn tag_shape(name: &str) -> RuntimeValue {
    RuntimeValue::ShapeTag(name.into())
}

fn circle_tagged(x: f64, y: f64, r: f64) -> RuntimeValue {
    rec(vec![
        ("tag", tag_shape("circle")),
        ("x", num(x)),
        ("y", num(y)),
        ("r", num(r)),
    ])
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

#[allow(dead_code)]
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
fn shape_tag_and_srgb_color_paths() {
    assert!(shape_from_graphics_value(&circle_tagged(1.0, 2.0, 3.0)).is_ok());
    assert!(color_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("srgb".into())),
        ("r", num(0.1)),
        ("g", num(0.2)),
        ("b", num(0.3)),
    ]))
    .is_ok());
    assert!(color_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("cmyk".into())),
        ("c", num(0.0)),
    ]))
    .is_err());
}

#[test]
fn stroke_line_and_polyline_happy() {
    let color = rgb(0.0, 0.0, 0.0);
    let line = rec(vec![
        ("tag", RuntimeValue::String("line".into())),
        ("x1", num(0.0)),
        ("y1", num(0.0)),
        ("x2", num(10.0)),
        ("y2", num(5.0)),
    ]);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", line),
        ("width", num(0.5)),
        ("color", color.clone()),
    ]))
    .is_ok());

    let poly = rec(vec![
        ("tag", RuntimeValue::String("polyline".into())),
        (
            "points",
            cons_list(vec![num(0.0), num(0.0), num(5.0), num(5.0)]),
        ),
    ]);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", poly),
        ("width", num(1.0)),
        ("color", color),
    ]))
    .is_ok());
}

#[test]
fn fill_ellipse_polygon_and_text_optional_dims() {
    let color = rgb(0.5, 0.5, 0.5);
    let ellipse = rec(vec![
        ("tag", RuntimeValue::String("ellipse".into())),
        ("x", num(10.0)),
        ("y", num(20.0)),
        ("rx", num(30.0)),
        ("ry", num(15.0)),
    ]);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("shape", ellipse),
        ("color", color.clone()),
    ]))
    .is_ok());

    let polygon = rec(vec![
        ("tag", RuntimeValue::String("polygon".into())),
        (
            "points",
            cons_list(vec![
                num(0.0),
                num(0.0),
                num(10.0),
                num(0.0),
                num(5.0),
                num(8.0),
            ]),
        ),
    ]);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("shape", polygon),
        ("color", color.clone()),
    ]))
    .is_ok());

    let text = rec(vec![
        ("tag", RuntimeValue::String("text".into())),
        ("x", num(1.0)),
        ("y", num(2.0)),
        ("size", num(8.0)),
        ("content", RuntimeValue::String("hi".into())),
        ("w", num(40.0)),
        ("h", num(12.0)),
    ]);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("shape", text),
        ("color", color),
    ]))
    .is_ok());
}

#[test]
fn paint_single_child_and_empty_lists() {
    let c = circle(5.0, 5.0, 5.0);
    // fill only → children.len() == 1
    let single = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", c.clone()),
        ("fill", rgb(1.0, 0.0, 0.0)),
        ("stroke-width", num(0.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]))
    .unwrap();
    assert!(!matches!(single, reciplexa_scene::Shape::Group { .. }));

    assert!(document_from_graphics_value(&cons_list(vec![])).is_err());
    assert!(document_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("pages".into())),
        ("items", cons_list(vec![])),
    ]))
    .is_err());
    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("not-page".into())),
        ("size", rec(vec![("width", num(1.0)), ("height", num(1.0))])),
        ("content", c),
    ]))
    .is_err());
}

#[test]
fn opacity_bounds_and_group_empty() {
    let child = circle(1.0, 1.0, 1.0);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("opacity".into())),
        ("alpha", num(1.5)),
        ("child", child.clone()),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("opacity".into())),
        ("alpha", num(-0.1)),
        ("child", child.clone()),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("opacity".into())),
        ("alpha", num(0.0)),
        ("child", child),
    ]))
    .is_ok());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("group".into())),
        ("children", cons_list(vec![])),
    ]))
    .is_err());
}

#[test]
fn cons_malformed_and_unsupported_tags() {
    let bad_cons = RuntimeValue::Variant {
        tag: "cons".into(),
        payload: Some(Box::new(rec(vec![("head", num(1.0))]))),
    };
    assert!(document_from_graphics_value(&bad_cons).is_err());
    assert!(
        shape_from_graphics_value(&rec(vec![("tag", RuntimeValue::String("widget".into()))]))
            .is_err()
    );
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("shape", circle(1.0, 2.0, 3.0)),
        ("color", rgb(2.0, 0.0, 0.0)),
    ]))
    .is_err());
}
