//! Host consume: `ja-tcy-bou-demo` → tate-chu-yoko + bou GlyphRuns/circles.

use reciplexa_eval::{
    document_from_graphics_value, layout_doc_page_to_scene_with_engine, RuntimeValue,
};
use reciplexa_scene::{Color, Document, Page, PaperSize, Shape};
use reciplexa_std::japanese::TateChuYoko;
use reciplexa_text_layout::{
    host_product_font, layout_bou_horizontal, layout_bou_vertical, layout_tate_chu_yoko,
    layout_vertical_run, positioned_bou_horizontal_to_shapes, positioned_bou_vertical_to_shapes,
    positioned_tcy_to_shapes, positioned_vertical_to_shapes, TypesetEngine,
};

use crate::graphics_bridge::GraphicsBridgeError;

const SIZE_MM: f64 = 10.0;
const COL_X_MM: f64 = 150.0;
const BOU_V_X_MM: f64 = 110.0;
const HORIZ_X_MM: f64 = 20.0;
const BELOW_DOC_GAP_MM: f64 = 16.0;
const FALLBACK_Y_MM: f64 = 210.0;

pub fn is_tcy_bou_demo(v: &RuntimeValue) -> bool {
    record_tag(v) == Some("ja-tcy-bou-demo")
}

pub fn document_from_tcy_bou_demo_value(
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
    let y0 = column_y(&doc);
    let before = string_field(fields, "before").unwrap_or_default();
    let after = string_field(fields, "after").unwrap_or_default();
    let tcy_body = string_field(fields, "tcy")?;
    let bou_body = string_field(fields, "bou")?;
    let mut shapes = Vec::new();
    let mut y = y0;
    if !before.is_empty() {
        let laid = layout_vertical_run(&font, &before, COL_X_MM, y, SIZE_MM)
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
        y -= laid.height_mm;
        shapes.extend(positioned_vertical_to_shapes(&laid, Color::BLACK));
    }
    let tcy = layout_tate_chu_yoko(&font, &TateChuYoko::new(tcy_body), COL_X_MM, y, SIZE_MM)
        .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
    y -= tcy.block_mm;
    shapes.extend(positioned_tcy_to_shapes(&tcy, Color::BLACK));
    if !after.is_empty() {
        let laid = layout_vertical_run(&font, &after, COL_X_MM, y, SIZE_MM)
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
        shapes.extend(positioned_vertical_to_shapes(&laid, Color::BLACK));
    }
    let bou_v = layout_bou_vertical(&font, &bou_body, BOU_V_X_MM, y0, SIZE_MM)
        .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
    shapes.extend(positioned_bou_vertical_to_shapes(&bou_v, Color::BLACK));
    let bou_h = layout_bou_horizontal(&font, &bou_body, HORIZ_X_MM, y0 - SIZE_MM * 6.0, SIZE_MM)
        .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
    shapes.extend(positioned_bou_horizontal_to_shapes(&bou_h, Color::BLACK));
    if let Some(page) = doc.pages.first_mut() {
        page.shapes.extend(shapes);
    }
    Ok(doc)
}

fn column_y(doc: &Document) -> f64 {
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
        .ok_or_else(|| GraphicsBridgeError::Bridge("ja-tcy-bou-demo missing page".into()))?;
    document_from_graphics_value(page).map_err(Into::into)
}

fn record_fields(v: &RuntimeValue) -> Result<&[(String, RuntimeValue)], GraphicsBridgeError> {
    match v {
        RuntimeValue::Record(f) => Ok(f.as_slice()),
        _ => Err(GraphicsBridgeError::Bridge(
            "tcy/bou record expected".into(),
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
            "tcy/bou missing string `{name}`"
        ))),
    }
}
