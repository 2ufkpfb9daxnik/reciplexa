//! Host live-layout consume: compose doc-page + sibling math into one scene.
//!
//! Package mains tagged `live-layout-demo` carry a `page` (`doc-page`) and a
//! `math` tree. Product engine is the host default; `RECIPLEXA_TYPESET_ENGINE=stub`
//! keeps the fontless heuristic. Product MATH/JA lower to [`reciplexa_scene::Shape::GlyphRun`].

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use reciplexa_eval::{
    estimate_style_from_value, layout_doc_page_to_scene, layout_doc_page_to_scene_with_engine,
    layout_math_to_shapes, layout_math_to_shapes_product, RuntimeValue,
};
use reciplexa_scene::{Document, Shape};
use reciplexa_std::math::EstimateStyle;
use reciplexa_text_layout::TypesetEngine;

use crate::graphics_bridge::GraphicsBridgeError;
use crate::load::{eval_package_entry_main, LocalPackageIndex};

static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// Compose a `live-layout-demo` record: `page` via [`layout_doc_page_to_scene`],
/// `math` via [`layout_math_to_shapes`], sibling shapes on page 0.
///
/// Optional record field `style` (`"text"` / `"display"`) selects
/// [`EstimateStyle`] for the math sibling; missing / unknown → Display.
/// Optional `inline-math` is always laid out as [`EstimateStyle::Text`] below
/// the display `math` sibling.
pub fn document_from_live_layout_value(v: &RuntimeValue) -> Result<Document, GraphicsBridgeError> {
    document_from_live_layout_value_with_style(v, estimate_style_from_value(v))
}

/// Like [`document_from_live_layout_value`], with an explicit math [`EstimateStyle`].
///
/// The style argument is the host layout choice; a nested `style` field on the
/// demo record is ignored.
pub fn document_from_live_layout_value_with_style(
    v: &RuntimeValue,
    layout_style: EstimateStyle,
) -> Result<Document, GraphicsBridgeError> {
    document_from_live_layout_value_with_engine_and_style(v, TypesetEngine::Stub, layout_style)
}

pub fn document_from_live_layout_value_with_engine(
    v: &RuntimeValue,
    engine: TypesetEngine,
) -> Result<Document, GraphicsBridgeError> {
    document_from_live_layout_value_with_engine_and_style(v, engine, estimate_style_from_value(v))
}

pub fn document_from_live_layout_value_with_engine_and_style(
    v: &RuntimeValue,
    engine: TypesetEngine,
    layout_style: EstimateStyle,
) -> Result<Document, GraphicsBridgeError> {
    let fields = match v {
        RuntimeValue::Record(f) => f.as_slice(),
        _ => {
            return Err(GraphicsBridgeError::Bridge(
                "live-layout-demo: expected record".into(),
            ));
        }
    };
    let tag = fields
        .iter()
        .find(|(k, _)| k == "tag")
        .and_then(|(_, v)| match v {
            RuntimeValue::String(s) => Some(s.as_str()),
            RuntimeValue::ShapeTag(s) => Some(s.as_str()),
            _ => None,
        });
    if tag != Some("live-layout-demo") {
        return Err(GraphicsBridgeError::Bridge(format!(
            "expected tag `live-layout-demo`, got {:?}",
            tag
        )));
    }
    let page_v = fields
        .iter()
        .find(|(k, _)| k == "page")
        .map(|(_, v)| v)
        .ok_or_else(|| GraphicsBridgeError::Bridge("live-layout-demo missing page".into()))?;
    let math_v = fields
        .iter()
        .find(|(k, _)| k == "math")
        .map(|(_, v)| v)
        .ok_or_else(|| GraphicsBridgeError::Bridge("live-layout-demo missing math".into()))?;

    let mut doc = match engine {
        TypesetEngine::Stub => layout_doc_page_to_scene(page_v),
        TypesetEngine::Product => layout_doc_page_to_scene_with_engine(page_v, engine),
    }
    .map_err(GraphicsBridgeError::from)?;
    let origin = math_origin_below_doc(&doc);
    let mut math_shapes = match engine {
        TypesetEngine::Stub => layout_math_to_shapes(math_v, origin, layout_style),
        TypesetEngine::Product => layout_math_to_shapes_product(math_v, origin, layout_style),
    }
    .map_err(|e| GraphicsBridgeError::Bridge(e.message))?;
    if let Some(inline_v) = fields
        .iter()
        .find(|(k, _)| k == "inline-math")
        .map(|(_, v)| v)
    {
        let below = math_shapes
            .iter()
            .filter_map(Shape::text_y_mm)
            .fold(origin.1, f64::max)
            + 10.0;
        let inline_shapes = match engine {
            TypesetEngine::Stub => {
                layout_math_to_shapes(inline_v, (origin.0, below), EstimateStyle::Text)
            }
            TypesetEngine::Product => {
                layout_math_to_shapes_product(inline_v, (origin.0, below), EstimateStyle::Text)
            }
        }
        .map_err(|e| GraphicsBridgeError::Bridge(e.message))?;
        math_shapes.extend(inline_shapes);
    }
    if let Some(page) = doc.pages.first_mut() {
        page.shapes.extend(math_shapes);
    }
    Ok(doc)
}

