//! Round-7 pipeline: package discover map_err + cwd fallback (env via unsafe OK here).

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use reciplexa::{document_from_source, document_for_export, wants_package_graphics_path};
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
fn package_discover_map_err_missing_root() {
    let _g = env_lock();
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        "d:/reciplexa/.tmp/rpx_pipeline_n6l_missing_root",
    );
    let err = document_from_source(
        r#"(import graphics/shapes only circle)
(val main (circle 1 2 3))"#,
    );
    assert!(err.is_err());
}

#[test]
fn package_search_roots_relative_fallback_and_export_err() {
    let _g = env_lock();
    let _clear = EnvGuard::remove("RECIPLEXA_PACKAGE_ROOT");
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");

    let tmp = std::env::temp_dir().join("rpx_pipeline_n6l_cwd_nopkgs");
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
    assert!(err.is_err());

    // Export package path with good root still works
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        packages_dir().to_str().expect("utf8"),
    );
    let mut h = TestHandler::default();
    let src = include_str!("../../../examples/shapes.rpx");
    let (doc, expanded) = document_for_export(&mut h, src).expect("export");
    assert!(!doc.pages.is_empty());
    assert!(wants_package_graphics_path(&expanded));
}
