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
        "columns" => {
            let columns = field(fields, "columns")
                .ok_or_else(|| GraphicsValueError::new("doc-block columns missing columns"))?;
            push_heading_or_paragraph(columns, cursor_y, shapes)
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

/// Soft-wrap budget (em) for document text nodes — ~A4 content width stub.
const DOC_TEXT_MAX_EM: f64 = 40.0;

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
            // Soft-wrap long headings via std `break_line` (same budget as paragraphs).
            push_soft_wrapped_text(&text, size, 0.0, cursor_y, shapes);
            Ok(())
        }
        "doc-paragraph" => {
            let text = string_field(fields, "text")?;
            let size = 4.0;
            let indent_em = field(fields, "indent-em").and_then(as_f64).unwrap_or(0.0);
            push_soft_wrapped_text(&text, size, indent_em, cursor_y, shapes);
            Ok(())
        }
        "doc-columns" => push_doc_columns(fields, cursor_y, shapes),
        other => Err(GraphicsValueError::new(format!(
            "expected doc-heading, doc-paragraph, or doc-columns, got `{other}`"
        ))),
    }
}

/// Place paragraph texts into columns via [`measure_columns`] (Wave 22 stub).
///
/// Record fields: `count`, `gutter-em`, optional `total-em`, `paragraphs` (cons of
/// strings or `doc-paragraph`). One paragraph per column (extras ignored). Not
/// balanced multi-column / `jlreq-multi-column` book layout.
fn push_doc_columns(
    fields: &[(String, RuntimeValue)],
    cursor_y: &mut f64,
    shapes: &mut Vec<Shape>,
) -> Result<(), GraphicsValueError> {
    let count = field(fields, "count")
        .and_then(as_f64)
        .unwrap_or(2.0)
        .max(0.0) as u32;
    let gutter_em = field(fields, "gutter-em").and_then(as_f64).unwrap_or(1.0);
    let total_em = field(fields, "total-em")
        .and_then(as_f64)
        .unwrap_or(DOC_TEXT_MAX_EM);
    let paragraphs = field(fields, "paragraphs")
        .ok_or_else(|| GraphicsValueError::new("doc-columns missing paragraphs"))?;
    let items = cons_items(paragraphs)?;
    let (col_w, xs) = reciplexa_std::japanese::measure_columns(total_em, count, gutter_em);
    let size_mm = 4.0;
    const BASE_X_MM: f64 = 20.0;
    let start_y = *cursor_y;
    let mut min_y = start_y;
    let n = count as usize;
    for (i, item) in items.iter().enumerate().take(n) {
        let text = match item {
            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Record(pf) => {
                expect_tag(pf, "doc-paragraph")?;
                string_field(pf, "text")?
            }
            _ => {
                return Err(GraphicsValueError::new(
                    "doc-columns paragraphs expect string or doc-paragraph",
                ));
            }
        };
        let x_em = xs.get(i).copied().unwrap_or(0.0);
        let budget = if col_w > 0.0 { col_w } else { DOC_TEXT_MAX_EM };
        let lines = reciplexa_std::japanese::break_line(&text, budget);
        let mut col_y = start_y;
        if lines.is_empty() {
            push_text_at(
                String::new(),
                size_mm,
                BASE_X_MM + x_em * size_mm,
                &mut col_y,
                shapes,
            );
            col_y -= size_mm + 3.0;
        } else {
            for line in lines {
                push_text_at(
                    line,
                    size_mm,
                    BASE_X_MM + x_em * size_mm,
                    &mut col_y,
                    shapes,
                );
                col_y -= size_mm + 3.0;
            }
        }
        if col_y < min_y {
            min_y = col_y;
        }
    }
    *cursor_y = min_y;
    Ok(())
}

/// Soft-wrap `text` via [`reciplexa_std::japanese::break_line`] into scene Text shapes.
///
/// Optional `indent_em` offsets the first line's x via [`indent_first_line`]
/// (1em ≈ `size_mm` for this stub).
fn push_soft_wrapped_text(
    text: &str,
    size_mm: f64,
    indent_em: f64,
    cursor_y: &mut f64,
    shapes: &mut Vec<Shape>,
) {
    const BASE_X_MM: f64 = 20.0;
    let lines = reciplexa_std::japanese::break_line(text, DOC_TEXT_MAX_EM);
    if lines.is_empty() {
        push_text_at(String::new(), size_mm, BASE_X_MM, cursor_y, shapes);
        *cursor_y -= size_mm + 3.0;
    } else {
        let indented = reciplexa_std::japanese::indent_first_line(&lines, indent_em);
        for (x_em, line) in indented {
            let x_mm = BASE_X_MM + x_em * size_mm;
            push_text_at(line, size_mm, x_mm, cursor_y, shapes);
            *cursor_y -= size_mm + 3.0;
        }
    }
}