fn math_origin_below_doc(doc: &Document) -> (f64, f64) {
    const BASE_X: f64 = 20.0;
    let Some(page) = doc.pages.first() else {
        return (BASE_X, 200.0);
    };
    let min_y = page
        .shapes
        .iter()
        .filter_map(Shape::text_y_mm)
        .fold(None, |acc: Option<f64>, y| {
            Some(acc.map_or(y, |a| a.min(y)))
        });
    match min_y {
        Some(y) => (BASE_X, y - 12.0),
        None => (BASE_X, page.paper.height_mm - 40.0),
    }
}

/// Elaborate/eval package entry `main`, then [`document_from_live_layout_value`].
pub fn document_from_live_layout_entry(
    entry_path: impl AsRef<Path>,
    index: &LocalPackageIndex,
) -> Result<Document, GraphicsBridgeError> {
    let v =
        eval_package_entry_main(entry_path.as_ref(), index).map_err(GraphicsBridgeError::from)?;
    document_from_live_layout_value(&v)
}

/// Write `source` to a temp entry, then [`document_from_live_layout_entry`].
pub fn document_from_live_layout_source(
    source: &str,
    entry_stem: &str,
    index: &LocalPackageIndex,
) -> Result<Document, GraphicsBridgeError> {
    let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-live-{}-{entry_stem}-{seq}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).map_err(|e| GraphicsBridgeError::Load(e.to_string()))?;
    let entry = dir.join(format!("{entry_stem}.rpx"));
    std::fs::write(&entry, source).map_err(|e| GraphicsBridgeError::Load(e.to_string()))?;
    document_from_live_layout_entry(&entry, index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::{Color, GlyphRunShape, Page, PaperSize, Text};

    fn page_with(shapes: Vec<Shape>) -> Document {
        Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes,
        })
    }

    #[test]
    fn math_origin_uses_glyph_run_baselines() {
        let doc = page_with(vec![Shape::GlyphRun(GlyphRunShape::one(
            20.0,
            80.0,
            12.0,
            "括",
            Color::BLACK,
            1,
            "digest",
            10.0,
        ))]);
        let (x, y) = math_origin_below_doc(&doc);
        assert!((x - 20.0).abs() < 1e-9);
        assert!(
            (y - 68.0).abs() < 1e-9,
            "GlyphRun y=80 must beat height-40 fallback, got {y}"
        );
        let fallback = PaperSize::a4().height_mm - 40.0;
        assert!((y - fallback).abs() > 10.0);
    }

    #[test]
    fn math_origin_still_sees_shape_text() {
        let doc = page_with(vec![Shape::Text(Text {
            x_mm: 20.0,
            y_mm: 90.0,
            size_mm: 12.0,
            width_mm: None,
            height_mm: None,
            content: "括".into(),
            fill: Color::BLACK,
        })]);
        let (_, y) = math_origin_below_doc(&doc);
        assert!((y - 78.0).abs() < 1e-9);
    }

    #[test]
    fn math_origin_falls_back_without_text_like_shapes() {
        let doc = page_with(vec![]);
        let (_, y) = math_origin_below_doc(&doc);
        assert!((y - (PaperSize::a4().height_mm - 40.0)).abs() < 1e-9);
    }
}
