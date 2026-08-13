//! Round-5 pipeline: remaining env force aliases, package discover / snapshot
//! error edges, and strip/page-shaped detection leftovers.

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
fn wants_package_force_false_true_case_variants() {
    let _g = env_lock();
    let pkg = r#"(import graphics/shapes only circle)
(val main (circle 1 2 3))"#;
    let expanded = expand(pkg).expect("expand");
    assert!(is_package_shaped_graphics_source(&expanded));

    for off in ["0", "false", "FALSE", "False"] {
        let _e = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", off);
        assert!(!wants_package_graphics_path(&expanded), "off={off}");
    }
    for on in ["1", "true", "TRUE", "True"] {
        let _e = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", on);
        assert!(wants_package_graphics_path(&expanded), "on={on}");
    }
    // Unrelated value falls through to auto-detect
    let _e = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "maybe");
    assert!(wants_package_graphics_path(&expanded));
}

#[test]
fn package_root_env_and_discover_error() {
    let _g = env_lock();
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        r"C:\Windows\Temp\rpx_pipeline_n6j_missing_pkgs",
    );
    let err = document_from_source(
        r#"(import graphics/shapes only circle)
(val main (circle 1 2 3))"#,
    )
    .expect_err("missing package root");
    assert_eq!(err.stage, "package");
    assert!(!PipelineError::display(&err).is_empty());

    let err = document_from_source_with_snapshot(
        r#"(import graphics/shapes only circle)
(val main (circle 1 2 3))"#,
        false,
    )
    .expect_err("snapshot package");
    assert_eq!(err.stage, "package");
}

#[test]
fn export_package_path_and_macro_error() {
    let _g = env_lock();
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        packages_dir().to_str().expect("utf8"),
    );
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");

    let mut h = TestHandler::default();
    let src = include_str!("../../../examples/shapes.rpx");
    let (doc, expanded) = document_for_export(&mut h, src).expect("export package");
    assert!(!doc.pages.is_empty());
    assert!(wants_package_graphics_path(&expanded));

    let mut h2 = TestHandler::default();
    let _ = run_effects(&mut h2, &expanded);

    let err = document_from_source("(color-byte").expect_err("macro");
    assert_eq!(err.stage, "macro");
}

#[test]
fn strip_effects_and_page_shaped_edges() {
    let _g = env_lock();
    let _clear = EnvGuard::remove("RECIPLEXA_PACKAGE_GRAPHICS");
    let with_page = "(page a4 (circle 1 2 3))\n(val main 1)";
    assert!(!is_package_shaped_graphics_source(with_page));
    let no_main = "(import graphics/shapes only circle)";
    assert!(!is_package_shaped_graphics_source(no_main));
    let _ = expand("(// c)\n(page a4)\n(val main 1)");

    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        packages_dir().to_str().expect("utf8"),
    );
    let with_effects = format!(
        "(perform log \"x\")\n(handle ask (fn (m k) (k m)) 1)\n(src (log \"y\"))\n{}",
        include_str!("../../../examples/shapes.rpx")
    );
    let _ = document_from_source(&with_effects);
}
