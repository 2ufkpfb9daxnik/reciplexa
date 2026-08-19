//! Host consume: `ja-ruby` / `ja-ruby-demo` → font-backed simple ruby GlyphRuns.

use reciplexa_eval::{document_from_graphics_value, RuntimeValue};
use reciplexa_scene::{Color, Document, Page, PaperSize};
use reciplexa_std::japanese::{Ruby, RubyKind};
use reciplexa_text_layout::{
    host_product_font, layout_simple_ruby, positioned_ruby_to_shapes, TypesetEngine,
};

use crate::graphics_bridge::GraphicsBridgeError;

const RUBY_ORIGIN_X_MM: f64 = 20.0;
const RUBY_ORIGIN_Y_MM: f64 = 200.0;
const RUBY_BASE_SIZE_MM: f64 = 12.0;

pub fn document_from_ruby_demo_value(
    v: &RuntimeValue,
    engine: TypesetEngine,
) -> Result<Document, GraphicsBridgeError> {
    match engine {
        TypesetEngine::Stub => stub_nested_page(v),
        TypesetEngine::Product => {
            let ruby = ruby_from_demo_or_node(v)?;
            let font =
                host_product_font().map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
            let laid = layout_simple_ruby(
                &font,
                &ruby,
                RUBY_ORIGIN_X_MM,
                RUBY_ORIGIN_Y_MM,
                RUBY_BASE_SIZE_MM,
            )
            .map_err(|e| GraphicsBridgeError::Bridge(e.to_string()))?;
            Ok(Document::single_page(Page {
                paper: PaperSize::a4(),
                shapes: positioned_ruby_to_shapes(&laid, Color::BLACK),
            }))
        }
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