fn push_text_at(
    content: String,
    size_mm: f64,
    x_mm: f64,
    cursor_y: &mut f64,
    shapes: &mut Vec<Shape>,
) {
    shapes.push(Shape::Text(Text {
        x_mm,
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
        RuntimeValue::Number(n) => Some(*n),
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
        assert!(texts.contains(&"Title"));
        assert!(texts.contains(&"Body"));
    }

    #[test]
    fn doc_paragraph_indent_em_offsets_first_text_x() {
        let paragraph = rec(vec![
            ("tag", RuntimeValue::String("doc-paragraph".into())),
            ("text", RuntimeValue::String("字下げ本文".into())),
            ("indent-em", RuntimeValue::Number(1.0)),
        ]);
        let blocks = cons(vec![rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("paragraph".into())),
            ("paragraph", paragraph),
        ])]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            (
                "title",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-heading".into())),
                    ("level", RuntimeValue::Int(2)),
                    ("text", RuntimeValue::String("H".into())),
                ]),
            ),
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
        let texts: Vec<_> = doc.pages[0]
            .shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t),
                _ => None,
            })
            .collect();
        let body = texts
            .iter()
            .find(|t| t.content == "字下げ本文")
            .expect("indented paragraph text");
        // BASE_X 20 + 1em * size 4.0 = 24.0
        assert!((body.x_mm - 24.0).abs() < 1e-9);
        let heading = texts.iter().find(|t| t.content == "H").expect("heading");
        assert!((heading.x_mm - 20.0).abs() < 1e-9);
    }

    #[test]
    fn doc_paragraph_long_text_uses_break_line() {
        // Longer than DOC_TEXT_MAX_EM (40): must emit multiple Text shapes via break_line.
        let long = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん";
        assert!(long.chars().count() > 40);
        let expected = reciplexa_std::japanese::break_line(long, 40.0);
        assert!(expected.len() > 1);

        let paragraph = rec(vec![
            ("tag", RuntimeValue::String("doc-paragraph".into())),
            ("text", RuntimeValue::String(long.into())),
        ]);
        let blocks = cons(vec![rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("paragraph".into())),
            ("paragraph", paragraph),
        ])]);
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
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", flow),
        ]);
        let doc = document_from_doc_value(&page).expect("doc lower");
        let texts: Vec<_> = doc.pages[0]
            .shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t.content.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, expected);
    }

    #[test]
    fn doc_heading_long_text_uses_break_line() {
        let long = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん";
        assert!(long.chars().count() > 40);
        let expected = reciplexa_std::japanese::break_line(long, 40.0);
        assert!(expected.len() > 1);

        let heading = rec(vec![
            ("tag", RuntimeValue::String("doc-heading".into())),
            ("level", RuntimeValue::Int(1)),
            ("text", RuntimeValue::String(long.into())),
        ]);
        let blocks = cons(vec![rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("heading".into())),
            ("heading", heading),
        ])]);
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
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", flow),
        ]);
        let doc = document_from_doc_value(&page).expect("doc lower");
        let texts: Vec<_> = doc.pages[0]
            .shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t.content.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, expected);
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

    #[test]
    fn doc_columns_places_paragraphs_with_measure_columns() {
        let columns = rec(vec![
            ("tag", RuntimeValue::String("doc-columns".into())),
            ("count", RuntimeValue::Int(2)),
            ("gutter-em", RuntimeValue::Number(1.0)),
            ("total-em", RuntimeValue::Number(21.0)),
            (
                "paragraphs",
                cons(vec![
                    RuntimeValue::String("左カラム".into()),
                    rec(vec![
                        ("tag", RuntimeValue::String("doc-paragraph".into())),
                        ("text", RuntimeValue::String("右カラム".into())),
                    ]),
                ]),
            ),
        ]);
        let blocks = cons(vec![rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("columns".into())),
            ("columns", columns),
        ])]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            (
                "title",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-heading".into())),
                    ("level", RuntimeValue::Int(2)),
                    ("text", RuntimeValue::String("Cols".into())),
                ]),
            ),
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
        let doc = document_from_doc_value(&page).expect("doc-columns lower");
        let texts: Vec<_> = doc.pages[0]
            .shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t),
                _ => None,
            })
            .collect();
        assert!(texts.iter().any(|t| t.content == "左カラム"));
        assert!(texts.iter().any(|t| t.content == "右カラム"));
        let left = texts.iter().find(|t| t.content == "左カラム").unwrap();
        let right = texts.iter().find(|t| t.content == "右カラム").unwrap();
        // col_w = (21-1)/2 = 10; xs = [0, 11]; size_mm = 4 → Δx = 44mm
        assert!((right.x_mm - left.x_mm - 11.0 * 4.0).abs() < 1e-6);
        assert!((left.y_mm - right.y_mm).abs() < 1e-9);
    }
}
