//! Host consume: `ja-ruby` / `ja-ruby-demo` → font-backed simple ruby GlyphRuns.
//!
//! Product compose matches `live-layout-demo`: nested `page` is laid out, then
//! ruby samples sit below that text so the annotation band is visible.

use reciplexa_eval::{
    document_from_graphics_value, layout_doc_page_to_scene_with_engine, RuntimeValue,
};
use reciplexa_scene::{Color, Document, Page, PaperSize, Shape};
use reciplexa_std::japanese::{Ruby, RubyKind, RUBY_ANNOTATION_SCALE};
use reciplexa_text_layout::ruby::RUBY_PARENT_GAP_EM;
use reciplexa_text_layout::{
    host_product_font, layout_ruby, positioned_ruby_to_shapes, TypesetEngine,
};

use crate::graphics_bridge::GraphicsBridgeError;

const RUBY_ORIGIN_X_MM: f64 = 20.0;
const RUBY_ORIGIN_Y_MM: f64 = 180.0;
const RUBY_BASE_SIZE_MM: f64 = 10.0;
const RUBY_SAMPLE_GAP_MM: f64 = 10.0;
const RUBY_BELOW_DOC_GAP_MM: f64 = 12.0;
const PAGE_RIGHT_MARGIN_MM: f64 = 20.0;

pub fn document_from_ruby_demo_value(
    v: &RuntimeValue,
    engine: TypesetEngine,
) -> Result<Document, GraphicsBridgeError> {
    match engine {
        TypesetEngine::Stub => stub_nested_page(v),
        TypesetEngine::Product => product_page_with_ruby(v),
    }
}

fn product_page_with_ruby(v: &RuntimeValue) -> Result<Document, GraphicsBridgeError> {
    let fields = record_fields(v)?;
    let rubies = rubies_from_demo(fields)?;
    let font = host_product_font().map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
    let mut doc = match fields.iter().find(|(k, _)| k == "page").map(|(_, val)| val) {
        Some(page) => layout_doc_page_to_scene_with_engine(page, TypesetEngine::Product)
            .map_err(GraphicsBridgeError::from)?,
        None => Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: Vec::new(),
        }),
    };
    let (mut x, mut y) = ruby_row_origin(&doc);
    let row_step =
        RUBY_BASE_SIZE_MM * (1.0 + RUBY_ANNOTATION_SCALE + RUBY_PARENT_GAP_EM) + RUBY_SAMPLE_GAP_MM;
    let page_w = doc.pages.first().map(|p| p.paper.width_mm).unwrap_or(210.0);
    let mut shapes = Vec::new();
    for ruby in &rubies {
        let measured = layout_ruby(&font, ruby, 0.0, 0.0, RUBY_BASE_SIZE_MM)
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
        if x > RUBY_ORIGIN_X_MM && x + measured.advance_mm > page_w - PAGE_RIGHT_MARGIN_MM {
            x = RUBY_ORIGIN_X_MM;
            y -= row_step;
        }
        let laid = layout_ruby(&font, ruby, x, y, RUBY_BASE_SIZE_MM)
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
        x += laid.advance_mm + RUBY_SAMPLE_GAP_MM;
        shapes.extend(positioned_ruby_to_shapes(&laid, Color::BLACK));
    }
    if let Some(page) = doc.pages.first_mut() {
        page.shapes.extend(shapes);
    }
    Ok(doc)
}

fn ruby_row_origin(doc: &Document) -> (f64, f64) {
    let Some(page) = doc.pages.first() else {
        return (RUBY_ORIGIN_X_MM, RUBY_ORIGIN_Y_MM);
    };
    let band = RUBY_BASE_SIZE_MM * (1.0 + RUBY_ANNOTATION_SCALE + RUBY_PARENT_GAP_EM);
    let min_y = page
        .shapes
        .iter()
        .filter_map(Shape::text_y_mm)
        .fold(None, |acc: Option<f64>, y| {
            Some(acc.map_or(y, |a| a.min(y)))
        });
    match min_y {
        Some(y) => (RUBY_ORIGIN_X_MM, y - RUBY_BELOW_DOC_GAP_MM - band),
        None => (RUBY_ORIGIN_X_MM, RUBY_ORIGIN_Y_MM),
    }
}

fn stub_nested_page(v: &RuntimeValue) -> Result<Document, GraphicsBridgeError> {
    let fields = record_fields(v)?;
    if tag_of(fields) == Some("ja-ruby") {
        return Err(GraphicsBridgeError::Bridge(
            "bare ja-ruby has no nested page for stub preview".into(),
        ));
    }
    let page = fields
        .iter()
        .find(|(k, _)| k == "page")
        .map(|(_, val)| val)
        .ok_or_else(|| GraphicsBridgeError::Bridge("ja-ruby-demo missing page".into()))?;
    document_from_graphics_value(page).map_err(Into::into)
}

