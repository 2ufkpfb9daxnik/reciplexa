//! Host live-layout consume: compose doc-page + sibling math into one scene.
//!
//! Package mains tagged `live-layout-demo` carry a `page` (`doc-page`) and a
//! `math` tree; this module lays out both and merges shapes. Heuristic only —
//! see `lang/live-layout-plan.md` (LL3).

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use reciplexa_eval::{
    eval_expr, layout_doc_page_to_scene, layout_math_to_shapes, RuntimeValue, UnitHost,
};
use reciplexa_scene::{Document, Shape};

use crate::graphics_bridge::GraphicsBridgeError;
use crate::load::{elaborate_with_packages, LocalPackageIndex};

static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// Compose a `live-layout-demo` record: `page` via [`layout_doc_page_to_scene`],
/// `math` via [`layout_math_to_shapes`], sibling shapes on page 0.
pub fn document_from_live_layout_value(v: &RuntimeValue) -> Result<Document, GraphicsBridgeError> {
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

    let mut doc = layout_doc_page_to_scene(page_v).map_err(GraphicsBridgeError::from)?;
    let origin = math_origin_below_doc(&doc);
    let math_shapes =
        layout_math_to_shapes(math_v, origin).map_err(|e| GraphicsBridgeError::Bridge(e.message))?;
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
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t.y_mm),
            _ => None,
        })
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
    let units = elaborate_with_packages(entry_path.as_ref(), index)?;
    let stem = entry_path
        .as_ref()
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");
    let demo = units
        .iter()
        .find(|u| u.name == stem)
        .ok_or_else(|| GraphicsBridgeError::Load(format!("missing elaborated unit `{stem}`")))?;
    let v = eval_expr(&demo.expr, &HashMap::new(), &mut UnitHost)
        .map_err(|e| GraphicsBridgeError::Eval(e.message))?;
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
