//! N5.1b — lower package `document/page` (`doc-*`) eval records to scene documents.
//!
//! Flow-oriented constructors from `packages/document` become a simple text layout on
//! paper. Interim CST `(page)/(circle)` keyword tables stay untouched.

use reciplexa_scene::{Color, Document, Page, PaperSize, Shape, Text};

use crate::graphics_value::GraphicsValueError;
use crate::value::RuntimeValue;

/// Lower a `tag: "doc-page"` package value to a scene [`Document`].
pub fn document_from_doc_value(v: &RuntimeValue) -> Result<Document, GraphicsValueError> {
    let page = page_from_doc_value(v)?;
    Ok(Document::single_page(page))
}

/// Lower a `doc-page` record to a scene [`Page`] with a naive top-down text layout.
pub fn page_from_doc_value(v: &RuntimeValue) -> Result<Page, GraphicsValueError> {
    let fields = record_fields(v, "doc-page")?;
    expect_tag(fields, "doc-page")?;
    let paper_v =
        field(fields, "paper").ok_or_else(|| GraphicsValueError::new("doc-page missing paper"))?;
    let flow_v =
        field(fields, "flow").ok_or_else(|| GraphicsValueError::new("doc-page missing flow"))?;
    let paper = paper_from_size_value(paper_v)?;
    let mut cursor_y = paper.height_mm - 25.0;
    let mut shapes = Vec::new();
    collect_flow_shapes(flow_v, &mut cursor_y, &mut shapes)?;
    Ok(Page { paper, shapes })
}

fn collect_flow_shapes(
    v: &RuntimeValue,
    cursor_y: &mut f64,
    shapes: &mut Vec<Shape>,
) -> Result<(), GraphicsValueError> {
    let fields = record_fields(v, "doc-flow")?;
    expect_tag(fields, "doc-flow")?;
    let sections = field(fields, "sections")
        .ok_or_else(|| GraphicsValueError::new("doc-flow missing sections"))?;
    for section in cons_items(sections)? {
        collect_section_shapes(section, cursor_y, shapes)?;
    }
    Ok(())
}

fn collect_section_shapes(
    v: &RuntimeValue,
    cursor_y: &mut f64,
    shapes: &mut Vec<Shape>,
) -> Result<(), GraphicsValueError> {
    let fields = record_fields(v, "doc-section")?;
    expect_tag(fields, "doc-section")?;
    if let Some(title) = field(fields, "title") {
        push_heading_or_paragraph(title, cursor_y, shapes)?;
    }
    let blocks = field(fields, "blocks")
        .ok_or_else(|| GraphicsValueError::new("doc-section missing blocks"))?;
    for block in cons_items(blocks)? {
        collect_block_shapes(block, cursor_y, shapes)?;
    }
    Ok(())
}

fn collect_block_shapes(
    v: &RuntimeValue,
    cursor_y: &mut f64,
    shapes: &mut Vec<Shape>,
) -> Result<(), GraphicsValueError> {
    let fields = record_fields(v, "doc-block")?;
    expect_tag(fields, "doc-block")?;
    let kind = match field(fields, "kind") {
        Some(RuntimeValue::String(s)) => s.as_str(),
        _ => {
            return Err(GraphicsValueError::new("doc-block missing kind string"));
        }
    };
    match kind {
        "heading" => {
            let heading = field(fields, "heading")
                .ok_or_else(|| GraphicsValueError::new("doc-block heading missing heading"))?;
            push_heading_or_paragraph(heading, cursor_y, shapes)
        }
        "paragraph" => {
            let paragraph = field(fields, "paragraph")
                .ok_or_else(|| GraphicsValueError::new("doc-block paragraph missing paragraph"))?;
            push_heading_or_paragraph(paragraph, cursor_y, shapes)
        }
        "spacer" => {
            let spacer = field(fields, "spacer")
                .ok_or_else(|| GraphicsValueError::new("doc-block spacer missing spacer"))?;
            let spacer_fields = record_fields(spacer, "doc-spacer")?;
            expect_tag(spacer_fields, "doc-spacer")?;
            let len = field(spacer_fields, "length")
                .and_then(as_f64)
                .unwrap_or(4.0);
            *cursor_y -= len;
            Ok(())
        }
        "list" | "table" | "figure" => {
            // Stub: skip visual for now; keep structure walkable without failing.
            Ok(())
        }
        other => Err(GraphicsValueError::new(format!(
            "unsupported doc-block kind `{other}`"
        ))),
    }
}

