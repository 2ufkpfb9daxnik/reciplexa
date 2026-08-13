//! Round-9 graphics_value: missing-field / not-drawable residuals (N6o).

use reciplexa_eval::graphics_value::{
    color_from_graphics_value, document_from_graphics_value, page_from_graphics_value,
    shape_from_graphics_value,
};
use reciplexa_eval::RuntimeValue;
use reciplexa_scene::Color;

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

fn circle_ok() -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("circle".into())),
        ("x", num(1.0)),
        ("y", num(2.0)),
        ("r", num(3.0)),
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

#[test]
fn graphics_value_round9_missing_fields_and_not_drawable() {
    // rect missing w
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("rect".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("h", num(1.0)),
    ]))
    .is_err());
    // ring missing r
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("ring".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("width", num(1.0)),
    ]))
    .is_err());
    // frame missing w / h / width
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("frame".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("h", num(1.0)),
        ("width", num(1.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("frame".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("w", num(1.0)),
        ("width", num(1.0)),
    ]))
    .is_err());
    // frame Ok with stroke color + stroke-via-paint (stroke_override Some)
    let _ = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("frame".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("w", num(10.0)),
        ("h", num(10.0)),
        ("width", num(1.0)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]));
    let _ = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        (
            "shape",
            rec(vec![
                ("tag", RuntimeValue::String("frame".into())),
                ("x", num(0.0)),
                ("y", num(0.0)),
                ("w", num(10.0)),
                ("h", num(10.0)),
                ("width", num(1.0)),
            ]),
        ),
        ("width", num(2.0)),
        ("color", rgb(1.0, 0.0, 0.0)),
    ]));
    // frame not drawable (zero size)
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("frame".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("w", num(0.0)),
        ("h", num(10.0)),
        ("width", num(1.0)),
    ]))
    .is_err());
    // text missing size
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("text".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("content", RuntimeValue::String("hi".into())),
    ]))
    .is_err());
    // image missing w
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("image".into())),
        ("path", RuntimeValue::String("a.png".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("h", num(10.0)),
    ]))
    .is_err());
    // rotate / scale / opacity with bad child list (shapes_from_list Err)
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("rotate".into())),
        ("degrees", num(90.0)),
        ("child", RuntimeValue::Int(1)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("scale".into())),
        ("sx", num(1.0)),
        ("sy", num(1.0)),
        ("child", RuntimeValue::Int(1)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("opacity".into())),
        ("alpha", num(0.5)),
        ("child", RuntimeValue::Int(1)),
    ]))
    .is_err());
    // Ok rotate/scale/opacity with single child
    let _ = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("rotate".into())),
        ("degrees", num(45.0)),
        ("child", circle_ok()),
    ]));
    let _ = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("scale".into())),
        ("sx", num(2.0)),
        ("sy", num(2.0)),
        ("child", circle_ok()),
    ]));
    let _ = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("opacity".into())),
        ("alpha", num(0.5)),
        ("child", circle_ok()),
    ]));
    // polygon not drawable: invalid fill channels via paint
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
                        num(1.0)
                    ]),
                ),
            ]),
        ),
        (
            "color",
            rec(vec![
                ("tag", RuntimeValue::String("rgb".into())),
                ("r", num(f64::NAN)),
                ("g", num(0.0)),
                ("b", num(0.0)),
            ]),
        ),
    ]))
    .is_err());
    // color_alpha / paint transparent on non-record
    let _ = shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("shape", circle_ok()),
        ("color", RuntimeValue::Int(1)),
    ]));
    // paper size missing width
    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        ("size", rec(vec![("height", num(297.0))])),
        ("content", circle_ok()),
    ]))
    .is_err());
    // document cons with point missing y (shape_from path via point record)
    assert!(document_from_graphics_value(&cons_list(vec![rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        (
            "size",
            rec(vec![("width", num(210.0)), ("height", num(297.0))]),
        ),
        (
            "content",
            rec(vec![
                ("tag", RuntimeValue::String("line".into())),
                ("x1", num(0.0)),
                ("y1", num(0.0)),
                ("x2", num(1.0)),
                // missing y2
            ]),
        ),
    ])]))
    .is_err());
    // color from non-record / alpha path
    assert!(color_from_graphics_value(&RuntimeValue::Int(1)).is_err());
    let _ = color_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("rgba".into())),
        ("r", num(0.1)),
        ("g", num(0.2)),
        ("b", num(0.3)),
        ("a", num(0.4)),
    ]));
    let _ = Color::BLACK;
}
