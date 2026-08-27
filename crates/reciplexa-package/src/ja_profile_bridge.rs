//! Host consume: `ja-profile-v2-demo` → combined JA Profile v2 GlyphRuns.

use reciplexa_eval::{
    document_from_graphics_value, layout_doc_page_to_scene_with_engine, RuntimeValue,
};
use reciplexa_scene::{Color, Document, Page, PaperSize, Shape};
use reciplexa_std::japanese::{Ruby, RubyKind, TateChuYoko, RUBY_ANNOTATION_SCALE};
use reciplexa_text_layout::ruby::RUBY_PARENT_GAP_EM;
use reciplexa_text_layout::{
    host_product_font, layout_bou_horizontal, layout_bou_vertical, layout_ruby,
    layout_tate_chu_yoko, layout_vertical_ruby, layout_vertical_run,
    positioned_bou_horizontal_to_shapes, positioned_bou_vertical_to_shapes,
    positioned_ruby_to_shapes, positioned_tcy_to_shapes, positioned_vertical_ruby_to_shapes,
    positioned_vertical_to_shapes, TypesetEngine,
};

use crate::graphics_bridge::GraphicsBridgeError;

const SIZE_MM: f64 = 10.0;
const ORIGIN_X_MM: f64 = 20.0;
const VERT_X_MM: f64 = 185.0;
const TCY_X_MM: f64 = 95.0;
const BOU_V_X_MM: f64 = 55.0;
const GAP_MM: f64 = 10.0;
const BELOW_DOC_GAP_MM: f64 = 14.0;
const COL_PITCH_MM: f64 = 16.0;
const FALLBACK_Y_MM: f64 = 220.0;

pub fn is_ja_profile_v2_demo(v: &RuntimeValue) -> bool {
    record_tag(v) == Some("ja-profile-v2-demo")
}

pub fn document_from_ja_profile_v2_value(
    v: &RuntimeValue,
    engine: TypesetEngine,
) -> Result<Document, GraphicsBridgeError> {
    match engine {
        TypesetEngine::Stub => stub_nested_page(v),
        TypesetEngine::Product => product_page(v),
    }
}

fn product_page(v: &RuntimeValue) -> Result<Document, GraphicsBridgeError> {
    let fields = record_fields(v)?;
    let font = host_product_font().map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
    let mut doc = match fields.iter().find(|(k, _)| k == "page").map(|(_, val)| val) {
        Some(page) => layout_doc_page_to_scene_with_engine(page, TypesetEngine::Product)
            .map_err(GraphicsBridgeError::from)?,
        None => Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: Vec::new(),
        }),
    };
    let y0 = below_doc_y(&doc);
    let ruby_band = SIZE_MM * (1.0 + RUBY_ANNOTATION_SCALE + RUBY_PARENT_GAP_EM);
    let mut shapes = Vec::new();
    let mut x = ORIGIN_X_MM;
    let mut y_ruby = y0 - ruby_band;
    for ruby in horiz_rubies(fields)? {
        let measured = layout_ruby(&font, &ruby, 0.0, 0.0, SIZE_MM)
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
        if x > ORIGIN_X_MM && x + measured.advance_mm > TCY_X_MM - GAP_MM {
            x = ORIGIN_X_MM;
            y_ruby -= ruby_band + GAP_MM;
        }
        let laid = layout_ruby(&font, &ruby, x, y_ruby, SIZE_MM)
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
        x += laid.advance_mm + GAP_MM;
        shapes.extend(positioned_ruby_to_shapes(&laid, Color::BLACK));
    }
    let bou_body = string_field(fields, "bou")?;
    let bou_h = layout_bou_horizontal(
        &font,
        &bou_body,
        ORIGIN_X_MM,
        y_ruby - SIZE_MM * 3.0,
        SIZE_MM,
    )
    .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
    shapes.extend(positioned_bou_horizontal_to_shapes(&bou_h, Color::BLACK));
    let y_col = y0;
    let mut vx = VERT_X_MM;
    if let Some(ruby_val) = fields
        .iter()
        .find(|(k, _)| k == "vertical-ruby")
        .map(|(_, v)| v)
    {
        let ruby = ruby_from_value(ruby_val)?;
        let laid = layout_vertical_ruby(&font, &ruby, vx, y_col, SIZE_MM)
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
        shapes.extend(positioned_vertical_ruby_to_shapes(&laid, Color::BLACK));
        vx -= laid.inline_mm + GAP_MM;
    }
    for sample in vert_samples(fields)? {
        let laid = layout_vertical_run(&font, &sample, vx, y_col, SIZE_MM)
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
        shapes.extend(positioned_vertical_to_shapes(&laid, Color::BLACK));
        vx -= COL_PITCH_MM;
    }
    let tcy_body = string_field(fields, "tcy")?;
    let before = string_field(fields, "before").unwrap_or_default();
    let after = string_field(fields, "after").unwrap_or_default();
    let mut ty = y_col;
    if !before.is_empty() {
        let laid = layout_vertical_run(&font, &before, TCY_X_MM, ty, SIZE_MM)
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
        ty -= laid.height_mm;
        shapes.extend(positioned_vertical_to_shapes(&laid, Color::BLACK));
    }
    let tcy = layout_tate_chu_yoko(&font, &TateChuYoko::new(tcy_body), TCY_X_MM, ty, SIZE_MM)
        .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
    ty -= tcy.block_mm;
    shapes.extend(positioned_tcy_to_shapes(&tcy, Color::BLACK));
    if !after.is_empty() {
        let laid = layout_vertical_run(&font, &after, TCY_X_MM, ty, SIZE_MM)
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
        shapes.extend(positioned_vertical_to_shapes(&laid, Color::BLACK));
    }
    let bou_v = layout_bou_vertical(&font, &bou_body, BOU_V_X_MM, y_col, SIZE_MM)
        .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
    shapes.extend(positioned_bou_vertical_to_shapes(&bou_v, Color::BLACK));
    if let Some(page) = doc.pages.first_mut() {
        page.shapes.extend(shapes);
    }
    Ok(doc)
}

