//! Round-5 graphics_value: `?` Err-region ends on stroke/paint/shape helpers.

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

fn circle(x: f64, y: f64, r: f64) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("circle".into())),
        ("x", num(x)),
        ("y", num(y)),
        ("r", num(r)),
    ])
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("rect".into())),
        ("x", num(x)),
        ("y", num(y)),
        ("w", num(w)),
        ("h", num(h)),
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
fn document_pages_and_cons_happy_paths() {
    let p1 = page(circle(1.0, 2.0, 3.0));
    let p2 = page(circle(4.0, 5.0, 6.0));
    let doc = document_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("pages".into())),
        ("items", cons_list(vec![p1.clone(), p2.clone()])),
    ]))
    .unwrap();
    assert_eq!(doc.pages.len(), 2);

    let via_cons = document_from_graphics_value(&cons_list(vec![p1, p2])).unwrap();
    assert_eq!(via_cons.pages.len(), 2);

    let shapes_page = page(cons_list(vec![
        circle(1.0, 2.0, 3.0),
        circle(0.0, 0.0, 1.0),
    ]));
    assert!(page_from_graphics_value(&shapes_page).is_ok());
}

#[test]
fn stroke_inner_circle_rect_missing_coords() {
    let color = rgb(0.0, 0.0, 0.0);
    for inner in [
        rec(vec![("tag", RuntimeValue::String("circle".into()))]),
        rec(vec![
            ("tag", RuntimeValue::String("circle".into())),
            ("x", num(1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("circle".into())),
            ("x", num(1.0)),
            ("y", num(2.0)),
        ]),
        rec(vec![("tag", RuntimeValue::String("rect".into()))]),
        rec(vec![
            ("tag", RuntimeValue::String("rect".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("rect".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
            ("w", num(1.0)),
        ]),
    ] {
        assert!(shape_from_graphics_value(&rec(vec![
            ("tag", RuntimeValue::String("stroke".into())),
            ("shape", inner),
            ("width", num(1.0)),
            ("color", color.clone()),
        ]))
        .is_err());
    }
}

#[test]
fn stroke_circle_rect_happy_and_non_drawable() {
    let color = rgb(0.0, 0.0, 1.0);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", circle(1.0, 2.0, 3.0)),
        ("width", num(0.5)),
        ("color", color.clone()),
    ]))
    .is_ok());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", rect(0.0, 0.0, 10.0, 5.0)),
        ("width", num(0.5)),
        ("color", color),
    ]))
    .is_ok());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", circle(0.0, 0.0, 0.0)),
        ("width", num(1.0)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
}

#[test]
fn fill_inner_missing_and_happy() {
    let color = rgb(0.2, 0.3, 0.4);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("shape", circle(1.0, 2.0, 3.0)),
        ("color", color.clone()),
    ]))
    .is_ok());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("color", color),
    ]))
    .is_err());
}

#[test]
fn paint_fallback_and_group_children() {
    let c = circle(2.0, 2.0, 2.0);
    let color = rgb(1.0, 0.0, 0.0);
    // Both fill + stroke → group with two children.
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", c.clone()),
        ("fill", rgba(1.0, 0.0, 0.0, 1.0)),
        ("stroke-width", num(1.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_ok());
    // Transparent fill, stroke only via if-let push.
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", c.clone()),
        ("fill", rgba(1.0, 0.0, 0.0, 0.0)),
        ("stroke-width", num(1.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_ok());
    // Fallback fill-only when stroke width zero.
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", c),
        ("fill", color),
        ("stroke-width", num(0.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_ok());
}

#[test]
fn shape_helpers_missing_fields_matrix() {
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("ellipse".into())),
        ("x", num(0.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("line".into())),
        ("x1", num(0.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("text".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("size", num(8.0)),
        ("content", RuntimeValue::String("hi".into())),
        ("w", RuntimeValue::String("bad".into())),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("image".into())),
        ("path", RuntimeValue::String("a.png".into())),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("ring".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("r", num(1.0)),
        ("width", num(1.0)),
        ("color", rgb(2.0, 0.0, 0.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("frame".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("w", num(10.0)),
        ("h", num(5.0)),
        ("width", num(0.5)),
        ("color", rgb(0.0, 2.0, 0.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("translate".into())),
        ("dx", num(1.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("rotate".into())),
        ("degrees", num(45.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("scale".into())),
        ("sx", num(2.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("opacity".into())),
        ("alpha", num(0.5)),
    ]))
    .is_err());
}

#[test]
fn polyline_polygon_cons_and_color_alpha() {
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("polyline".into())),
        ("points", cons_list(vec![num(0.0), num(0.0)])),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("polygon".into())),
        ("points", cons_list(vec![num(0.0), num(0.0), num(1.0)])),
    ]))
    .is_err());
    assert!(
        document_from_graphics_value(&cons_list(vec![RuntimeValue::String("not-a-page".into()),]))
            .is_err()
    );
    assert!(color_from_graphics_value(&rgba(0.5, 0.5, 0.5, 0.5)).is_ok());
}