fn push_heading_or_paragraph(
    v: &RuntimeValue,
    cursor_y: &mut f64,
    shapes: &mut Vec<Shape>,
) -> Result<(), GraphicsValueError> {
    let fields = record_fields(v, "doc text node")?;
    let tag = tag_of(fields).unwrap_or("");
    match tag {
        "doc-heading" => {
            let level = field(fields, "level").and_then(as_f64).unwrap_or(1.0);
            let text = string_field(fields, "text")?;
            let size = match level as i64 {
                1 => 8.0,
                2 => 6.5,
                _ => 5.5,
            };
            push_text(text, size, cursor_y, shapes);
            *cursor_y -= size + 3.0;
            Ok(())
        }
        "doc-paragraph" => {
            let text = string_field(fields, "text")?;
            push_text(text, 4.0, cursor_y, shapes);
            *cursor_y -= 7.0;
            Ok(())
        }
        other => Err(GraphicsValueError::new(format!(
            "expected doc-heading or doc-paragraph, got `{other}`"
        ))),
    }
}

fn push_text(content: String, size_mm: f64, cursor_y: &mut f64, shapes: &mut Vec<Shape>) {
    shapes.push(Shape::Text(Text {
        x_mm: 20.0,
        y_mm: *cursor_y,
        size_mm,
        width_mm: None,
        height_mm: None,
        content,
        fill: Color::BLACK,
    }));
}

fn paper_from_size_value(v: &RuntimeValue) -> Result<PaperSize, GraphicsValueError> {
    let fields = record_fields(v, "paper size")?;
    let width_mm = number_field(fields, "width")?;
    let height_mm = number_field(fields, "height")?;
    let paper = PaperSize {
        width_mm,
        height_mm,
    };
    if !paper.is_positive() {
        return Err(GraphicsValueError::new("paper size must be positive"));
    }
    Ok(paper)
}

fn record_fields<'a>(
    v: &'a RuntimeValue,
    ctx: &str,
) -> Result<&'a [(String, RuntimeValue)], GraphicsValueError> {
    match v {
        RuntimeValue::Record(fields) => Ok(fields.as_slice()),
        _ => Err(GraphicsValueError::new(format!(
            "{ctx}: expected record, got non-record"
        ))),
    }
}

fn field<'a>(fields: &'a [(String, RuntimeValue)], name: &str) -> Option<&'a RuntimeValue> {
    fields.iter().find(|(k, _)| k == name).map(|(_, v)| v)
}

fn tag_of(fields: &[(String, RuntimeValue)]) -> Option<&str> {
    match field(fields, "tag")? {
        RuntimeValue::String(s) => Some(s.as_str()),
        RuntimeValue::ShapeTag(s) => Some(s.as_str()),
        _ => None,
    }
}

fn expect_tag(fields: &[(String, RuntimeValue)], want: &str) -> Result<(), GraphicsValueError> {
    match tag_of(fields) {
        Some(t) if t == want => Ok(()),
        Some(t) => Err(GraphicsValueError::new(format!(
            "expected tag `{want}`, got `{t}`"
        ))),
        None => Err(GraphicsValueError::new(format!(
            "expected tag `{want}`, missing tag"
        ))),
    }
}

fn string_field(
    fields: &[(String, RuntimeValue)],
    name: &str,
) -> Result<String, GraphicsValueError> {
    match field(fields, name) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        _ => Err(GraphicsValueError::new(format!(
            "missing string field `{name}`"
        ))),
    }
}

fn number_field(fields: &[(String, RuntimeValue)], name: &str) -> Result<f64, GraphicsValueError> {
    field(fields, name)
        .and_then(as_f64)
        .ok_or_else(|| GraphicsValueError::new(format!("missing numeric field `{name}`")))
}

fn as_f64(v: &RuntimeValue) -> Option<f64> {
    match v {
        RuntimeValue::Int(n) => Some(*n as f64),
        RuntimeValue::F64(n) => Some(*n),
        _ => None,
    }
}

