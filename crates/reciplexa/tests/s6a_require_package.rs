//! S6a: RECIPLEXA_REQUIRE_PACKAGE rejects interim top-level `(page …)`.

use reciplexa::{document_from_source, refuse_interim_if_required};
use std::sync::{Mutex, OnceLock};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

struct EnvGuard {
    key: &'static str,
    prev: Option<String>,
}
impl EnvGuard {
    fn set(key: &'static str, val: &str) -> Self {
        let prev = std::env::var(key).ok();
        std::env::set_var(key, val);
        Self { key, prev }
    }
}
impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.prev {
            Some(v) => std::env::set_var(self.key, v),
            None => std::env::remove_var(self.key),
        }
    }
}

#[test]
fn require_package_rejects_interim_page_when_enabled() {
    let _lock = env_lock();
    let interim = "(page a4 (circle 1 2 3))";
    assert!(refuse_interim_if_required(interim).is_ok());
    let _req = EnvGuard::set("RECIPLEXA_REQUIRE_PACKAGE", "1");
    let err = refuse_interim_if_required(interim).expect_err("require package");
    assert_eq!(err.stage, "package");
    assert!(err.message.contains("deprecated"));
    let doc_err = document_from_source(interim).expect_err("ingest blocked");
    assert_eq!(doc_err.stage, "package");
    let pkg = include_str!("../../../examples/black_circle.rpx");
    assert!(document_from_source(pkg).is_ok());
}
