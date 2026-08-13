//! Round-8 pipeline: strip / discover / page-shaped residual tips.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use reciplexa::{document_from_source, PipelineError};

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

    #[allow(dead_code)]
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
fn pipeline_round8_strip_and_page_residuals() {
    let _lock = env_lock();

    // Top-level perform/raise stripped before elaborator
    let _ = document_from_source("(perform log \"x\")\n(val main 1)\n");
    let _ = document_from_source("(raise \"e\")\n(val main 1)\n");

    // Page-shaped
    let _ = document_from_source("(page a4)\n");
    let _ = document_from_source("(// c)\n(page a4 (circle 1 2 3))\n");

    // Empty / parse-fail neighborhood
    let _ = document_from_source("");
    let _ = document_from_source("(");

    // Package discover miss via bogus PACKAGE_ROOT
    let missing =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.tmp/rpx_pipeline_n6m_missing");
    let _root = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        missing.to_str().unwrap_or("missing"),
    );
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");
    let err = document_from_source("(import widgets/shapes only circle)\n(val main 1)\n");
    assert!(err.is_err());
    let _ = PipelineError::display(&err.unwrap_err());
}
