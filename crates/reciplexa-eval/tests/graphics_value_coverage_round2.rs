//! Round-2 graphics_value leaves: helper Err arms and nondrawable matrix.

use reciplexa_eval::graphics_value::{
    color_from_graphics_value, document_from_graphics_value, page_from_graphics_value,
    shape_from_graphics_value, GraphicsValueError,
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

#[allow(dead_code)]
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

fn err_msg(r: Result<impl std::fmt::Debug, GraphicsValueError>) -> String {
    r.unwrap_err().message
}

#[test]
fn document_and_page_residual_errors() {
    assert!(
        err_msg(document_from_graphics_value(&RuntimeValue::Variant {
            tag: "nil".into(),
            payload: None,
        }))
        .contains("empty")
    );
    assert!(err_msg(document_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("pages".into())),
        (
            "items",
            RuntimeValue::Variant {
                tag: "nil".into(),
                payload: None,
            }
        ),
    ])))
    .contains("at least one"));
    assert!(err_msg(page_from_graphics_value(&RuntimeValue::Int(1))).contains("record"));
    assert!(err_msg(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        ("size", num(1.0)),
    ])))
    .contains("content"));
}

#[test]
fn shape_field_missing_matrix() {
    for v in [
        rec(vec![("tag", RuntimeValue::String("circle".into()))]),
        rec(vec![
            ("tag", RuntimeValue::String("rect".into())),
            ("x", num(0.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("ellipse".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
        ]),
        rec(vec![("tag", RuntimeValue::String("line".into()))]),
        rec(vec![("tag", RuntimeValue::String("polyline".into()))]),
        rec(vec![("tag", RuntimeValue::String("polygon".into()))]),
        rec(vec![("tag", RuntimeValue::String("ring".into()))]),
        rec(vec![("tag", RuntimeValue::String("frame".into()))]),
        rec(vec![("tag", RuntimeValue::String("text".into()))]),
        rec(vec![("tag", RuntimeValue::String("image".into()))]),
        rec(vec![("tag", RuntimeValue::String("group".into()))]),
    ] {
        assert!(shape_from_graphics_value(&v).is_err(), "{v:?}");
    }
}

#[test]
fn nondrawable_via_fill_and_stroke_wrappers() {
    let bad_rect = rec(vec![
        ("tag", RuntimeValue::String("rect".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("w", num(0.0)),
        ("h", num(1.0)),
    ]);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("shape", bad_rect.clone()),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", bad_rect),
        ("width", num(1.0)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());

    let bad_line = rec(vec![
        ("tag", RuntimeValue::String("line".into())),
        ("x1", num(0.0)),
        ("y1", num(0.0)),
        ("x2", num(0.0)),
        ("y2", num(0.0)),
    ]);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", bad_line),
        ("width", num(1.0)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
}

#[test]
fn color_paper_and_cons_helper_errors() {
    assert!(color_from_graphics_value(&RuntimeValue::Unit).is_err());
    assert!(color_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("rgb".into())),
        ("r", num(0.0)),
    ]))
    .is_err());
    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        (
            "size",
            rec(vec![("width", num(-1.0)), ("height", num(1.0))])
        ),
        ("content", circle(1.0, 2.0, 3.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("polyline".into())),
        ("points", RuntimeValue::Int(1)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("translate".into())),
        ("dx", num(1.0)),
        ("dy", num(2.0)),
        ("child", RuntimeValue::Int(1)),
    ]))
    .is_err());

    // ring without optional color uses default stroke
    let ring = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("ring".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("r", num(5.0)),
        ("width", num(1.0)),
    ]))
    .unwrap();
    assert!(matches!(ring, Shape::Ring(_)));
}