fn rubies_from_demo(fields: &[(String, RuntimeValue)]) -> Result<Vec<Ruby>, GraphicsBridgeError> {
    if tag_of(fields) == Some("ja-ruby") {
        return Ok(vec![ruby_from_fields(fields)?]);
    }
    if let Some(samples) = fields.iter().find(|(k, _)| k == "samples").map(|(_, v)| v) {
        let items = cons_items(samples)?;
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            out.push(ruby_from_demo_or_node(item)?);
        }
        if !out.is_empty() {
            return Ok(out);
        }
    }
    let placed = fields
        .iter()
        .find(|(k, _)| k == "placed")
        .map(|(_, val)| val)
        .ok_or_else(|| GraphicsBridgeError::Bridge("ja-ruby-demo missing placed".into()))?;
    Ok(vec![ruby_from_demo_or_node(placed)?])
}

fn ruby_from_demo_or_node(v: &RuntimeValue) -> Result<Ruby, GraphicsBridgeError> {
    let fields = record_fields(v)?;
    let tag = tag_of(fields);
    if tag == Some("ja-ruby") {
        return ruby_from_fields(fields);
    }
    if tag == Some("ja-ruby-demo") {
        let placed = fields
            .iter()
            .find(|(k, _)| k == "placed")
            .map(|(_, val)| val)
            .ok_or_else(|| GraphicsBridgeError::Bridge("ja-ruby-demo missing placed".into()))?;
        return ruby_from_demo_or_node(placed);
    }
    Err(GraphicsBridgeError::Bridge(format!(
        "expected ja-ruby or ja-ruby-demo, got {:?}",
        tag
    )))
}

fn ruby_from_fields(fields: &[(String, RuntimeValue)]) -> Result<Ruby, GraphicsBridgeError> {
    let base = string_field(fields, "base")?;
    let annotation = string_field(fields, "annotation")?;
    let kind = match string_field(fields, "kind") {
        Ok(s) if s == "jukugo" => RubyKind::Jukugo,
        _ => RubyKind::Simple,
    };
    Ok(Ruby {
        base,
        annotation,
        kind,
    })
}

fn record_fields(v: &RuntimeValue) -> Result<&[(String, RuntimeValue)], GraphicsBridgeError> {
    match v {
        RuntimeValue::Record(f) => Ok(f.as_slice()),
        _ => Err(GraphicsBridgeError::Bridge("ruby record expected".into())),
    }
}

fn tag_of(fields: &[(String, RuntimeValue)]) -> Option<&str> {
    fields.iter().find_map(|(k, v)| {
        if k != "tag" {
            return None;
        }
        match v {
            RuntimeValue::String(s) | RuntimeValue::ShapeTag(s) => Some(s.as_str()),
            _ => None,
        }
    })
}

fn string_field(
    fields: &[(String, RuntimeValue)],
    name: &str,
) -> Result<String, GraphicsBridgeError> {
    match fields.iter().find(|(k, _)| k == name).map(|(_, v)| v) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        _ => Err(GraphicsBridgeError::Bridge(format!(
            "ruby missing string `{name}`"
        ))),
    }
}

fn cons_items(v: &RuntimeValue) -> Result<Vec<&RuntimeValue>, GraphicsBridgeError> {
    let mut out = Vec::new();
    let mut cur = v;
    loop {
        match cur {
            RuntimeValue::Variant { tag, .. } if tag == "nil" => break,
            RuntimeValue::Variant { tag, payload } if tag == "cons" => {
                let Some(payload) = payload.as_ref() else {
                    return Err(GraphicsBridgeError::Bridge(
                        "cons variant missing payload".into(),
                    ));
                };
                let RuntimeValue::Record(fields) = payload.as_ref() else {
                    return Err(GraphicsBridgeError::Bridge("cons payload record".into()));
                };
                let head = fields
                    .iter()
                    .find(|(k, _)| k == "head")
                    .map(|(_, val)| val)
                    .ok_or_else(|| GraphicsBridgeError::Bridge("cons missing head".into()))?;
                let tail = fields
                    .iter()
                    .find(|(k, _)| k == "tail")
                    .map(|(_, val)| val)
                    .ok_or_else(|| GraphicsBridgeError::Bridge("cons missing tail".into()))?;
                out.push(head);
                cur = tail;
            }
            _ => {
                return Err(GraphicsBridgeError::Bridge(
                    "expected cons/nil list of ruby samples".into(),
                ));
            }
        }
    }
    Ok(out)
}
