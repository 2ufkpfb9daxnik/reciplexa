//! Round-2 pipeline tips: env overrides, package-root search, package export/snapshot,
//! interim-page detection beside `(val main)`, and package-stage errors.

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

fn shapes_src() -> &'static str {
    include_str!("../../../examples/shapes.rpx")
}

fn effects_src() -> &'static str {
    include_str!("../../../examples/effects.rpx")
}

#[test]
fn wants_package_graphics_env_force_off_and_on() {
    let _g = env_lock();
    let expanded = expand(shapes_src()).expect("expand shapes");
    assert!(wants_package_graphics_path(&expanded));

    for off in ["0", "false", "FALSE"] {
        let _e = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", off);
        assert!(
            !wants_package_graphics_path(&expanded),
            "force off via {off}"
        );
    }

    // Force on even when auto-detect would refuse (interim page + import).
    let mixed = expand(
        r#"(import graphics/shapes)
(val main 1)
(page a4 (circle 1 2 3))"#,
    )
    .expect("expand mixed");
    assert!(!is_package_shaped_graphics_source(&mixed));
    for on in ["1", "true", "TRUE"] {
        let _e = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", on);
        assert!(wants_package_graphics_path(&mixed), "force on via {on}");
    }

    let _clear = EnvGuard::remove("RECIPLEXA_PACKAGE_GRAPHICS");
    assert!(wants_package_graphics_path(&expanded));
}

#[test]
fn package_search_root_override_and_ingest() {
    let _g = env_lock();
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        packages_dir().to_str().expect("utf8 path"),
    );
    let _force = EnvGuard::remove("RECIPLEXA_PACKAGE_GRAPHICS");
    let doc = document_from_source(shapes_src()).expect("ingest via PACKAGE_ROOT");
    assert_eq!(doc.pages.len(), 1);
}

#[test]
fn package_shaped_rejects_top_level_page_and_accepts_comment() {
    let _g = env_lock();
    let _force = EnvGuard::remove("RECIPLEXA_PACKAGE_GRAPHICS");
    let with_page = expand(
        r#"(import graphics/shapes)
(val main (circle 1 2 3))
(page a4 (circle 1 2 3))"#,
    )
    .expect("expand");
    assert!(!is_package_shaped_graphics_source(&with_page));
    assert!(!wants_package_graphics_path(&with_page));

    let with_comment = expand(
        r#"(// skip)
(import graphics/shapes only circle)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (circle 1 2 3 black)))"#,
    )
    .expect("expand comment");
    assert!(is_package_shaped_graphics_source(&with_comment));

    // Parse failure inside page probe → treat as no interim page.
    assert!(is_package_shaped_graphics_source(
        "(import graphics/shapes)\n(val main 1)\n(page"
    ));
}

#[test]
fn document_for_export_and_snapshot_use_package_bridge() {
    let _g = env_lock();
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        packages_dir().to_str().expect("utf8 path"),
    );
    let _force = EnvGuard::remove("RECIPLEXA_PACKAGE_GRAPHICS");

    let mut h = TestHandler::default();
    h.random_seq = vec![0.25];
    let (doc, expanded) = document_for_export(&mut h, effects_src()).expect("export package");
    assert_eq!(doc.pages.len(), 1);
    assert!(wants_package_graphics_path(&expanded));
    assert!(!h.logs.is_empty());
    assert!(!h.writes.is_empty());

    // Package bridge scene ok; editable snapshot is scene-backed (no interim CST layers).
    let no_snap = document_from_source_with_snapshot(shapes_src(), false).expect("no snap");
    assert_eq!(no_snap.scene.pages.len(), 1);
    assert!(no_snap.editable.is_none());

    let with_snap =
        document_from_source_with_snapshot(shapes_src(), true).expect("package editable");
    assert_eq!(with_snap.scene.pages.len(), 1);
    assert!(with_snap.editable.is_some());
}

#[test]
fn package_stage_errors_surface() {
    let _g = env_lock();
    let missing = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("__no_such_packages_root__");
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        missing.to_str().expect("utf8 path"),
    );
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");

    let err = document_from_source(
        r#"(import graphics/shapes)
(val main 1)"#,
    )
    .expect_err("missing package root");
    assert_eq!(err.stage, "package");
    assert!(!PipelineError::display(&err).is_empty());

    // Discover ok but eval/bridge fails (main is not a page document).
    let _root_ok = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        packages_dir().to_str().expect("utf8 path"),
    );
    let err2 = document_from_source(
        r#"(import graphics/shapes only circle)
(val main (circle 1 2 3))"#,
    )
    .expect_err("bridge should reject bare circle as document");
    assert_eq!(err2.stage, "package");
}
