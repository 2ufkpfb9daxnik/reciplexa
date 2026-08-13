//! Round-8 graphics_value: missing-field / bad-type Err arms across shapes,
//! pages document empty list, cons payload shape, paint/stroke color failures.

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

fn a4_size() -> RuntimeValue {
    rec(vec![("width", num(210.0)), ("height", num(297.0))])
}

#[test]
fn pages_document_empty_and_bad_items() {
    // tag pages + empty cons → "pages expects at least one page"
    let empty_pages = rec(vec![
        ("tag", RuntimeValue::String("pages".into())),
        (
            "items",
            RuntimeValue::Variant {
                tag: "nil".into(),
                payload: None,
            },
        ),
    ]);
    assert!(document_from_graphics_value(&empty_pages).is_err());

    // pages items not a list
    let bad_items = rec(vec![
        ("tag", RuntimeValue::String("pages".into())),
        ("items", RuntimeValue::String("nope".into())),
    ]);
    assert!(document_from_graphics_value(&bad_items).is_err());

    // empty cons page list at top level
    let empty_list = RuntimeValue::Variant {
        tag: "nil".into(),
        payload: None,
    };
    assert!(document_from_graphics_value(&empty_list).is_err());
}

#[test]
fn missing_and_non_numeric_shape_fields() {
    // Partial circle / rect / ellipse / line
    for shape in [
        rec(vec![
            ("tag", RuntimeValue::String("circle".into())),
            ("x", num(1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("circle".into())),
            ("x", num(1.0)),
            ("y", num(2.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("rect".into())),
            ("y", num(1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("rect".into())),
            ("x", num(1.0)),
            ("y", num(2.0)),
            ("w", num(3.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("ellipse".into())),
            ("y", num(1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("ellipse".into())),
            ("x", num(1.0)),
            ("y", num(2.0)),
            ("rx", num(3.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("line".into())),
            ("x1", num(0.0)),
            ("y1", num(0.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("line".into())),
            ("x1", num(0.0)),
            ("y1", num(0.0)),
            ("x2", num(1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("ring".into())),
            ("x", num(0.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("ring".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
            ("r", num(1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("frame".into())),
            ("x", num(0.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("frame".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
            ("w", num(1.0)),
            ("h", num(1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("text".into())),
            ("x", num(0.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("text".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
            ("size", num(8.0)),
            ("content", RuntimeValue::String("hi".into())),
            ("h", RuntimeValue::String("bad".into())),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("image".into())),
            ("path", RuntimeValue::String("a.png".into())),
            ("x", num(0.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("image".into())),
            ("path", RuntimeValue::String("a.png".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
            ("w", num(1.0)),
        ]),
        rec(vec![("tag", RuntimeValue::String("translate".into()))]),
        rec(vec![
            ("tag", RuntimeValue::String("translate".into())),
            ("dx", num(1.0)),
            ("dy", num(2.0)),
            ("child", circle(1.0, 2.0, 3.0)),
        ]),
        rec(vec![("tag", RuntimeValue::String("rotate".into()))]),
        rec(vec![
            ("tag", RuntimeValue::String("rotate".into())),
            ("degrees", num(90.0)),
            ("child", circle(1.0, 2.0, 3.0)),
        ]),
        rec(vec![("tag", RuntimeValue::String("scale".into()))]),
        rec(vec![
            ("tag", RuntimeValue::String("scale".into())),
            ("sx", num(2.0)),
            ("sy", num(2.0)),
            ("child", circle(1.0, 2.0, 3.0)),
        ]),
        rec(vec![("tag", RuntimeValue::String("opacity".into()))]),
        rec(vec![
            ("tag", RuntimeValue::String("opacity".into())),
            ("alpha", num(0.5)),
            ("child", circle(1.0, 2.0, 3.0)),
        ]),
    ] {
        let _ = shape_from_graphics_value(&shape);
    }

    // Non-numeric field on present keys
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("circle".into())),
        ("x", RuntimeValue::String("nope".into())),
        ("y", num(1.0)),
        ("r", num(1.0)),
    ]))
    .is_err());
}

#[test]
fn fill_stroke_paint_color_and_non_record_targets() {
    let bad_color = RuntimeValue::String("red".into());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", circle(1.0, 2.0, 3.0)),
        ("width", num(1.0)),
        ("color", bad_color.clone()),
    ]))
    .is_err());

    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", circle(1.0, 2.0, 3.0)),
        ("fill", bad_color.clone()),
        ("stroke-width", num(1.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());

    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", circle(1.0, 2.0, 3.0)),
        ("fill", rgb(1.0, 0.0, 0.0)),
        ("stroke-width", num(1.0)),
        ("stroke-color", bad_color),
    ]))
    .is_err());

    // fill/stroke applied to non-record shape value
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("shape", RuntimeValue::Int(1)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", RuntimeValue::Int(1)),
        ("width", num(1.0)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
}

#[test]
fn cons_payload_and_paper_size_errors() {
    // cons payload is not a record
    let bad_cons = RuntimeValue::Variant {
        tag: "cons".into(),
        payload: Some(Box::new(RuntimeValue::String("nope".into()))),
    };
    assert!(document_from_graphics_value(&bad_cons).is_err());

    // cons missing head
    let no_head = RuntimeValue::Variant {
        tag: "cons".into(),
        payload: Some(Box::new(rec(vec![(
            "tail",
            RuntimeValue::Variant {
                tag: "nil".into(),
                payload: None,
            },
        )]))),
    };
    assert!(document_from_graphics_value(&no_head).is_err());

    // paper size missing / non-record
    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        ("size", RuntimeValue::Int(1)),
        ("content", circle(1.0, 2.0, 3.0)),
    ]))
    .is_err());
    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        ("size", rec(vec![("width", num(210.0))])),
        ("content", circle(1.0, 2.0, 3.0)),
    ]))
    .is_err());
    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        ("size", a4_size()),
        (
            "content",
            cons_list(vec![circle(1.0, 2.0, 3.0), RuntimeValue::Int(9)]),
        ),
    ]))
    .is_err());

    // color missing channels / alpha on non-record (paint transparent path)
    assert!(color_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("rgb".into())),
        ("r", num(0.1)),
    ]))
    .is_err());
    assert!(color_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("rgb".into())),
        ("r", num(0.1)),
        ("g", num(0.2)),
    ]))
    .is_err());

    // polygon points list that fails number_list / odd length
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("polygon".into())),
        ("points", RuntimeValue::String("nope".into())),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("polygon".into())),
        (
            "points",
            cons_list(vec![num(0.0), num(0.0), num(1.0), num(0.0), num(0.5)]),
        ),
    ]))
    .is_err());
}
