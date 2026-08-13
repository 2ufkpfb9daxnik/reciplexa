//! Round-3 pipeline tips: search-root fallback, expand/export error arms,
//! interim export, and package discover failures without PACKAGE_ROOT.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use reciplexa::{
    document_for_export, document_from_source, document_from_source_with_snapshot, expand,
    is_package_shaped_graphics_source, wants_package_graphics_path, PipelineError,
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
        // SAFETY: serialized via env_lock; restored in Drop.
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
fn package_search_roots_fallback_without_ancestor() {
    let _g = env_lock();
    let _clear_root = EnvGuard::remove("RECIPLEXA_PACKAGE_ROOT");
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");

    let tmp = std::env::temp_dir().join("rpx_pipeline_no_pkgs_ancestor");
    let _ = std::fs::create_dir_all(&tmp);
    let prev = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(&tmp).expect("chdir orphan");

    let err = document_from_source(
        r#"(import graphics/shapes only circle)
(val main (circle 1 2 3))"#,
    );
    let _ = std::env::set_current_dir(prev);
    let err = err.expect_err("fallback packages/ should fail discover");
    assert_eq!(err.stage, "package");
    assert!(!PipelineError::display(&err).is_empty());
}

#[test]
fn with_snapshot_and_export_expand_errors() {
    let err = document_from_source_with_snapshot("(page a4", false).expect_err("macro");
    assert_eq!(err.stage, "macro");

    let mut h = TestHandler::default();
    let err2 = document_for_export(&mut h, "(unclosed").expect_err("macro");
    assert_eq!(err2.stage, "macro");
}

#[test]
fn document_for_export_interim_and_package_force() {
    let _g = env_lock();
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        packages_dir().to_str().expect("utf8"),
    );
    let _force = EnvGuard::remove("RECIPLEXA_PACKAGE_GRAPHICS");

    // Interim (non-package) export path: typecheck → effects → lower.
    let mut h = TestHandler::default();
    let (doc, expanded) =
        document_for_export(&mut h, "(page a4 (circle 1 2 3))").expect("interim export");
    assert_eq!(doc.pages.len(), 1);
    assert!(!wants_package_graphics_path(&expanded));

    // Package export with effects stripped for ingest.
    let mut h2 = TestHandler::default();
    h2.random_seq = vec![0.1];
    let src = include_str!("../../../examples/effects.rpx");
    let (pkg, exp) = document_for_export(&mut h2, src).expect("package export");
    assert_eq!(pkg.pages.len(), 1);
    assert!(wants_package_graphics_path(&exp));
}

#[test]
fn with_snapshot_package_scene_without_editable() {
    let _g = env_lock();
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        packages_dir().to_str().expect("utf8"),
    );
    let _force = EnvGuard::remove("RECIPLEXA_PACKAGE_GRAPHICS");
    let src = include_str!("../../../examples/shapes.rpx");
    let out = document_from_source_with_snapshot(src, false).expect("package scene");
    assert_eq!(out.scene.pages.len(), 1);
    assert!(out.editable.is_none());
}

#[test]
fn package_shaped_comment_and_perform_forms() {
    let expanded = expand(
        r#"(// header)
(import graphics/shapes only circle)
(import graphics/page only a4 page)
(import graphics/color only black)
(src (perform log "x"))
(val main (page a4 (circle 1 2 3 black)))"#,
    )
    .expect("expand");
    assert!(is_package_shaped_graphics_source(&expanded));
    assert!(wants_package_graphics_path(&expanded));
}
