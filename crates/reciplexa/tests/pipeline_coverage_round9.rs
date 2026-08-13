//! Round-9 pipeline: document_for_export package path + search-roots residual.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use reciplexa_effect::TestHandler;
use reciplexa::{document_for_export, document_from_source, PipelineError};

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

#[test]
fn pipeline_round9_export_package_and_search_roots() {
    let _lock = env_lock();

    // Package graphics export path (lines 232–233)
    let shapes = include_str!("../../../examples/shapes.rpx");
    let mut h = TestHandler::default();
    let _ = document_for_export(&mut h, shapes);

    // Interim export path (typecheck + effects + lower) — lines 236–238
    let mut h2 = TestHandler::default();
    let _ = document_for_export(&mut h2, "(page a4 (circle 1 2 3))\n");

    // Type error on export
    let mut h3 = TestHandler::default();
    let err = document_for_export(&mut h3, "(page a4 (circle x 2 3))\n");
    assert!(err.is_err());

    // Empty PACKAGE_ROOT → discover miss; also clear force flag
    let missing = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.tmp/rpx_pipeline_n6n_missing");
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        missing.to_str().unwrap_or("missing"),
    );
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");
    let _ = document_from_source("(import widgets/shapes only circle)\n(val main 1)\n");

    // No PACKAGE_ROOT → package_search_roots walk / empty fallback
    drop(_root);
    let _gone = EnvGuard::remove("RECIPLEXA_PACKAGE_ROOT");
    let _ = document_from_source("(import widgets/shapes only circle)\n(val main 1)\n");

    let pe = PipelineError::from(reciplexa_effect::EffectError {
        message: "x".into(),
    });
    let _ = pe.display();
}
