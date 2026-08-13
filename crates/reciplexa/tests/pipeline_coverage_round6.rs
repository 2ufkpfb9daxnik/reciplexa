//! Round-6 pipeline: residual package map_err, snapshot lower, export interim,
//! and strip/parse-fallback edges still under pipeline.rs miss set.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use reciplexa::pipeline::typecheck;
use reciplexa::{
    document_for_export, document_from_source, document_from_source_with_snapshot, expand,
    wants_package_graphics_path, PipelineError,
};
use reciplexa_effect::TestHandler;

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

struct EnvGuard {
    key: &'static str,
    prev: Option<String>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let prev = std::env::var(key).ok();
        unsafe { std::env::set_var(key, value) };
        Self { key, prev }
    }

    fn remove(key: &'static str) -> Self {
        let prev = std::env::var(key).ok();
        unsafe { std::env::remove_var(key) };
        Self { key, prev }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.prev {
            Some(v) => unsafe { std::env::set_var(self.key, v) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

fn packages_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("packages")
        .canonicalize()
        .expect("packages dir")
}

#[test]
fn package_source_map_err_after_discover_ok() {
    let _g = env_lock();
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        packages_dir().to_str().expect("utf8"),
    );
    // Discover succeeds; package eval/bridge fails → document_from_package_source map_err.
    let err = document_from_source(
        r#"(import graphics/shapes only circle)
(val main 42)"#,
    )
    .expect_err("bridge/int main");
    assert_eq!(err.stage, "package");
    assert!(!PipelineError::display(&err).is_empty());
}

#[test]
fn snapshot_interim_refused_and_export_package() {
    let _g = env_lock();
    let _clear = EnvGuard::remove("RECIPLEXA_PACKAGE_GRAPHICS");

    let interim = "(page a4 (circle 1 2 3))";
    let err = document_from_source_with_snapshot(interim, false).expect_err("interim refused");
    assert_eq!(err.stage, "package");

    let bad_lower = document_from_source_with_snapshot("(page a4 (not-a-shape))", false);
    assert!(bad_lower.is_err());

    let mut h = TestHandler::default();
    let err2 = document_for_export(&mut h, interim).expect_err("export interim");
    assert_eq!(err2.stage, "package");

    let mut h2 = TestHandler::default();
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        packages_dir().to_str().expect("utf8"),
    );
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");
    let src = include_str!("../../../examples/shapes.rpx");
    let (doc, expanded) = document_for_export(&mut h2, src).expect("export package");
    assert!(!doc.pages.is_empty());
    assert!(wants_package_graphics_path(&expanded));
}

#[test]
fn package_search_roots_fallback_when_cwd_has_no_packages() {
    let _g = env_lock();
    let _clear_root = EnvGuard::remove("RECIPLEXA_PACKAGE_ROOT");
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");

    let tmp = std::env::temp_dir().join("rpx_pipeline_n6k_cwd_nopkgs");
    let _ = std::fs::create_dir_all(&tmp);
    let prev = std::env::current_dir().ok();
    std::env::set_current_dir(&tmp).expect("chdir");
    let err = document_from_source(
        r#"(import graphics/shapes only circle)
(val main (circle 1 2 3))"#,
    );
    if let Some(p) = prev {
        let _ = std::env::set_current_dir(p);
    }
    // Default relative `packages` root from empty walk → discover/package Err.
    assert!(err.is_err());
}

#[test]
fn expand_typecheck_lower_stage_errors() {
    let err = expand("(color-byte").expect_err("macro");
    assert_eq!(err.stage, "macro");

    let expanded = expand("(page a4 (circle 1 2 3))").unwrap();
    let ty_err = typecheck(&expanded).expect_err("interim typecheck retired");
    assert_eq!(ty_err.stage, "type");

    // Bare keyword page fails at package refuse stage.
    let err = document_from_source("(page a4 (not-a-real-shape 1))").expect_err("package");
    assert_eq!(err.stage, "package", "{err:?}");
}
