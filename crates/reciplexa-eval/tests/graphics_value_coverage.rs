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

fn page(content: RuntimeValue) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        ("size", a4()),
        ("content", content),
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
fn page_fill_circle_matches_scene_geometry() {
    let page = rec(vec![
        ("tag", RuntimeValue::String("page".into())),
        ("size", a4()),
        (
            "content",
            rec(vec![
                ("tag", RuntimeValue::String("fill".into())),
                ("shape", circle(105.0, 148.5, 40.0)),
                ("color", rgb(0.0, 0.0, 0.0)),
            ]),
        ),
    ]);
    let doc = document_from_graphics_value(&page).unwrap();
    assert_eq!(doc.pages.len(), 1);
    assert_eq!(doc.pages[0].paper, PaperSize::a4());
    match &doc.pages[0].shapes[0] {
        Shape::Circle(c) => {
            assert_eq!(c.x_mm, 105.0);
            assert_eq!(c.y_mm, 148.5);
            assert_eq!(c.radius_mm, 40.0);
        }
        other => panic!("expected circle, got {other:?}"),
    }
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

    // fill on rect/ellipse/polygon/text + stroke polyline + paint dual group
    for tag_shape in [
        rec(vec![
            ("tag", RuntimeValue::String("fill".into())),
            (
                "shape",
                rec(vec![
                    ("tag", RuntimeValue::String("rect".into())),
                    ("x", num(0.0)),
                    ("y", num(0.0)),
                    ("w", num(2.0)),
                    ("h", num(3.0)),
                ]),
            ),
            ("color", rgb(0.5, 0.5, 0.5)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("fill".into())),
            (
                "shape",
                rec(vec![
                    ("tag", RuntimeValue::String("ellipse".into())),
                    ("x", num(0.0)),
                    ("y", num(0.0)),
                    ("rx", num(2.0)),
                    ("ry", num(3.0)),
                ]),
            ),
            ("color", rgb(0.5, 0.5, 0.5)),
        ]),
        rec(vec![
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
            ("color", rgb(0.5, 0.5, 0.5)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("fill".into())),
            (
                "shape",
                rec(vec![
                    ("tag", RuntimeValue::String("text".into())),
                    ("x", num(0.0)),
                    ("y", num(0.0)),
                    ("size", num(8.0)),
                    ("content", RuntimeValue::String("x".into())),
                ]),
            ),
            ("color", rgb(0.5, 0.5, 0.5)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("stroke".into())),
            (
                "shape",
                rec(vec![
                    ("tag", RuntimeValue::String("polyline".into())),
                    (
                        "points",
                        cons_list(vec![num(0.0), num(0.0), num(1.0), num(1.0)]),
                    ),
                ]),
            ),
            ("width", num(0.5)),
            ("color", rgb(0.0, 0.0, 0.0)),
        ]),
    ] {
        assert!(
            shape_from_graphics_value(&tag_shape).is_ok(),
            "{tag_shape:?}"
        );
    }

    // paint with both fill+stroke → group
    let paint_both = rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
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
        ("fill", rgba(1.0, 0.0, 0.0, 1.0)),
        ("stroke-width", num(1.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]);
    assert!(matches!(
        shape_from_graphics_value(&paint_both).unwrap(),
        Shape::Group { children, .. } if children.len() == 2
    ));

    // missing fill/stroke fields
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("shape", circle(0.0, 0.0, 1.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", circle(0.0, 0.0, 1.0)),
        ("width", num(1.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", circle(0.0, 0.0, 1.0)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
}

#[test]
fn multipage_document_and_shape_lists() {
    let p1 = page(circle(1.0, 2.0, 3.0));
    let p2 = page(cons_list(vec![
        circle(0.0, 0.0, 1.0),
        circle(2.0, 0.0, 1.0),
    ]));

    let via_cons = document_from_graphics_value(&cons_list(vec![p1.clone(), p2.clone()])).unwrap();
    assert_eq!(via_cons.pages.len(), 2);
    assert_eq!(via_cons.pages[0].shapes.len(), 1);
    assert_eq!(via_cons.pages[1].shapes.len(), 2);

    let via_pages = document_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("pages".into())),
        ("items", cons_list(vec![p1, p2])),
    ]))
    .unwrap();
    assert_eq!(via_pages.pages.len(), 2);

    let multi_shape_page = page_from_graphics_value(&page(cons_list(vec![
        circle(0.0, 0.0, 1.0),
        circle(3.0, 0.0, 1.0),
    ])))
    .unwrap();
    assert_eq!(multi_shape_page.shapes.len(), 2);

    assert!(document_from_graphics_value(&RuntimeValue::Variant {
        tag: "nil".into(),
        payload: None,
    })
    .is_err());
    assert!(document_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("pages".into())),
        (
            "items",
            RuntimeValue::Variant {
                tag: "nil".into(),
                payload: None,
            }
        ),
    ]))
    .is_err());
    assert!(document_from_graphics_value(&rec(vec![(
        "tag",
        RuntimeValue::String("pages".into())
    )]))
    .is_err());
}

