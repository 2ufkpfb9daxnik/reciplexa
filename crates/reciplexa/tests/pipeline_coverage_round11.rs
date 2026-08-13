//! Round-11 pipeline: package discover Io fail, strip effects, export edges.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use reciplexa_effect::TestHandler;
use reciplexa::{
    document_for_export, document_from_source, document_from_source_with_snapshot, expand,
    is_package_shaped_graphics_source, lower, run_effects, wants_package_graphics_path,
};

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
fn pipeline_round11_discover_fail_strip_export() {
    let _lock = env_lock();
    let pkgs = packages_root();
    assert!(pkgs.is_dir());

    // Package graphics forced on + valid root
    let _root = EnvGuard::set("RECIPLEXA_PACKAGE_ROOT", pkgs.to_str().unwrap());
    let _force = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "1");

    let shapes = include_str!("../../../examples/shapes.rpx");
    let mut h = TestHandler::default();
    let _ = document_for_export(&mut h, shapes);
    let _ = wants_package_graphics_path(shapes);
    let _ = is_package_shaped_graphics_source(shapes);

    // Strip top-level perform/handle/src before package path
    let with_fx = format!(
        "{shapes}\n(perform log \"x\")\n(handle ask (fn (m) m) 1)\n(src \"y\")\n"
    );
    let _ = document_from_source(&with_fx);
    let _ = document_from_source_with_snapshot(&with_fx, false);

    // Structured comment + page detect
    let _ = wants_package_graphics_path("(// note)\n(import graphics/shapes)\n(val main 1)\n");
    let _ = is_package_shaped_graphics_source("(import graphics/shapes)\n(page a4 (circle 1 2 3))\n");
    let _ = is_package_shaped_graphics_source("(import graphics/shapes)\n(val main 1)\n");

    // Discover fail: PACKAGE_ROOT points at a *file* (read_dir Io)
    drop(_root);
    let bad = std::env::temp_dir().join(format!(
        "rpx_pipe_n6p_bad_{}",
        std::process::id()
    ));
    fs::write(&bad, b"not-a-dir").unwrap();
    let _bad_root = EnvGuard::set("RECIPLEXA_PACKAGE_ROOT", bad.to_str().unwrap_or("x"));
    let _ = document_from_source(shapes);
    let mut h2 = TestHandler::default();
    let _ = document_for_export(&mut h2, shapes);
    let _ = document_from_source_with_snapshot(shapes, false);
    drop(_bad_root);
    let _ = fs::remove_file(&bad);

    // Missing root + force off / interim export
    let _gone = EnvGuard::remove("RECIPLEXA_PACKAGE_ROOT");
    drop(_force);
    let _off = EnvGuard::set("RECIPLEXA_PACKAGE_GRAPHICS", "0");
    let _ = document_from_source("(page a4 (circle 1 2 3))\n");
    let mut h3 = TestHandler::default();
    let _ = document_for_export(&mut h3, "(page a4 (circle 1 2 3))\n(perform log \"z\")\n");
    let _ = run_effects(&mut h3, "(perform log \"z\")\n");
    let _ = document_from_source_with_snapshot("(page a4 (rect 1 2 3 4))\n", true);

    // Expand/typecheck/lower error surfaces
    let _ = document_from_source("(");
    let _ = document_from_source("(val main (unknown-form))\n");

    // package_search_roots walk: no PACKAGE_ROOT, cwd may lack packages
    drop(_gone);
    let _walk = EnvGuard::remove("RECIPLEXA_PACKAGE_ROOT");
    let _auto = EnvGuard::remove("RECIPLEXA_PACKAGE_GRAPHICS");
    let _ = wants_package_graphics_path(
        "(import graphics/shapes)\n(val main 1)\n(// c)\n",
    );
    let _ = document_from_source(shapes);
    // Interim page + import graphics → not package-shaped
    let _ = document_from_source(
        "(import graphics/shapes)\n(page a4 (circle 1 2 3))\n",
    );
    // Snapshot editable true on interim
    let _ = document_from_source_with_snapshot("(page a4 (text 1 2 3 \"x\"))\n", true);
    let mut h4 = TestHandler::default();
    let _ = document_for_export(&mut h4, "(markup @title{T})\n");
    let _ = expand("(markup @title{Hi})");
    let _ = lower("(page a4 (circle 1 2 3))\n");

    // StructuredComment skip in has_top_level_interim_page + Opacity leaf walk
    let _auto2 = EnvGuard::remove("RECIPLEXA_PACKAGE_GRAPHICS");
    let _ = is_package_shaped_graphics_source(
        "(// note)\n(import graphics/shapes)\n(val main 1)\n",
    );
    let _ = wants_package_graphics_path(
        "(// note)\n(import graphics/shapes)\n(page a4 (circle 1 2 3))\n",
    );
    let _ = document_from_source("(page a4 (opacity 0.5 (rect 1 2 3 4)))\n");
    let _ = document_from_source("(page a4 (group (opacity 0.5 (circle 1 2 3))))\n");
    // typecheck+lower path (no package graphics)
    let _ = document_from_source("(page a4 (line 1 2 3 4 black 1))\n");
    let _ = document_from_source_with_snapshot("(page a4 (polyline 1 2 3 4))\n", false);
}
