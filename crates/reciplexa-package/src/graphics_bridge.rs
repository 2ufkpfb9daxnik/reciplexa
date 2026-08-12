//! Thin Slice D adapter: package-entry → scene document via graphics bridge.
//!
//! GUI preview routes package-shaped sources through [`reciplexa::pipeline::document_from_source`].
//! Interim CST `(page)/(circle)` keyword lower remains for `black_circle.rpx` CST sync.

use std::collections::HashMap;
use std::path::Path;

use reciplexa_eval::{document_from_graphics_value, eval_expr, GraphicsValueError, UnitHost};
use reciplexa_scene::Document;

use crate::load::{elaborate_with_packages, LocalPackageIndex, PackageLoadError};

/// Errors from package load/eval or graphics bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphicsBridgeError {
    Load(String),
    Eval(String),
    Bridge(String),
}

impl From<PackageLoadError> for GraphicsBridgeError {
    fn from(e: PackageLoadError) -> Self {
        Self::Load(e.to_string())
    }
}

impl From<GraphicsValueError> for GraphicsBridgeError {
    fn from(e: GraphicsValueError) -> Self {
        Self::Bridge(e.message)
    }
}

/// Elaborate `entry_path`, eval its entry `main`, bridge to a scene [`Document`].
///
/// Does **not** replace GUI interim ingest; it is an opt-in strangler adapter.
pub fn document_from_package_entry(
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
    document_from_graphics_value(&v).map_err(Into::into)
}

/// Write `source` to a temp entry file, then [`document_from_package_entry`].
pub fn document_from_package_source(
    source: &str,
    entry_stem: &str,
    index: &LocalPackageIndex,
) -> Result<Document, GraphicsBridgeError> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-pkg-{}-{entry_stem}-{seq}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).map_err(|e| GraphicsBridgeError::Load(e.to_string()))?;
    let entry = dir.join(format!("{entry_stem}.rpx"));
    std::fs::write(&entry, source).map_err(|e| GraphicsBridgeError::Load(e.to_string()))?;
    document_from_package_entry(&entry, index)
}
