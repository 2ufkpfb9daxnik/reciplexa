//! Residual math_value `?` / class / layout / metric-record coverage.

use reciplexa_eval::{
    estimate_math_box_from_value, estimate_math_box_from_value_with_style,
    estimate_style_from_value, layout_math_to_shapes, math_atom_from_value, RuntimeValue,
};
use reciplexa_std::math::EstimateStyle;

fn rec(fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
    RuntimeValue::Record(
        fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

fn cons_list(items: Vec<RuntimeValue>) -> RuntimeValue {
    let mut cur = RuntimeValue::Variant {
        tag: "nil".into(),
        payload: None,
    };
    for item in items.into_iter().rev() {
        cur = RuntimeValue::Variant {
            tag: "cons".into(),
            payload: Some(Box::new(rec(vec![("head", item), ("tail", cur)]))),
        };
    }
    cur
}

fn sym(glyph: &str, class: &str) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("math-symbol".into())),
        ("glyph", RuntimeValue::String(glyph.into())),
        ("class", RuntimeValue::String(class.into())),
    ])
}

#[test]
fn missing_fields_and_class_matrix() {
    for v in [
        rec(vec![("tag", RuntimeValue::String("math-symbol".into()))]),
        rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::Int(1)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("x".into())),
            ("class", RuntimeValue::Int(1)),
        ]),
        rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("x".into())),
            ("class", RuntimeValue::String("nope".into())),
        ]),
        rec(vec![(
            "tag",
            RuntimeValue::String("math-operatorname".into()),
        )]),
        rec(vec![("tag", RuntimeValue::String("math-text".into()))]),
        rec(vec![("tag", RuntimeValue::String("math-row".into()))]),
        rec(vec![("tag", RuntimeValue::String("math-fraction".into()))]),
        rec(vec![
            ("tag", RuntimeValue::String("math-fraction".into())),
            ("numerator", sym("1", "ord")),
        ]),
        rec(vec![("tag", RuntimeValue::String("math-scripts".into()))]),
        rec(vec![("tag", RuntimeValue::String("math-radical".into()))]),
        rec(vec![("tag", RuntimeValue::String("math-delimiter".into()))]),
        rec(vec![("tag", RuntimeValue::String("math-accent".into()))]),
        rec(vec![("tag", RuntimeValue::String("math-bigop".into()))]),
        rec(vec![("tag", RuntimeValue::String("math-stack".into()))]),
        rec(vec![("tag", RuntimeValue::String("math-cases".into()))]),
        rec(vec![("tag", RuntimeValue::String("math-align".into()))]),
        rec(vec![("tag", RuntimeValue::String("math-align-at".into()))]),
        rec(vec![]),
        RuntimeValue::Unit,
    ] {
        assert!(math_atom_from_value(&v).is_err(), "{v:?}");
    }

    for class in ["ord", "op", "bin", "rel", "open", "close", "punct", "fence"] {
        assert!(math_atom_from_value(&sym("x", class)).is_ok(), "{class}");
    }
    assert!(math_atom_from_value(&rec(vec![
        ("tag", RuntimeValue::String("math-symbol".into())),
        ("glyph", RuntimeValue::String("x".into())),
    ]))
    .is_ok());
}

#[test]
fn textop_text_align_and_layout_no_ink() {
    let textop = rec(vec![
        ("tag", RuntimeValue::String("math-textop".into())),
        ("glyph", RuntimeValue::String("lim".into())),
        ("class", RuntimeValue::String("op".into())),
    ]);
    assert!(math_atom_from_value(&textop).is_ok());

    let text = rec(vec![
        ("tag", RuntimeValue::String("math-text".into())),
        ("body", RuntimeValue::String("hello".into())),
    ]);
    assert!(math_atom_from_value(&text).is_ok());

    let align_body = rec(vec![
        ("tag", RuntimeValue::String("math-aligned".into())),
        (
            "rows",
            cons_list(vec![rec(vec![
                ("tag", RuntimeValue::String("math-align-row".into())),
                ("cells", cons_list(vec![sym("x", "ord")])),
            ])]),
        ),
    ]);
    let align = rec(vec![
        ("tag", RuntimeValue::String("math-align".into())),
        ("body", align_body.clone()),
    ]);
    assert!(math_atom_from_value(&align).is_ok());

    let align_at = rec(vec![
        ("tag", RuntimeValue::String("math-align-at".into())),
        ("body", align_body),
    ]);
    assert!(math_atom_from_value(&align_at).is_ok());

    let phantom = rec(vec![
        ("tag", RuntimeValue::String("math-phantom".into())),
        ("width", num_like(1.0)),
        ("height", num_like(2.0)),
        ("depth", num_like(0.5)),
    ]);
    assert!(
        layout_math_to_shapes(&phantom, (0.0, 0.0), EstimateStyle::Display)
            .unwrap()
            .is_empty()
    );
    let smash = rec(vec![
        ("tag", RuntimeValue::String("math-smash".into())),
        ("width", RuntimeValue::Int(1)),
        ("height", RuntimeValue::Number(2.0)),
        ("depth", RuntimeValue::F64(0.0)),
    ]);
    assert!(
        layout_math_to_shapes(&smash, (0.0, 0.0), EstimateStyle::Text)
            .unwrap()
            .is_empty()
    );
    assert!(estimate_math_box_from_value(&phantom).is_ok());
    assert!(estimate_math_box_from_value_with_style(&smash, EstimateStyle::Text).is_ok());

    assert_eq!(
        estimate_style_from_value(&RuntimeValue::Unit),
        EstimateStyle::Display
    );
    assert_eq!(
        estimate_style_from_value(&rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("style", RuntimeValue::String("text".into())),
            ("glyph", RuntimeValue::String("x".into())),
        ])),
        EstimateStyle::Text
    );
    assert_eq!(
        estimate_style_from_value(&rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("style", RuntimeValue::ShapeTag("display".into())),
        ])),
        EstimateStyle::Display
    );

    let shapes =
        layout_math_to_shapes(&sym("x", "ord"), (10.0, 20.0), EstimateStyle::Display).unwrap();
    assert!(!shapes.is_empty());
}

fn num_like(n: f64) -> RuntimeValue {
    RuntimeValue::F64(n)
}

#[test]
fn row_skips_phantom_children() {
    let row = rec(vec![
        ("tag", RuntimeValue::String("math-row".into())),
        (
            "children",
            cons_list(vec![
                rec(vec![("tag", RuntimeValue::String("math-phantom".into()))]),
                sym("+", "bin"),
            ]),
        ),
    ]);
    assert!(math_atom_from_value(&row).is_ok());
    assert!(estimate_math_box_from_value(&row).unwrap().width > 0.0);
}