#[test]
fn nondrawable_shape_matrix() {
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("rect".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("w", num(0.0)),
        ("h", num(5.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("ellipse".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("rx", num(0.0)),
        ("ry", num(3.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("line".into())),
        ("x1", num(0.0)),
        ("y1", num(0.0)),
        ("x2", num(0.0)),
        ("y2", num(0.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("ring".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("r", num(0.0)),
        ("width", num(1.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("frame".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("w", num(0.0)),
        ("h", num(10.0)),
        ("width", num(0.5)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("text".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("size", num(0.0)),
        ("content", RuntimeValue::String("x".into())),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("image".into())),
        ("path", RuntimeValue::String("a.png".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("w", num(0.0)),
        ("h", num(10.0)),
    ]))
    .is_err());
}

#[test]
fn stroke_paint_and_transform_error_leaves() {
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", circle(0.0, 0.0, 0.0)),
        ("width", num(1.0)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        (
            "shape",
            rec(vec![
                ("tag", RuntimeValue::String("rect".into())),
                ("x", num(0.0)),
                ("y", num(0.0)),
                ("w", num(0.0)),
                ("h", num(5.0)),
            ]),
        ),
        ("width", num(1.0)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
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

    // paint missing fields + fallback when fill/stroke silently fail
    assert!(
        shape_from_graphics_value(&rec(vec![("tag", RuntimeValue::String("paint".into()))]))
            .is_err()
    );
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", circle(0.0, 0.0, 2.0)),
        ("fill", rgb(1.0, 0.0, 0.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
        ("shape", circle(0.0, 0.0, 2.0)),
        ("fill", rgb(1.0, 0.0, 0.0)),
        ("stroke-width", num(1.0)),
    ]))
    .is_err());
    let paint_fallback = rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
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
        ("fill", rgba(1.0, 0.0, 0.0, 0.0)),
        ("stroke-width", num(0.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]);
    assert!(shape_from_graphics_value(&paint_fallback).is_err());

    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("translate".into())),
        ("dx", num(1.0)),
        ("dy", num(2.0)),
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
        ("sy", num(3.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("opacity".into())),
        ("alpha", num(0.5)),
    ]))
    .is_err());
}

#[test]
fn cons_list_and_field_type_errors() {
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("polyline".into())),
        ("points", RuntimeValue::String("bad".into())),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("polyline".into())),
        (
            "points",
            cons_list(vec![RuntimeValue::String("nope".into()), num(0.0)]),
        ),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("group".into())),
        ("children", RuntimeValue::String("bad".into())),
    ]))
    .is_err());

    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::Int(1)),
        ("size", a4()),
        ("content", circle(1.0, 2.0, 3.0)),
    ]))
    .is_err());
    assert!(page_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("not-page".into())),
        ("size", a4()),
        ("content", circle(1.0, 2.0, 3.0)),
    ]))
    .is_err());

    // Int numeric fields + optional bad w/h on text
    let int_circle = rec(vec![
        ("tag", RuntimeValue::String("circle".into())),
        ("x", RuntimeValue::Int(1)),
        ("y", RuntimeValue::Int(2)),
        ("r", RuntimeValue::Int(3)),
    ]);
    assert!(matches!(
        shape_from_graphics_value(&int_circle).unwrap(),
        Shape::Circle(_)
    ));
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("text".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("size", num(8.0)),
        ("content", RuntimeValue::String("x".into())),
        ("w", RuntimeValue::String("bad".into())),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("text".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("size", num(8.0)),
        ("content", RuntimeValue::Int(1)),
    ]))
    .is_err());

    // color without tag (empty string path) + ring optional color err
    assert!(color_from_graphics_value(&rec(vec![
        ("r", num(0.1)),
        ("g", num(0.2)),
        ("b", num(0.3)),
    ]))
    .is_ok());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("ring".into())),
        ("x", num(0.0)),
        ("y", num(0.0)),
        ("r", num(5.0)),
        ("width", num(1.0)),
        (
            "color",
            rec(vec![("tag", RuntimeValue::String("hsl".into()))])
        ),
    ]))
    .is_err());

    // malformed cons + bad page in multipage list
    assert!(document_from_graphics_value(&RuntimeValue::Variant {
        tag: "cons".into(),
        payload: None,
    })
    .is_err());
    let bad_page_list = cons_list(vec![
        page(circle(1.0, 2.0, 3.0)),
        rec(vec![("tag", RuntimeValue::String("not-page".into()))]),
    ]);
    assert!(document_from_graphics_value(&bad_page_list).is_err());

    // paint: fill unsupported but stroke ok on line → stroke child only
    let paint_stroke_only = rec(vec![
        ("tag", RuntimeValue::String("paint".into())),
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
        ("fill", rgb(1.0, 0.0, 0.0)),
        ("stroke-width", num(1.0)),
        ("stroke-color", rgb(0.0, 0.0, 0.0)),
    ]);
    assert!(matches!(
        shape_from_graphics_value(&paint_stroke_only).unwrap(),
        Shape::Line(_)
    ));

    // RuntimeValue::Number coords in polyline
    let num_points = cons_list(vec![
        RuntimeValue::Number(0.0),
        RuntimeValue::Number(0.0),
        RuntimeValue::Number(1.0),
        RuntimeValue::Number(1.0),
    ]);
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("polyline".into())),
        ("points", num_points),
    ]))
    .is_ok());

    // inner shape missing tag for stroke/fill
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("stroke".into())),
        ("shape", rec(vec![("x", num(0.0))])),
        ("width", num(1.0)),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
    assert!(shape_from_graphics_value(&rec(vec![
        ("tag", RuntimeValue::String("fill".into())),
        ("shape", rec(vec![("x", num(0.0))])),
        ("color", rgb(0.0, 0.0, 0.0)),
    ]))
    .is_err());
}
