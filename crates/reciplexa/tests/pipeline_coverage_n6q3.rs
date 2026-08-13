//! N6 tip q3: pipeline env force + package root residuals.

use reciplexa::{
    document_for_export, document_from_source, document_from_source_with_snapshot,
    wants_package_graphics_path,
};
use reciplexa_effect::TestHandler;
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
    fn remove(key: &'static str) -> Self {
        let prev = std::env::var(key).ok();
        std::env::remove_var(key);
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
fn pipeline_n6q3_env_force_and_roots() {
    let _g = env_lock();
    let pkg = r#"(import graphics/shapes)
(val main (circle 1 2 3))"#;
    let packages = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let _root_ok = EnvGuard::set("RECIPLEXA_PACKAGE_ROOT", packages.to_str().expect("utf8"));

    // Force off via "false" / "0"
    {
        let _off = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "false");
        assert!(!wants_package_graphics_path(pkg));
    }
    {
        let _off0 = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "0");
        assert!(!wants_package_graphics_path(pkg));
    }

    // Force on via "true" / "1" with real package root
    {
        let _on = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "true");
        assert!(wants_package_graphics_path(pkg));
        let _ = document_from_source(pkg);
        let _ = document_from_source_with_snapshot(pkg, false);
        let mut h = TestHandler::default();
        let _ = document_for_export(&mut h, pkg);
    }
    {
        let _on1 = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");
        assert!(wants_package_graphics_path(pkg));
        let _ = document_from_source(pkg);
    }

    // PACKAGE_ROOT pointing at missing → package stage err
    {
        let _bad = EnvGuard::set(
            "RECIPLEXA_PACKAGE_ROOT",
            "d:/reciplexa/.tmp/no_such_pkg_root_n6q3",
        );
        let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");
        assert!(document_from_source(pkg).is_err());
    }

    // No PACKAGE_ROOT: walk cwd parents / fallback "packages"
    {
        let _rm = EnvGuard::remove("RECIPLEXA_PACKAGE_ROOT");
        let _force = EnvGuard::remove("RECIPLEXA_PACKAGE_GRAPHICS");
        let _ = document_from_source("(page a4 (circle 1 2 3))");
    }
}
