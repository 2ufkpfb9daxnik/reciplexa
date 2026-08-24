//! Host consume: `ja-vertical-demo` → font-backed vertical-rl GlyphRuns.

use reciplexa_eval::{
    document_from_graphics_value, layout_doc_page_to_scene_with_engine, RuntimeValue,
};
use reciplexa_scene::{Color, Document, Page, PaperSize, Shape};
use reciplexa_std::japanese::{Ruby, RubyKind};
use reciplexa_text_layout::{
    host_product_font, layout_vertical_ruby, layout_vertical_run,
    positioned_vertical_ruby_to_shapes, positioned_vertical_to_shapes, TypesetEngine,
};

use crate::graphics_bridge::GraphicsBridgeError;

const COL_ORIGIN_X_MM: f64 = 160.0;
const COL_ORIGIN_Y_MM: f64 = 220.0;
const COL_SIZE_MM: f64 = 10.0;
const COL_PITCH_MM: f64 = 16.0;
const COL_BELOW_DOC_GAP_MM: f64 = 14.0;

pub fn document_from_vertical_demo_value(
    v: &RuntimeValue,
    engine: TypesetEngine,
) -> Result<Document, GraphicsBridgeError> {
    match engine {
        TypesetEngine::Stub => stub_nested_page(v),
        TypesetEngine::Product => product_page_with_columns(v),
    }
}

pub fn is_vert_product_demo(v: &RuntimeValue) -> bool {
    let Ok(fields) = record_fields(v) else {
        return false;
    };
    let tag = fields.iter().find_map(|(k, val)| {
        if k != "tag" {
            return None;
        }
        match val {
            RuntimeValue::String(s) | RuntimeValue::ShapeTag(s) => Some(s.as_str()),
            _ => None,
        }
    });
    if tag != Some("ja-vertical-demo") {
        return false;
    }
    fields.iter().any(|(k, val)| {
        k == "samples"
            || matches!(
                (k.as_str(), val),
                ("placed", RuntimeValue::String(s)) if !s.is_empty()
            )
    })
}

fn product_page_with_columns(v: &RuntimeValue) -> Result<Document, GraphicsBridgeError> {
    let fields = record_fields(v)?;
    let samples = samples_from_demo(fields)?;
    let font = host_product_font().map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
    let mut doc = match fields.iter().find(|(k, _)| k == "page").map(|(_, val)| val) {
        Some(page) => layout_doc_page_to_scene_with_engine(page, TypesetEngine::Product)
            .map_err(GraphicsBridgeError::from)?,
        None => Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: Vec::new(),
        }),
    };
    let (mut x, y) = column_origin(&doc);
    let mut shapes = Vec::new();
    if let Some(ruby_val) = fields
        .iter()
        .find(|(k, _)| k == "vertical-ruby")
        .map(|(_, v)| v)
    {
        let ruby = ruby_from_value(ruby_val)?;
        let laid = layout_vertical_ruby(&font, &ruby, x, y, COL_SIZE_MM)
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
        shapes.extend(positioned_vertical_ruby_to_shapes(&laid, Color::BLACK));
        x -= laid.inline_mm + COL_BELOW_DOC_GAP_MM;
    }
    for sample in &samples {
        let laid = layout_vertical_run(&font, sample, x, y, COL_SIZE_MM)
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
        shapes.extend(positioned_vertical_to_shapes(&laid, Color::BLACK));
        x -= COL_PITCH_MM;
    }
    if let Some(page) = doc.pages.first_mut() {
        page.shapes.extend(shapes);
    }
    Ok(doc)
}

fn column_origin(doc: &Document) -> (f64, f64) {
    let Some(page) = doc.pages.first() else {
        return (COL_ORIGIN_X_MM, COL_ORIGIN_Y_MM);
    };
    let min_y = page
        .shapes
        .iter()
        .filter_map(Shape::text_y_mm)
        .fold(None, |acc: Option<f64>, y| {
            Some(acc.map_or(y, |a| a.min(y)))
        });
    match min_y {
        Some(y) => (COL_ORIGIN_X_MM, y - COL_BELOW_DOC_GAP_MM),
        None => (COL_ORIGIN_X_MM, COL_ORIGIN_Y_MM),
    }
}

fn stub_nested_page(v: &RuntimeValue) -> Result<Document, GraphicsBridgeError> {
    let fields = record_fields(v)?;
    let page = fields
        .iter()
        .find(|(k, _)| k == "page")
        .or_else(|| fields.iter().find(|(k, _)| k == "vertical-page"))
        .map(|(_, val)| val)
        .ok_or_else(|| GraphicsBridgeError::Bridge("ja-vertical-demo missing page".into()))?;
    document_from_graphics_value(page).map_err(Into::into)
}

fn samples_from_demo(
    fields: &[(String, RuntimeValue)],
) -> Result<Vec<String>, GraphicsBridgeError> {
    if let Some(samples) = fields.iter().find(|(k, _)| k == "samples").map(|(_, v)| v) {
        let items = cons_items(samples)?;
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            match item {
                RuntimeValue::String(s) if !s.is_empty() => out.push(s.clone()),
                _ => {
                    return Err(GraphicsBridgeError::Bridge(
                        "vertical samples must be non-empty strings".into(),
                    ));
                }
            }
        }
        if !out.is_empty() {
            return Ok(out);
        }
    }
    if let Some(RuntimeValue::String(s)) =
        fields.iter().find(|(k, _)| k == "placed").map(|(_, v)| v)
    {
        if !s.is_empty() {
            return Ok(vec![s.clone()]);
        }
    }
    Err(GraphicsBridgeError::Bridge(
        "ja-vertical-demo missing samples".into(),
    ))
}

fn record_fields(v: &RuntimeValue) -> Result<&[(String, RuntimeValue)], GraphicsBridgeError> {
    match v {
        RuntimeValue::Record(f) => Ok(f.as_slice()),
        _ => Err(GraphicsBridgeError::Bridge(
            "vertical demo record expected".into(),
        )),
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
                    "expected cons/nil list of vertical samples".into(),
                ));
            }
        }
    }
    Ok(out)
}

fn ruby_from_value(v: &RuntimeValue) -> Result<Ruby, GraphicsBridgeError> {
    let fields = record_fields(v)?;
    let tag = fields.iter().find_map(|(k, val)| {
        if k != "tag" {
            return None;
        }
        match val {
            RuntimeValue::String(s) | RuntimeValue::ShapeTag(s) => Some(s.as_str()),
            _ => None,
        }
    });
    if tag != Some("ja-ruby") {
        return Err(GraphicsBridgeError::Bridge(format!(
            "vertical-ruby expects ja-ruby, got {:?}",
            tag
        )));
    }
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
