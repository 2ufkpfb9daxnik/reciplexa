//! document_value missing-field / stub-block residual coverage.

use reciplexa_eval::{document_from_doc_value, page_from_doc_value, RuntimeValue};

fn rec(fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
    RuntimeValue::Record(
        fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
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

fn a4() -> RuntimeValue {
    rec(vec![
        ("width", RuntimeValue::F64(210.0)),
        ("height", RuntimeValue::F64(297.0)),
    ])
}

fn para(text: &str) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("doc-paragraph".into())),
        ("text", RuntimeValue::String(text.into())),
    ])
}

fn heading(text: &str, level: i128) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("doc-heading".into())),
        ("text", RuntimeValue::String(text.into())),
        ("level", RuntimeValue::Int(level)),
    ])
}

fn block(kind: &str, extra: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
    let mut fields = vec![
        ("tag", RuntimeValue::String("doc-block".into())),
        ("kind", RuntimeValue::String(kind.into())),
    ];
    fields.extend(extra);
    rec(fields)
}

fn section(title: Option<RuntimeValue>, blocks: Vec<RuntimeValue>) -> RuntimeValue {
    let mut fields = vec![
        ("tag", RuntimeValue::String("doc-section".into())),
        ("blocks", cons_list(blocks)),
    ];
    if let Some(t) = title {
        fields.push(("title", t));
    }
    rec(fields)
}

fn page(flow: RuntimeValue) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("doc-page".into())),
        ("paper", a4()),
        ("flow", flow),
    ])
}

#[test]
fn doc_page_missing_and_ok_matrix() {
    assert!(page_from_doc_value(&RuntimeValue::Unit).is_err());
    assert!(page_from_doc_value(&rec(vec![("tag", RuntimeValue::String("page".into()))])).is_err());
    assert!(page_from_doc_value(&rec(vec![
        ("tag", RuntimeValue::String("doc-page".into())),
        ("paper", a4()),
    ]))
    .is_err());
    assert!(page_from_doc_value(&rec(vec![
        ("tag", RuntimeValue::String("doc-page".into())),
        (
            "flow",
            rec(vec![("tag", RuntimeValue::String("doc-flow".into()))])
        ),
    ]))
    .is_err());

    let flow = rec(vec![
        ("tag", RuntimeValue::String("doc-flow".into())),
        (
            "sections",
            cons_list(vec![section(
                Some(heading("Title", 1)),
                vec![
                    block("heading", vec![("heading", heading("H2", 2))]),
                    block(
                        "paragraph",
                        vec![("paragraph", para(&"hello world ".repeat(20)))],
                    ),
                    block(
                        "columns",
                        vec![(
                            "columns",
                            rec(vec![
                                ("tag", RuntimeValue::String("doc-columns".into())),
                                (
                                    "paragraphs",
                                    cons_list(vec![
                                        RuntimeValue::String("left".into()),
                                        para("right"),
                                    ]),
                                ),
                                ("count", RuntimeValue::Int(2)),
                                ("gutter-em", RuntimeValue::F64(1.0)),
                                ("total-em", RuntimeValue::F64(20.0)),
                            ]),
                        )],
                    ),
                    block(
                        "spacer",
                        vec![(
                            "spacer",
                            rec(vec![
                                ("tag", RuntimeValue::String("doc-spacer".into())),
                                ("length", RuntimeValue::F64(8.0)),
                            ]),
                        )],
                    ),
                    block("list", vec![]),
                    block("table", vec![]),
                    block("figure", vec![]),
                ],
            )]),
        ),
    ]);
    let doc = document_from_doc_value(&page(flow)).unwrap();
    assert_eq!(doc.pages.len(), 1);
}

#[test]
fn doc_block_error_arms() {
    let mk = |blocks: Vec<RuntimeValue>| {
        rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            ("paper", a4()),
            (
                "flow",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-flow".into())),
                    ("sections", cons_list(vec![section(None, blocks)])),
                ]),
            ),
        ])
    };
    assert!(document_from_doc_value(&mk(vec![rec(vec![(
        "tag",
        RuntimeValue::String("doc-block".into())
    ),])]))
    .is_err());
    assert!(document_from_doc_value(&mk(vec![block("heading", vec![])])).is_err());
    assert!(document_from_doc_value(&mk(vec![block("paragraph", vec![])])).is_err());
    assert!(document_from_doc_value(&mk(vec![block("columns", vec![])])).is_err());
    assert!(document_from_doc_value(&mk(vec![block("weird", vec![])])).is_err());
    assert!(document_from_doc_value(&mk(vec![block(
        "heading",
        vec![(
            "heading",
            rec(vec![("tag", RuntimeValue::String("doc-heading".into()))])
        )],
    )]))
    .is_err());
}
