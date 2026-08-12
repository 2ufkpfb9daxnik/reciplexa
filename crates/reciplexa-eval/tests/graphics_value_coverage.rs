//! Exhaustive graphics_value bridge coverage (external so llvm-cov attributes to src).

use reciplexa_eval::graphics_value::{
    color_from_graphics_value, document_from_graphics_value, page_from_graphics_value,
    shape_from_graphics_value,
};
use reciplexa_eval::RuntimeValue;
use reciplexa_scene::{Color, PaperSize, Shape};

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

fn rgba(r: f64, g: f64, b: f64, a: f64) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("rgba".into())),
        ("r", num(r)),
        ("g", num(g)),
        ("b", num(b)),
        ("a", num(a)),
    ])
}

fn a4() -> RuntimeValue {
    rec(vec![("width", num(210.0)), ("height", num(297.0))])
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
fn page_and_document_ok_and_err() {
    let page = rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        ("size", a4()),
        ("content", circle(1.0, 2.0, 3.0)),
    ]);
    let doc = document_from_graphics_value(&page).unwrap();
    assert_eq!(doc.pages[0].paper, PaperSize::a4());
    assert!(page_from_graphics_value(&page).is_ok());

    assert!(document_from_graphics_value(&RuntimeValue::Unit).is_err());
    assert!(
        page_from_graphics_value(&rec(vec![("tag", RuntimeValue::String("circle".into()))]))
            .is_err()
    );
    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        ("size", a4()),
    ]))
    .is_err());
    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        ("content", circle(1.0, 2.0, 3.0)),
    ]))
    .is_err());
    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        ("size", rec(vec![("width", num(0.0)), ("height", num(1.0))])),
        ("content", circle(1.0, 2.0, 3.0)),
    ]))
    .is_err());
}

#[test]
fn all_shape_tags_and_paint_paths() {
    let shapes = [
        circle(1.0, 2.0, 3.0),
        rec(vec![
            ("tag", RuntimeValue::String("rect".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
            ("w", num(10.0)),
            ("h", num(5.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("ellipse".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
            ("rx", num(2.0)),
            ("ry", num(3.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("line".into())),
            ("x1", num(0.0)),
            ("y1", num(0.0)),
            ("x2", num(1.0)),
            ("y2", num(1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("polyline".into())),
            (
                "points",
                cons_list(vec![
                    num(0.0),
                    num(0.0),
                    num(1.0),
                    num(1.0),
                    num(2.0),
                    num(0.0),
                ]),
            ),
        ]),
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
        rec(vec![
            ("tag", RuntimeValue::String("ring".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
            ("r", num(5.0)),
            ("width", num(1.0)),
            ("color", rgb(1.0, 0.0, 0.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("frame".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
            ("w", num(20.0)),
            ("h", num(10.0)),
            ("width", num(0.5)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("text".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
            ("size", num(8.0)),
            ("content", RuntimeValue::String("hi".into())),
            ("w", num(40.0)),
            ("h", num(12.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("image".into())),
            ("path", RuntimeValue::String("a.png".into())),
            ("x", num(0.0)),
            ("y", num(0.0)),
            ("w", num(10.0)),
            ("h", num(10.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("group".into())),
            ("children", circle(0.0, 0.0, 1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("translate".into())),
            ("dx", num(1.0)),
            ("dy", num(2.0)),
            ("child", circle(0.0, 0.0, 1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("rotate".into())),
            ("degrees", num(45.0)),
            ("child", circle(0.0, 0.0, 1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("scale".into())),
            ("sx", num(2.0)),
            ("sy", num(3.0)),
            ("child", circle(0.0, 0.0, 1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("opacity".into())),
            ("alpha", num(0.25)),
            ("child", circle(0.0, 0.0, 1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("fill".into())),
            ("shape", circle(0.0, 0.0, 1.0)),
            ("color", rgb(0.1, 0.2, 0.3)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("stroke".into())),
            ("shape", circle(0.0, 0.0, 1.0)),
            ("width", num(1.0)),
            ("color", rgb(0.0, 1.0, 0.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("stroke".into())),
            (
                "shape",
                rec(vec![
                    ("tag", RuntimeValue::String("rect".into())),
                    ("x", num(0.0)),
                    ("y", num(0.0)),
                    ("w", num(10.0)),
                    ("h", num(5.0)),
                ]),
            ),
            ("width", num(0.5)),
            ("color", rgb(0.0, 0.0, 1.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("stroke".into())),
            (
                "shape",
                rec(vec![
                    ("tag", RuntimeValue::String("line".into())),
                    ("x1", num(0.0)),
                    ("y1", num(0.0)),
                    ("x2", num(1.0)),
                    ("y2", num(1.0)),
                ]),
            ),
            ("width", num(0.5)),
            ("color", rgb(1.0, 1.0, 0.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("paint".into())),
            ("shape", circle(0.0, 0.0, 2.0)),
            ("fill", rgba(1.0, 0.0, 0.0, 1.0)),
            ("stroke-width", num(1.0)),
            ("stroke-color", rgb(0.0, 0.0, 0.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("paint".into())),
            ("shape", circle(0.0, 0.0, 2.0)),
            ("fill", rgba(1.0, 0.0, 0.0, 0.0)),
            ("stroke-width", num(1.0)),
            ("stroke-color", rgb(0.0, 0.0, 0.0)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("paint".into())),
            ("shape", circle(0.0, 0.0, 2.0)),
            ("fill", rgb(1.0, 0.0, 0.0)),
            ("stroke-width", num(0.0)),
            ("stroke-color", rgb(0.0, 0.0, 0.0)),
        ]),
    ];
    for s in &shapes {
        assert!(shape_from_graphics_value(s).is_ok(), "{s:?}");
    }
}

#[test]
fn color_and_error_matrix() {
    assert_eq!(
        color_from_graphics_value(&rgb(0.2, 0.4, 0.6)).unwrap(),
        Color::new(0.2, 0.4, 0.6)
    );
    assert!(color_from_graphics_value(&rgb(2.0, 0.0, 0.0)).is_err());
    assert!(
        color_from_graphics_value(&rec(vec![("tag", RuntimeValue::String("hsl".into()))])).is_err()
    );
    // ShapeTag tag path
    let tagged = rec(vec![
        ("tag", RuntimeValue::ShapeTag("circle".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("r", num(1.0)),
    ]);
    assert!(matches!(
        shape_from_graphics_value(&tagged).unwrap(),
        Shape::Circle(_)
    ));

    assert!(shape_from_graphics_value(&rec(vec![])).is_err());
    assert!(
        shape_from_graphics_value(&rec(vec![("tag", RuntimeValue::String("hexagon".into()))]))
            .is_err()
    );
    assert!(shape_from_graphics_value(&circle(0.0, 0.0, 0.0)).is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("opacity".into())),
        ("alpha", num(2.0)),
        ("child", circle(0.0, 0.0, 1.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        (
            "shape",
            rec(vec![("tag", RuntimeValue::String("line".into()))])
        ),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
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
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("group".into())),
        (
            "children",
            RuntimeValue::Variant {
                tag: "nil".into(),
                payload: None,
            }
        ),
    ]))
    .is_err());
}
