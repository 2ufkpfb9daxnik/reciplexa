//! Direct Native v2 runtime for `document/page` record constructors (DN2-5).

use crate::domain_native::{DocumentPageOp, DomainNativeOp};
use crate::domain_native_failure::{take1, take2, take3, take4};
use crate::value::RuntimeValue;
use crate::EvalError;

pub fn call_document_constructor(
    op: DomainNativeOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        DomainNativeOp::DocumentPage(sub) => call_document_page(sub, args),
        other => Err(EvalError {
            message: format!("not a document/page constructor op: {other:?}"),
        }),
    }
}

fn call_document_page(
    op: DocumentPageOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        DocumentPageOp::A4 => size_record(210, 297),
        DocumentPageOp::Letter => size_record_f64(215.9, 279.4),
        DocumentPageOp::A5 => size_record(148, 210),
        DocumentPageOp::A3 => size_record(297, 420),
        DocumentPageOp::Legal => size_record_f64(215.9, 355.6),
        DocumentPageOp::Page => {
            let [paper, flow] = take2(args, "`document/page page`")?;
            Ok(tagged(
                "doc-page",
                vec![("paper", paper.clone()), ("flow", flow.clone())],
            ))
        }
        DocumentPageOp::PageFramed => {
            let [paper, margins, flow] = take3(args, "`document/page page-framed`")?;
            Ok(tagged(
                "doc-page",
                vec![
                    ("paper", paper.clone()),
                    ("margins", margins.clone()),
                    ("flow", flow.clone()),
                ],
            ))
        }
        DocumentPageOp::Margins => {
            let [top, right, bottom, left] = take4(args, "`document/page margins`")?;
            Ok(tagged(
                "doc-margins",
                vec![
                    ("top", top.clone()),
                    ("right", right.clone()),
                    ("bottom", bottom.clone()),
                    ("left", left.clone()),
                ],
            ))
        }
        DocumentPageOp::PageBreak => Ok(tagged("doc-pagebreak", vec![])),
        DocumentPageOp::BlockPageBreak => Ok(record(vec![
            ("tag".into(), str_val("doc-block")),
            ("kind".into(), str_val("pagebreak")),
        ])),
        DocumentPageOp::Flow => {
            let [sections] = take1(args, "`document/page flow`")?;
            Ok(tagged("doc-flow", vec![("sections", sections.clone())]))
        }
        DocumentPageOp::Section => {
            let [title, blocks] = take2(args, "`document/page section`")?;
            Ok(tagged(
                "doc-section",
                vec![("title", title.clone()), ("blocks", blocks.clone())],
            ))
        }
        DocumentPageOp::Heading => {
            let [level, text] = take2(args, "`document/page heading`")?;
            Ok(tagged(
                "doc-heading",
                vec![("level", level.clone()), ("text", text.clone())],
            ))
        }
        DocumentPageOp::Paragraph => {
            let [text] = take1(args, "`document/page paragraph`")?;
            Ok(tagged("doc-paragraph", vec![("text", text.clone())]))
        }
        DocumentPageOp::ParagraphIndented => {
            let [text, em] = take2(args, "`document/page paragraph-indented`")?;
            Ok(record(vec![
                ("tag".into(), str_val("doc-paragraph")),
                ("text".into(), text.clone()),
                ("indent-em".into(), em.clone()),
            ]))
        }
        DocumentPageOp::Columns => {
            let [count, gutter_em, total_em, paragraphs] = take4(args, "`document/page columns`")?;
            Ok(record(vec![
                ("tag".into(), str_val("doc-columns")),
                ("count".into(), count.clone()),
                ("gutter-em".into(), gutter_em.clone()),
                ("total-em".into(), total_em.clone()),
                ("paragraphs".into(), paragraphs.clone()),
            ]))
        }
        DocumentPageOp::UnorderedList => {
            let [items] = take1(args, "`document/page unordered-list`")?;
            Ok(record(vec![
                ("tag".into(), str_val("doc-list")),
                ("ordered".into(), bool_val(false)),
                ("items".into(), items.clone()),
            ]))
        }
        DocumentPageOp::OrderedList => {
            let [items] = take1(args, "`document/page ordered-list`")?;
            Ok(record(vec![
                ("tag".into(), str_val("doc-list")),
                ("ordered".into(), bool_val(true)),
                ("items".into(), items.clone()),
            ]))
        }
        DocumentPageOp::ListItem => {
            let [paragraphs] = take1(args, "`document/page list-item`")?;
            Ok(tagged(
                "doc-list-item",
                vec![("paragraphs", paragraphs.clone())],
            ))
        }
        DocumentPageOp::Table => {
            let [columns, rows] = take2(args, "`document/page table`")?;
            Ok(tagged(
                "doc-table",
                vec![("columns", columns.clone()), ("rows", rows.clone())],
            ))
        }
        DocumentPageOp::Figure => {
            let [visual, caption] = take2(args, "`document/page figure`")?;
            Ok(tagged(
                "doc-figure",
                vec![("visual", visual.clone()), ("caption", caption.clone())],
            ))
        }
        DocumentPageOp::Note => {
            let [text] = take1(args, "`document/page note`")?;
            Ok(tagged("doc-note", vec![("text", text.clone())]))
        }
        DocumentPageOp::Spacer => {
            let [length] = take1(args, "`document/page spacer`")?;
            Ok(tagged("doc-spacer", vec![("length", length.clone())]))
        }
        DocumentPageOp::BlockHeading => block("heading", "heading", args),
        DocumentPageOp::BlockParagraph => block("paragraph", "paragraph", args),
        DocumentPageOp::BlockColumns => block("columns", "columns", args),
        DocumentPageOp::BlockList => block("list", "list", args),
        DocumentPageOp::BlockTable => block("table", "table", args),
        DocumentPageOp::BlockFigure => block("figure", "figure", args),
        DocumentPageOp::BlockNote => block("note", "note", args),
        DocumentPageOp::BlockSpacer => block("spacer", "spacer", args),
    }
}

fn block(kind: &str, field: &str, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    let [value] = take1(args, &format!("`document/page block-{kind}`"))?;
    Ok(record(vec![
        ("tag".into(), str_val("doc-block")),
        ("kind".into(), str_val(kind)),
        (field.into(), value.clone()),
    ]))
}

fn tagged(tag: &str, fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
    let mut out = vec![("tag".into(), str_val(tag))];
    for (k, v) in fields {
        out.push((k.into(), v));
    }
    record(out)
}

fn size_record(width: i128, height: i128) -> Result<RuntimeValue, EvalError> {
    Ok(record(vec![
        ("width".into(), int_val(width)),
        ("height".into(), int_val(height)),
    ]))
}

fn size_record_f64(width: f64, height: f64) -> Result<RuntimeValue, EvalError> {
    Ok(record(vec![
        ("width".into(), num_val(width)),
        ("height".into(), num_val(height)),
    ]))
}

fn record(fields: Vec<(String, RuntimeValue)>) -> RuntimeValue {
    RuntimeValue::Record(fields)
}

fn str_val(s: &str) -> RuntimeValue {
    RuntimeValue::String(s.into())
}

fn int_val(n: i128) -> RuntimeValue {
    RuntimeValue::Int(n)
}

fn num_val(n: f64) -> RuntimeValue {
    RuntimeValue::Number(n)
}

fn bool_val(b: bool) -> RuntimeValue {
    RuntimeValue::Bool(b)
}
