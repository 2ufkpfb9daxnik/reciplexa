//! Round-6 graphics_value: wrap-em text, empty pages, remaining `?` ends.

use reciplexa_eval::graphics_value::{document_from_graphics_value, shape_from_graphics_value};
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

fn circle(x: f64, y: f64, r: f64) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("circle".into())),
        ("x", num(x)),
        ("y", num(y)),
        ("r", num(r)),
    ])
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
fn empty_pages_list_and_stroke_rect_missing_h() {
    let empty = rec(vec![
        ("tag", RuntimeValue::String("pages".into())),
        ("items", cons_list(vec![])),
    ]);
    assert!(document_from_graphics_value(&empty).is_err());

    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        (
            "shape",
            rec(vec![
                ("tag", RuntimeValue::String("rect".into())),
                ("x", num(0.0)),
                ("y", num(0.0)),
                ("w", num(10.0)),
            ]),
        ),
        ("width", num(1.0)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
}

#[test]
fn polygon_undrawable_fill_and_text_wrap() {
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        (
            "shape",
            rec(vec![
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
            ]),
        ),
        ("color", rgb(2.0, 0.0, 0.0)),
    ]))
    .is_err());

    let wrapped = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("text".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("size", num(4.0)),
        ("content", RuntimeValue::String("abcdefghijklmnop".into())),
        ("wrap-em", num(4.0)),
        ("leading", num(5.0)),
    ]))
    .unwrap();
    assert!(matches!(wrapped, Shape::Group { .. } | Shape::Text(_)));

    let newlines = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("text".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("size", num(4.0)),
        ("content", RuntimeValue::String("a\nb\nc".into())),
    ]))
    .unwrap();
    assert!(matches!(newlines, Shape::Group { .. } | Shape::Text(_)));

    let wrap_and_nl = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("text".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("size", num(4.0)),
        (
            "content",
            RuntimeValue::String("hello world\n\nnext".into()),
        ),
        ("wrap-em", num(3.0)),
        ("leading", num(6.0)),
    ]))
    .unwrap();
    assert!(matches!(wrap_and_nl, Shape::Group { .. } | Shape::Text(_)));
}

#[test]
fn paint_transparent_and_doc_page_via_document() {
    let c = circle(2.0, 2.0, 2.0);
    let transparent = rec(vec![
        ("tag", RuntimeValue::String("rgba".into())),
        ("r", num(1.0)),
        ("g", num(0.0)),
        ("b", num(0.0)),
        ("a", num(0.0)),
    ]);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", c),
        ("fill", transparent),
        ("stroke-width", num(0.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_ok());

    let p = page(circle(1.0, 2.0, 3.0));
    assert!(document_from_graphics_value(&p).is_ok());
}
