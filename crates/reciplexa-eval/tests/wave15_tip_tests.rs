//! Wave 15 I3 tip coverage: indent_first_line + doc-paragraph indent-em.

use reciplexa_eval::document_from_doc_value;
use reciplexa_eval::RuntimeValue;
use reciplexa_scene::Shape;
use reciplexa_std::japanese::indent_first_line;

fn rec(fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
    RuntimeValue::Record(
        fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

fn cons(items: Vec<RuntimeValue>) -> RuntimeValue {
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

#[test]
fn tip_wave15_indent_first_line_and_doc_paragraph() {
    let lines = vec!["甲".into(), "乙".into()];
    let indented = indent_first_line(&lines, 1.0);
    assert!((indented[0].0 - 1.0).abs() < 1e-9);
    assert!((indented[1].0 - 0.0).abs() < 1e-9);

    let paragraph = rec(vec![
        ("tag", RuntimeValue::String("doc-paragraph".into())),
        ("text", RuntimeValue::String("甲".into())),
        ("indent-em", RuntimeValue::Int(1)),
    ]);
    let page = rec(vec![
        ("tag", RuntimeValue::String("doc-page".into())),
        (
            "paper",
            rec(vec![
                ("width", RuntimeValue::Int(210)),
                ("height", RuntimeValue::Int(297)),
            ]),
        ),
        (
            "flow",
            rec(vec![
                ("tag", RuntimeValue::String("doc-flow".into())),
                (
                    "sections",
                    cons(vec![rec(vec![
                        ("tag", RuntimeValue::String("doc-section".into())),
                        (
                            "title",
                            rec(vec![
                                ("tag", RuntimeValue::String("doc-heading".into())),
                                ("level", RuntimeValue::Int(1)),
                                ("text", RuntimeValue::String("T".into())),
                            ]),
                        ),
                        (
                            "blocks",
                            cons(vec![rec(vec![
                                ("tag", RuntimeValue::String("doc-block".into())),
                                ("kind", RuntimeValue::String("paragraph".into())),
                                ("paragraph", paragraph),
                            ])]),
                        ),
                    ])]),
                ),
            ]),
        ),
    ]);
    let doc = document_from_doc_value(&page).expect("lower");
    let body = doc.pages[0]
        .shapes
        .iter()
        .find_map(|s| match s {
            Shape::Text(t) if t.content == "甲" => Some(t),
            _ => None,
        })
        .expect("body");
    assert!((body.x_mm - 24.0).abs() < 1e-9);
}
