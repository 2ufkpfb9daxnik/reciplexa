//! Thin Slice D adapter: package-entry → scene document via graphics bridge.
//!
//! GUI preview routes package-shaped sources (including golden `black_circle.rpx`)
//! through [`reciplexa::pipeline::document_from_source`]. Interim CST keyword lower
//! is fixture-only (`interim-surface` / `lower_interim_source`). Production pipeline
//! always refuses bare `(page …)`.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use reciplexa_eval::{document_from_graphics_value, GraphicsValueError, RuntimeValue};
use reciplexa_scene::Document;
use reciplexa_text_layout::{host_typeset_engine, TypesetEngine};

use crate::load::{eval_package_entry_main, LocalPackageIndex, PackageLoadError};

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

impl std::fmt::Display for GraphicsBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(s) | Self::Eval(s) | Self::Bridge(s) => write!(f, "{s}"),
        }
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
    document_from_package_entry_with_engine(entry_path, index, TypesetEngine::Stub)
}

/// Host preview/export path: uses [`host_typeset_engine`] (product unless
/// `RECIPLEXA_TYPESET_ENGINE=stub`).
pub fn document_from_package_entry_host(
    entry_path: impl AsRef<Path>,
    index: &LocalPackageIndex,
) -> Result<Document, GraphicsBridgeError> {
    document_from_package_entry_with_engine(entry_path, index, host_typeset_engine())
}

pub fn document_from_package_entry_with_engine(
    entry_path: impl AsRef<Path>,
    index: &LocalPackageIndex,
    engine: TypesetEngine,
) -> Result<Document, GraphicsBridgeError> {
    let entry_path = entry_path.as_ref();
    let v = eval_package_entry_main(entry_path, index).map_err(GraphicsBridgeError::from)?;
    if is_live_layout_demo_tag(&v) {
        return crate::live_layout_bridge::document_from_live_layout_value_with_engine(&v, engine);
    }
    if is_math_demo_tag(&v) {
        return reciplexa_eval::document_from_math_demo_value(&v, engine)
            .map_err(|e| GraphicsBridgeError::Bridge(e.message));
    }
    if engine == TypesetEngine::Product {
        if let Ok(fields) = match &v {
            RuntimeValue::Record(f) => Ok(f.as_slice()),
            _ => Err(()),
        } {
            let tag = fields
                .iter()
                .find(|(k, _)| k == "tag")
                .and_then(|(_, val)| match val {
                    RuntimeValue::String(s) | RuntimeValue::ShapeTag(s) => Some(s.as_str()),
                    _ => None,
                });
            if tag == Some("doc-page") {
                return reciplexa_eval::layout_doc_page_to_scene_with_engine(&v, engine)
                    .map_err(Into::into);
            }
        }
    }
    document_from_graphics_value(&v).map_err(Into::into)
}

fn is_live_layout_demo_tag(v: &RuntimeValue) -> bool {
    record_tag(v) == Some("live-layout-demo")
}

fn is_math_demo_tag(v: &RuntimeValue) -> bool {
    record_tag(v) == Some("math-demo")
}

fn record_tag(v: &RuntimeValue) -> Option<&str> {
    let RuntimeValue::Record(fields) = v else {
        return None;
    };
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

/// Like [`document_from_package_source`] using the host typeset engine.
pub fn document_from_package_source_host(
    source: &str,
    entry_stem: &str,
    index: &LocalPackageIndex,
) -> Result<Document, GraphicsBridgeError> {
    let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-pkg-host-{}-{entry_stem}-{seq}",
        std::process::id()
    ));
    fs_create_dir_all(&dir).map_err(|e| GraphicsBridgeError::Load(e.to_string()))?;
    let entry = dir.join(format!("{entry_stem}.rpx"));
    fs_write(&entry, source).map_err(|e| GraphicsBridgeError::Load(e.to_string()))?;
    document_from_package_entry_host(&entry, index)
}
