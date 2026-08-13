//! Round-7 graphics_value: paint empty→stroke fallback, cons mid-list break,
//! frame without color, non-drawable polygon/ring/frame edges.

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

fn rgb(r: f64, g: f64, b: f64) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("rgb".into())),
        ("r", num(r)),
        ("g", num(g)),
        ("b", num(b)),
    ])
}

fn rgba(r: f64, g: f64, b: f64, a: f64) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("rgba".into())),
        ("r", num(r)),
        ("g", num(g)),
        ("b", num(b)),
        ("a", num(a)),
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

fn circle(x: f64, y: f64, r: f64) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("circle".into())),
        ("x", num(x)),
        ("y", num(y)),
        ("r", num(r)),
    ])
}

#[test]
fn paint_transparent_fill_stroke_retry_empty_children() {
    // Transparent fill + strokeable shape that fails first stroke (r=0 ring) →
    // children empty + stroke_width > 0 → fallback shape_with_stroke (miss ~208/213).
    let bad_circle = circle(0.0, 0.0, 0.0);
    let _ = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", bad_circle),
        ("fill", rgba(1.0, 0.0, 0.0, 0.0)),
        ("stroke-width", num(1.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]));

    // Transparent fill + fill-only tag (ellipse): stroke unsupported → empty → retry.
    let ellipse = rec(vec![
        ("tag", RuntimeValue::String("ellipse".into())),
        ("x", num(1.0)),
        ("y", num(2.0)),
        ("rx", num(3.0)),
        ("ry", num(4.0)),
    ]);
    let _ = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", ellipse),
        ("fill", rgba(0.0, 1.0, 0.0, 0.0)),
        ("stroke-width", num(2.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]));

    // Transparent fill + stroke_width 0 → fill fallback path (already partial).
    let _ = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", circle(1.0, 1.0, 1.0)),
        ("fill", rgba(0.0, 0.0, 1.0, 0.0)),
        ("stroke-width", num(0.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]));
}

#[test]
fn cons_tail_breaks_to_non_list() {
    // is_cons_or_nil passes, then mid-list tail is not cons/nil → miss ~593.
    let broken = RuntimeValue::Variant {
        tag: "cons".into(),
        payload: Some(Box::new(rec(vec![
            ("head", circle(1.0, 2.0, 3.0)),
            ("tail", RuntimeValue::String("not-a-list".into())),
        ]))),
    };
    assert!(document_from_graphics_value(&broken).is_err());
    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        (
            "size",
            rec(vec![("width", num(210.0)), ("height", num(297.0))]),
        ),
        ("content", broken.clone()),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("group".into())),
        ("children", broken),
    ]))
    .is_err());
}

#[test]
fn frame_without_color_and_non_drawable_edges() {
    // frame tag with no color → None override + unwrap_or BLACK
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("frame".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("w", num(10.0)),
        ("h", num(5.0)),
        ("width", num(0.5)),
    ]))
    .is_ok());

    // non-drawable frame (zero size / zero stroke width)
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("frame".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("w", num(0.0)),
        ("h", num(5.0)),
        ("width", num(0.5)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("frame".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("w", num(10.0)),
        ("h", num(5.0)),
        ("width", num(0.0)),
    ]))
    .is_err());

    // ring non-drawable
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("ring".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("r", num(0.0)),
        ("width", num(1.0)),
    ]))
    .is_err());

    // polygon with ≥3 points (happy) + stroke circle non-drawable via paint already above
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("polygon".into())),
        (
            "points",
            cons_list(vec![
                num(0.0),
                num(0.0),
                num(1.0),
                num(0.0),
                num(0.0),
                num(1.0),
            ]),
        ),
    ]))
    .is_ok());

    let _ = color_from_graphics_value(&rgba(0.1, 0.2, 0.3, 0.0));
}
