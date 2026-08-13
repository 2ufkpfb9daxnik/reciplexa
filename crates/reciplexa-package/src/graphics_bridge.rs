//! Thin Slice D adapter: package-entry → scene document via graphics bridge.
//!
//! GUI preview routes package-shaped sources (including golden `black_circle.rpx`)
//! through [`reciplexa::pipeline::document_from_source`]. Interim CST keyword lower
//! is fixture-only (`interim-surface` / `lower_interim_source`). Production pipeline
//! always refuses bare `(page …)`.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use reciplexa_eval::{
    document_from_graphics_value, eval_expr, primitive_env, GraphicsValueError, RuntimeValue,
    UnitHost,
};
use reciplexa_scene::Document;

use crate::load::{elaborate_with_packages, LocalPackageIndex, PackageLoadError};
use crate::resource_value::maybe_materialize_package_resources_for_entry;

/// Seq for unique temp dirs; module-level so llvm-cov marks static init covered.
static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

// Test-only FS fail seam: when set, `create_dir_all` / `write` return `io::Error`
// so the `map_err` Load arms in `document_from_package_source` are reachable.
// Unset (default) preserves production behavior.
thread_local! {
    static TEST_FAIL_CREATE_DIR: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static TEST_FAIL_WRITE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Inject `create_dir_all` failure (coverage / tests only). Default off.
#[doc(hidden)]
pub fn test_set_fail_create_dir(fail: bool) {
    TEST_FAIL_CREATE_DIR.with(|c| c.set(fail));
}

/// Inject `write` failure (coverage / tests only). Default off.
#[doc(hidden)]
pub fn test_set_fail_write(fail: bool) {
    TEST_FAIL_WRITE.with(|c| c.set(fail));
}

fn fs_create_dir_all(path: &Path) -> std::io::Result<()> {
    if TEST_FAIL_CREATE_DIR.with(|c| c.get()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "TEST_FAIL_CREATE_DIR",
        ));
    }
    std::fs::create_dir_all(path)
}

fn fs_write(path: &Path, contents: &str) -> std::io::Result<()> {
    if TEST_FAIL_WRITE.with(|c| c.get()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "TEST_FAIL_WRITE",
        ));
    }
    std::fs::write(path, contents)
}

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
/// Product pipeline auto-routes package-shaped sources; bare `(page …)` is
/// refused at the pipeline gate (S6b).
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
    let v = eval_expr(&demo.expr, &primitive_env(), &mut UnitHost)
        .map_err(|e| GraphicsBridgeError::Eval(e.message))?;
    let v = maybe_materialize_package_resources_for_entry(&v, entry_path.as_ref());
    // Live-layout demos (doc page + sibling math) are not graphics/page trees.
    if is_live_layout_demo_tag(&v) {
        return crate::live_layout_bridge::document_from_live_layout_value(&v);
    }
    document_from_graphics_value(&v).map_err(Into::into)
}

fn is_live_layout_demo_tag(v: &RuntimeValue) -> bool {
    let RuntimeValue::Record(fields) = v else {
        return false;
    };
    fields.iter().any(|(k, val)| {
        k == "tag"
            && matches!(
                val,
                RuntimeValue::String(s) | RuntimeValue::ShapeTag(s) if s == "live-layout-demo"
            )
    })
}

/// Write `source` to a temp entry file, then [`document_from_package_entry`].
pub fn document_from_package_source(
    source: &str,
    entry_stem: &str,
    index: &LocalPackageIndex,
) -> Result<Document, GraphicsBridgeError> {
    let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-pkg-{}-{entry_stem}-{seq}",
        std::process::id()
    ));
    fs_create_dir_all(&dir).map_err(|e| GraphicsBridgeError::Load(e.to_string()))?;
    let entry = dir.join(format!("{entry_stem}.rpx"));
    fs_write(&entry, source).map_err(|e| GraphicsBridgeError::Load(e.to_string()))?;
    document_from_package_entry(&entry, index)
}
