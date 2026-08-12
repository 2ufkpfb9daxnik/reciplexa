use std::env;
use std::fs;
use std::sync::{Mutex, MutexGuard};

use eframe::egui;
use reciplexa_gui::prefs::{apply_ui_theme, GuiPrefs, UiTheme};

/// Prefs path reads process env; serialize tests that mutate those vars.
fn prefs_env_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

struct EnvGuard {
    local: Option<std::ffi::OsString>,
    xdg: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn capture() -> Self {
        Self {
            local: env::var_os("LOCALAPPDATA"),
            xdg: env::var_os("XDG_CONFIG_HOME"),
            home: env::var_os("HOME"),
        }
    }

    fn clear_all_config_roots() {
        unsafe {
            env::remove_var("LOCALAPPDATA");
            env::remove_var("XDG_CONFIG_HOME");
            env::remove_var("HOME");
        }
    }

    fn set_local(dir: &std::path::Path) {
        unsafe {
            env::set_var("LOCALAPPDATA", dir);
            env::remove_var("XDG_CONFIG_HOME");
            env::remove_var("HOME");
        }
    }

    fn set_xdg(dir: &std::path::Path) {
        unsafe {
            env::remove_var("LOCALAPPDATA");
            env::set_var("XDG_CONFIG_HOME", dir);
            env::remove_var("HOME");
        }
    }

