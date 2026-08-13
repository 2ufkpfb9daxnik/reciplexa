//! Round-4 pipeline tips: package search-root fallback outside the repo tree,
//! package path with editable snapshot, force-off / force-on env edges, and
//! strip/effect-shaped package sources.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use reciplexa::{
    document_for_export, document_from_source, document_from_source_with_snapshot, expand,
    is_package_shaped_graphics_source, run_effects, wants_package_graphics_path, PipelineError,
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

/// cwd with no `packages/` ancestor within 8 pops — must sit outside the repo.
fn orphan_cwd() -> PathBuf {
    let base = PathBuf::from(r"C:\Windows\Temp").join("rpx_pipeline_n6g_orphan");
    let _ = std::fs::create_dir_all(&base);
    base
}

#[test]
fn package_search_roots_fallback_outside_repo_tree() {
    let _g = env_lock();
    let _clear_root = EnvGuard::remove("RECIPLEXA_PACKAGE_ROOT");
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");

    let tmp = orphan_cwd();
    let prev = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(&tmp).expect("chdir orphan outside repo");

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
fn package_then_editable_and_interim_editable_ok() {
    let _g = env_lock();
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        packages_dir().to_str().expect("utf8"),
    );
    let _force = EnvGuard::remove("RECIPLEXA_PACKAGE_GRAPHICS");
    let src = include_str!("../../../examples/shapes.rpx");
    // Package scene builds, then editable snapshot rejects `import` heads.
    let err = document_from_source_with_snapshot(src, true).expect_err("editable on package");
    assert_eq!(err.stage, "document");

    // Interim path covers with_editable Ok.
    let out = document_from_source_with_snapshot("(page a4 (circle 1 2 3))", true)
        .expect("interim+editable");
    assert_eq!(out.scene.pages.len(), 1);
    assert!(out.editable.is_some());
}

#[test]
fn wants_package_force_off_and_true_aliases() {
    let _g = env_lock();
    let pkg_shaped = r#"(import graphics/shapes only circle)
(val main (circle 1 2 3))"#;
    let expanded = expand(pkg_shaped).expect("expand");
    assert!(is_package_shaped_graphics_source(&expanded));

    let _off = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "false");
    assert!(!wants_package_graphics_path(&expanded));
    drop(_off);

    let _on = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "true");
    assert!(wants_package_graphics_path(&expanded));
}

#[test]
fn interim_export_and_effects_then_package_force() {
    let _g = env_lock();
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        packages_dir().to_str().expect("utf8"),
    );

    // Interim lower path (no graphics import).
    let mut h = TestHandler::default();
    let (doc, exp) = document_for_export(&mut h, "(page a4 (circle 1 2 3))").expect("interim");
    assert_eq!(doc.pages.len(), 1);
    assert!(!wants_package_graphics_path(&exp));

    // Force package path + top-level effect strip before package elaborator.
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");
    // Force package path + strip top-level `(perform …)` before package elaborator.
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");
    let with_effect = format!(
        "(perform log \"skip-me\")\n{}",
        include_str!("../../../examples/shapes.rpx")
    );
    let forced = expand(&with_effect).expect("expand forced");
    assert!(wants_package_graphics_path(&forced));

    let mut h2 = TestHandler::default();
    let _ = run_effects(&mut h2, &forced).expect("effects");

    let mut h3 = TestHandler::default();
    let (pkg, _) = document_for_export(&mut h3, &with_effect).expect("forced package export");
    assert_eq!(pkg.pages.len(), 1);
}

#[test]
fn package_discover_bad_root_and_macro_errors() {
    let _g = env_lock();
    let _root = EnvGuard::set("RECIPLEXA_PACKAGE_ROOT", r"C:\Windows\Temp\rpx_no_such_pkgs_n6g");
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");
    let err = document_from_source(
        r#"(import graphics/shapes only circle)
(val main (circle 1 2 3))"#,
    )
    .expect_err("missing package root");
    assert_eq!(err.stage, "package");

    let err2 = document_from_source_with_snapshot("(page a4", true).expect_err("macro");
    assert_eq!(err2.stage, "macro");
}