fn cons_items(v: &RuntimeValue) -> Result<Vec<&RuntimeValue>, GraphicsValueError> {
    let mut out = Vec::new();
    let mut cur = v;
    loop {
        match cur {
            RuntimeValue::Variant { tag, .. } if tag == "nil" => break,
            RuntimeValue::Variant { tag, payload } if tag == "cons" => {
                let Some(payload) = payload.as_ref() else {
                    return Err(GraphicsValueError::new("cons variant missing payload"));
                };
                let fields = record_fields(payload, "cons")?;
                let head = field(fields, "head")
                    .ok_or_else(|| GraphicsValueError::new("cons missing head"))?;
                let tail = field(fields, "tail")
                    .ok_or_else(|| GraphicsValueError::new("cons missing tail"))?;
                out.push(head);
                cur = tail;
            }
            _ => {
                return Err(GraphicsValueError::new(
                    "expected cons/nil list of document nodes",
                ));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::Shape;

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
    fn lowers_doc_page_heading_and_paragraph() {
        let heading = rec(vec![
            ("tag", RuntimeValue::String("doc-heading".into())),
            ("level", RuntimeValue::Int(1)),
            ("text", RuntimeValue::String("Title".into())),
        ]);
        let paragraph = rec(vec![
            ("tag", RuntimeValue::String("doc-paragraph".into())),
            ("text", RuntimeValue::String("Body".into())),
        ]);
        let spacer = rec(vec![
            ("tag", RuntimeValue::String("doc-spacer".into())),
            ("length", RuntimeValue::Int(4)),
        ]);
        let blocks = cons(vec![
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("heading".into())),
                ("heading", heading.clone()),
            ]),
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("paragraph".into())),
                ("paragraph", paragraph),
            ]),
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("spacer".into())),
                ("spacer", spacer),
            ]),
        ]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            ("title", heading),
            ("blocks", blocks),
        ]);
        let flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![section])),
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
            ("flow", flow),
        ]);
        let doc = document_from_doc_value(&page).expect("doc lower");
        assert_eq!(doc.pages.len(), 1);
        assert_eq!(doc.pages[0].paper.width_mm, 210.0);
        let texts: Vec<_> = doc.pages[0]
            .shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t.content.as_str()),
                _ => None,
            })
            .collect();
        assert!(texts.iter().any(|t| *t == "Title"));
        assert!(texts.iter().any(|t| *t == "Body"));
    }

    #[test]
    fn rejects_non_doc_page() {
        let v = rec(vec![("tag", RuntimeValue::String("page".into()))]);
        assert!(document_from_doc_value(&v).is_err());
    }

    #[test]
    fn n5_3_tips_block_stubs_levels_and_error_arms() {
        // list/table/figure stubs (no visual)
        for kind in ["list", "table", "figure"] {
            let block = rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String(kind.into())),
            ]);
            let section = rec(vec![
                ("tag", RuntimeValue::String("doc-section".into())),
                ("blocks", cons(vec![block])),
            ]);
            let flow = rec(vec![
                ("tag", RuntimeValue::String("doc-flow".into())),
                ("sections", cons(vec![section])),
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
                ("flow", flow),
            ]);
            let doc = document_from_doc_value(&page).expect(kind);
            assert!(doc.pages[0].shapes.is_empty(), "{kind} stub draws nothing");
        }

        // heading levels 2 / default + missing kind / unknown kind / bad text node
        let h2 = rec(vec![
            ("tag", RuntimeValue::String("doc-heading".into())),
            ("level", RuntimeValue::Int(2)),
            ("text", RuntimeValue::String("H2".into())),
        ]);
        let h3 = rec(vec![
            ("tag", RuntimeValue::String("doc-heading".into())),
            ("level", RuntimeValue::Int(9)),
            ("text", RuntimeValue::String("Hn".into())),
        ]);
        let blocks = cons(vec![
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("heading".into())),
                ("heading", h2),
            ]),
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("heading".into())),
                ("heading", h3),
            ]),
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("spacer".into())),
                (
                    "spacer",
                    rec(vec![
                        ("tag", RuntimeValue::String("doc-spacer".into())),
                        // missing length → default 4.0
                    ]),
                ),
            ]),
        ]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            ("blocks", blocks),
        ]);
        let flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![section])),
        ]);
        let page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::F64(100.0)),
                    ("height", RuntimeValue::F64(200.0)),
                ]),
            ),
            ("flow", flow),
        ]);
        let doc = document_from_doc_value(&page).expect("levels");
        assert!(doc.pages[0]
            .shapes
            .iter()
            .any(|s| matches!(s, Shape::Text(t) if t.content == "H2")));

        assert!(document_from_doc_value(&rec(vec![(
            "tag",
            RuntimeValue::String("doc-page".into())
        ),]))
        .is_err());
        let bad_block = rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::Int(1)),
        ]);
        let bad_section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            ("blocks", cons(vec![bad_block])),
        ]);
        let bad_flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![bad_section])),
        ]);
        let bad_page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", bad_flow),
        ]);
        assert!(document_from_doc_value(&bad_page).is_err());

        let unk = rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("mystery".into())),
        ]);
        let unk_section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            ("blocks", cons(vec![unk])),
        ]);
        let unk_flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![unk_section])),
        ]);
        let unk_page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", unk_flow),
        ]);
        assert!(document_from_doc_value(&unk_page)
            .unwrap_err()
            .message
            .contains("unsupported"));

        // non-positive paper
        let zero_paper = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(0)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            (
                "flow",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-flow".into())),
                    ("sections", cons(vec![])),
                ]),
            ),
        ]);
        assert!(document_from_doc_value(&zero_paper).is_err());
    }
}