    fn set_home(dir: &std::path::Path) {
        unsafe {
            env::remove_var("LOCALAPPDATA");
            env::remove_var("XDG_CONFIG_HOME");
            env::set_var("HOME", dir);
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        unsafe {
            match self.local.take() {
                Some(v) => env::set_var("LOCALAPPDATA", v),
                None => env::remove_var("LOCALAPPDATA"),
            }
            match self.xdg.take() {
                Some(v) => env::set_var("XDG_CONFIG_HOME", v),
                None => env::remove_var("XDG_CONFIG_HOME"),
            }
            match self.home.take() {
                Some(v) => env::set_var("HOME", v),
                None => env::remove_var("HOME"),
            }
        }
    }
}

#[test]
fn prefs_text_parses_theme_zoom_and_panes() {
    let mut prefs = GuiPrefs::default_prefs();
    prefs.apply_prefs_text(
        "theme=dark\ngrid=1\nzoom=2.5\npan_x=10\npan_y=-3\npane_source=0\npane_layers=1\npane_preview=0\npane_props=0\nfloat_source=1\nfloat_preview=1\n",
    );
    assert_eq!(prefs.theme, UiTheme::Dark);
    assert!(prefs.show_grid);
    assert!((prefs.zoom - 2.5).abs() < 1e-4);
    assert!((prefs.pan_x - 10.0).abs() < 1e-4);
    assert!(!prefs.show_source);
    assert!(prefs.show_layers);
    assert!(!prefs.show_preview);
    assert!(!prefs.show_props);
    assert!(prefs.float_source);
    assert!(!prefs.float_layers);
    assert!(prefs.float_preview);
    assert!(!prefs.float_props);
}

#[test]
fn prefs_missing_float_defaults_false() {
    let mut prefs = GuiPrefs::default_prefs();
    prefs.apply_prefs_text("theme=light\npane_source=1\n");
    assert!(!prefs.float_source);
    assert!(!prefs.float_layers);
    assert!(!prefs.float_preview);
    assert!(!prefs.float_props);
    assert!(prefs.show_props);
}

#[test]
fn prefs_zoom_clamps() {
    let mut prefs = GuiPrefs::default_prefs();
    prefs.apply_prefs_text("zoom=99\n");
    assert!((prefs.zoom - 8.0).abs() < 1e-4);
}

#[test]
fn ui_theme_label_toggle_and_partitions() {
    assert_eq!(UiTheme::Light.label(), "Light");
    assert_eq!(UiTheme::Dark.label(), "Dark");
    assert_eq!(UiTheme::Light.toggle(), UiTheme::Dark);
    assert_eq!(UiTheme::Dark.toggle(), UiTheme::Light);
}

#[test]
fn prefs_boolean_equivalence_and_legacy_dark() {
    let mut prefs = GuiPrefs::default_prefs();
    prefs.apply_prefs_text(
        "grid=true\npane_source=on\npane_layers=false\nfloat_layers=1\nfloat_props=true\n",
    );
    assert!(prefs.show_grid);
    assert!(prefs.show_source);
    assert!(!prefs.show_layers);
    assert!(prefs.float_layers);
    assert!(prefs.float_props);

    let mut prefs = GuiPrefs::default_prefs();
    prefs.apply_prefs_text("DARK\n");
    assert_eq!(prefs.theme, UiTheme::Dark);

    let mut prefs = GuiPrefs::default_prefs();
    prefs.apply_prefs_text("dark");
    assert_eq!(prefs.theme, UiTheme::Dark);

    let mut prefs = GuiPrefs::default_prefs();
    prefs.apply_prefs_text("theme=LIGHT\nzoom=0.01\nzoom=not-a-number\npan_x=abc\npan_y=\n");
    assert_eq!(prefs.theme, UiTheme::Light);
    assert!((prefs.zoom - 0.2).abs() < 1e-4);
    assert_eq!(prefs.pan_x, 0.0);
}

#[test]
fn prefs_roundtrip_via_apply_of_save_format() {
    let mut prefs = GuiPrefs::default_prefs();
    prefs.theme = UiTheme::Dark;
    prefs.show_grid = true;
    prefs.zoom = 3.5;
    prefs.pan_x = -1.25;
    prefs.pan_y = 4.5;
    prefs.show_source = false;
    prefs.float_props = true;
    let body = format!(
        "theme={}\ngrid={}\nzoom={:.4}\npan_x={:.2}\npan_y={:.2}\npane_source={}\npane_layers={}\npane_preview={}\npane_props={}\nfloat_source={}\nfloat_layers={}\nfloat_preview={}\nfloat_props={}\n",
        "dark",
        "1",
        prefs.zoom.clamp(0.2, 8.0),
        prefs.pan_x,
        prefs.pan_y,
        "0",
        "1",
        "1",
        "1",
        "0",
        "0",
        "0",
        "1",
    );
    let mut loaded = GuiPrefs::default_prefs();
    loaded.apply_prefs_text(&body);
    assert_eq!(loaded.theme, UiTheme::Dark);
    assert!(loaded.show_grid);
    assert!((loaded.zoom - 3.5).abs() < 1e-3);
    assert!(!loaded.show_source);
    assert!(loaded.float_props);
}

#[test]
fn prefs_save_load_roundtrip_via_temp_config_home() {
    let _lock = prefs_env_lock();
    let _guard = EnvGuard::capture();
    let dir = std::env::temp_dir().join(format!("reciplexa-prefs-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    EnvGuard::set_local(&dir);

    let mut prefs = GuiPrefs::default_prefs();
    prefs.theme = UiTheme::Dark;
    prefs.show_grid = true;
    prefs.zoom = 2.0;
    prefs.pan_x = 1.5;
    prefs.pan_y = -2.5;
    prefs.show_source = false;
    prefs.float_layers = true;
    prefs.save();

    let loaded = GuiPrefs::load();
    assert_eq!(loaded.theme, UiTheme::Dark);
    assert!(loaded.show_grid);
    assert!((loaded.zoom - 2.0).abs() < 1e-3);
    assert!((loaded.pan_x - 1.5).abs() < 1e-3);
    assert!(!loaded.show_source);
    assert!(loaded.float_layers);

    UiTheme::Light.save_prefs();
    assert_eq!(GuiPrefs::load().theme, UiTheme::Light);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn prefs_load_missing_file_returns_defaults() {
    let _lock = prefs_env_lock();
    let _guard = EnvGuard::capture();
    let dir = std::env::temp_dir().join(format!("reciplexa-prefs-missing-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    EnvGuard::set_local(&dir);
    let prefs = GuiPrefs::load();
    assert_eq!(prefs.theme, UiTheme::Light);
    assert!((prefs.zoom - 1.0).abs() < 1e-6);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn prefs_load_and_save_noop_without_config_home() {
    let _lock = prefs_env_lock();
    let _guard = EnvGuard::capture();
    EnvGuard::clear_all_config_roots();
    let prefs = GuiPrefs::load();
    assert_eq!(prefs.theme, UiTheme::Light);
    assert!((prefs.zoom - 1.0).abs() < 1e-6);
    GuiPrefs::default_prefs().save();
    UiTheme::Dark.save_prefs();
}

#[test]
fn prefs_path_falls_back_to_xdg_and_home() {
    let _lock = prefs_env_lock();
    let _guard = EnvGuard::capture();
    let xdg = std::env::temp_dir().join(format!("reciplexa-prefs-xdg-{}", std::process::id()));
    let home = std::env::temp_dir().join(format!("reciplexa-prefs-home-{}", std::process::id()));
    let _ = fs::remove_dir_all(&xdg);
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&home).unwrap();

    EnvGuard::set_xdg(&xdg);
    let mut prefs = GuiPrefs::default_prefs();
    prefs.theme = UiTheme::Dark;
    prefs.show_grid = false;
    prefs.show_source = true;
    prefs.show_layers = false;
    prefs.show_preview = false;
    prefs.show_props = false;
    prefs.float_source = true;
    prefs.float_layers = true;
    prefs.float_preview = true;
    prefs.float_props = true;
    prefs.zoom = 0.1;
    prefs.save();
    let loaded = GuiPrefs::load();
    assert_eq!(loaded.theme, UiTheme::Dark);
    assert!(!loaded.show_grid);
    assert!(loaded.show_source);
    assert!(!loaded.show_layers);
    assert!(!loaded.show_preview);
    assert!(!loaded.show_props);
    assert!(loaded.float_source);
    assert!(loaded.float_layers);
    assert!(loaded.float_preview);
    assert!(loaded.float_props);
    assert!((loaded.zoom - 0.2).abs() < 1e-3);

    EnvGuard::set_home(&home);
    let mut prefs = GuiPrefs::default_prefs();
    prefs.theme = UiTheme::Light;
    prefs.save();
    assert_eq!(GuiPrefs::load().theme, UiTheme::Light);

    let _ = fs::remove_dir_all(&xdg);
    let _ = fs::remove_dir_all(&home);
}

#[test]
fn apply_ui_theme_light_and_dark() {
    let ctx = egui::Context::default();
    apply_ui_theme(&ctx, UiTheme::Light);
    assert!(!ctx.style().visuals.dark_mode);
    apply_ui_theme(&ctx, UiTheme::Dark);
    assert!(ctx.style().visuals.dark_mode);
}
