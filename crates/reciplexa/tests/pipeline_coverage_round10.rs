//! Round-10 pipeline: tip production-hash paths via explicit PACKAGE_ROOT
//! (llvm-cov cwd often misses auto-discovered `packages/`).

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use reciplexa::{
    document_for_export, document_from_source, document_from_source_with_snapshot,
    wants_package_graphics_path,
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

fn packages_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages")
}

#[test]
fn pipeline_round10_package_root_and_export_matrix() {
    let _lock = env_lock();
    let pkgs = packages_root();
    assert!(pkgs.is_dir(), "packages dir missing: {}", pkgs.display());

    // Force package root + graphics flag true ("1" / "true")
    let _root = EnvGuard::set("RECIPLEXA_PACKAGE_ROOT", pkgs.to_str().unwrap());
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");

    let shapes = include_str!("../../../examples/shapes.rpx");
    let mut h = TestHandler::default();
    let (doc, _) = document_for_export(&mut h, shapes).expect("export package");
    assert_eq!(doc.pages.len(), 1);
    let _ = wants_package_graphics_path(shapes);

    // Snapshot path: package graphics scene; editable optional (scene-backed, no CST layers)
    let snap = document_from_source_with_snapshot(shapes, false).expect("snap package");
    assert_eq!(snap.scene.pages.len(), 1);
    let _ = document_from_source_with_snapshot(shapes, true);
    let _ = document_from_source_with_snapshot("(page a4 (circle 1 2 3))\n", true);

    // Flag "true" / "false" / "0" branches
    drop(_force);
    let _t = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "true");
    let _ = document_from_source(shapes);
    drop(_t);
    let _f = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "false");
    let _ = document_from_source("(page a4 (circle 1 2 3))\n");
    drop(_f);
    let _z = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "0");
    let _ = document_from_source("(page a4 (circle 1 2 3))\n");
    drop(_z);

    // Interim export (typecheck + effects + lower)
    let mut h2 = TestHandler::default();
    let _ = document_for_export(&mut h2, "(page a4 (circle 1 2 3))\n");

    // Empty / missing PACKAGE_ROOT → walk / packages fallback
    drop(_root);
    let _gone = EnvGuard::remove("RECIPLEXA_PACKAGE_ROOT");
    let _ = document_from_source(shapes);
    let missing =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.tmp/rpx_pipeline_n6o_missing2");
    let _miss = EnvGuard::set(
        "RECIPLEXA_PACKAGE_ROOT",
        missing.to_str().unwrap_or("missing"),
    );
    let _ = document_from_source("(import widgets/shapes only circle)\n(val main 1)\n");
}