fn below_doc_y(doc: &Document) -> f64 {
    let Some(page) = doc.pages.first() else {
        return FALLBACK_Y_MM;
    };
    let min_y = page
        .shapes
        .iter()
        .filter_map(Shape::text_y_mm)
        .fold(None, |acc: Option<f64>, y| {
            Some(acc.map_or(y, |a| a.min(y)))
        });
    match min_y {
        Some(y) => y - BELOW_DOC_GAP_MM,
        None => FALLBACK_Y_MM,
    }
}

fn stub_nested_page(v: &RuntimeValue) -> Result<Document, GraphicsBridgeError> {
    let fields = record_fields(v)?;
    let page = fields
        .iter()
        .find(|(k, _)| k == "page")
        .map(|(_, val)| val)
        .ok_or_else(|| GraphicsBridgeError::Bridge("ja-profile-v2-demo missing page".into()))?;
    document_from_graphics_value(page).map_err(Into::into)
}

fn horiz_rubies(fields: &[(String, RuntimeValue)]) -> Result<Vec<Ruby>, GraphicsBridgeError> {
    let samples = fields
        .iter()
        .find(|(k, _)| k == "horiz-ruby")
        .map(|(_, v)| v)
        .ok_or_else(|| {
            GraphicsBridgeError::Bridge("ja-profile-v2-demo missing horiz-ruby".into())
        })?;
    let items = cons_items(samples)?;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(ruby_from_value(item)?);
    }
    if out.is_empty() {
        return Err(GraphicsBridgeError::Bridge(
            "ja-profile-v2-demo horiz-ruby is empty".into(),
        ));
    }
    Ok(out)
}

fn vert_samples(fields: &[(String, RuntimeValue)]) -> Result<Vec<String>, GraphicsBridgeError> {
    let samples = fields
        .iter()
        .find(|(k, _)| k == "vert-samples")
        .map(|(_, v)| v)
        .ok_or_else(|| {
            GraphicsBridgeError::Bridge("ja-profile-v2-demo missing vert-samples".into())
        })?;
    let items = cons_items(samples)?;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        match item {
            RuntimeValue::String(s) if !s.is_empty() => out.push(s.clone()),
            _ => {
                return Err(GraphicsBridgeError::Bridge(
                    "vert-samples must be non-empty strings".into(),
                ));
            }
        }
    }
    Ok(out)
}

fn ruby_from_value(v: &RuntimeValue) -> Result<Ruby, GraphicsBridgeError> {
    let fields = record_fields(v)?;
    if record_tag(v) != Some("ja-ruby") {
        return Err(GraphicsBridgeError::Bridge(format!(
            "expected ja-ruby, got {:?}",
            record_tag(v)
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

fn record_fields(v: &RuntimeValue) -> Result<&[(String, RuntimeValue)], GraphicsBridgeError> {
    match v {
        RuntimeValue::Record(f) => Ok(f.as_slice()),
        _ => Err(GraphicsBridgeError::Bridge(
            "ja-profile-v2-demo record expected".into(),
        )),
    }
}

fn record_tag(v: &RuntimeValue) -> Option<&str> {
    let fields = record_fields(v).ok()?;
    fields.iter().find_map(|(k, val)| {
        if k != "tag" {
            return None;
        }
        match val {
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
            "ja-profile-v2-demo missing string `{name}`"
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
                return Err(GraphicsBridgeError::Bridge("expected cons/nil list".into()));
            }
        }
    }
    Ok(out)
}
