//! UI theme and persistent GUI preferences (OS config path boundary).

use std::env;
use std::fs;
use std::path::PathBuf;

use eframe::egui;

/// Light / dark chrome. Paper itself stays light (print preview).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UiTheme {
    Light,
    Dark,
}

impl UiTheme {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub(crate) fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    pub(crate) fn save_prefs(self) {
        let mut prefs = GuiPrefs::load();
        prefs.theme = self;
        prefs.save();
    }
}

fn theme_prefs_path() -> Option<PathBuf> {
    let base = env::var_os("LOCALAPPDATA")
        .or_else(|| env::var_os("XDG_CONFIG_HOME"))
        .or_else(|| env::var_os("HOME"))?;
    Some(PathBuf::from(base).join("reciplexa").join("prefs.txt"))
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GuiPrefs {
    pub(crate) theme: UiTheme,
    pub(crate) show_grid: bool,
    pub(crate) zoom: f32,
    pub(crate) pan_x: f32,
    pub(crate) pan_y: f32,
    pub(crate) show_source: bool,
    pub(crate) show_layers: bool,
    pub(crate) show_preview: bool,
    pub(crate) show_props: bool,
    pub(crate) float_source: bool,
    pub(crate) float_layers: bool,
    pub(crate) float_preview: bool,
    pub(crate) float_props: bool,
}

impl GuiPrefs {
    pub(crate) fn load() -> Self {
        let mut prefs = Self::default_prefs();
        let Some(path) = theme_prefs_path() else {
            return prefs;
        };
        let Ok(raw) = fs::read_to_string(path) else {
            return prefs;
        };
        prefs.apply_prefs_text(&raw);
        prefs
    }

    fn default_prefs() -> Self {
        Self {
            theme: UiTheme::Light,
            show_grid: false,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            show_source: true,
            show_layers: true,
            show_preview: true,
            show_props: true,
            float_source: false,
            float_layers: false,
            float_preview: false,
            float_props: false,
        }
    }

    fn apply_prefs_text(&mut self, raw: &str) {
        for line in raw.lines() {
            let line = line.trim();
            if let Some(v) = line.strip_prefix("theme=") {
                if v.eq_ignore_ascii_case("dark") {
                    self.theme = UiTheme::Dark;
                } else {
                    self.theme = UiTheme::Light;
                }
            } else if let Some(v) = line.strip_prefix("grid=") {
                self.show_grid = matches!(v, "1" | "true" | "on");
            } else if let Some(v) = line.strip_prefix("zoom=") {
                if let Ok(z) = v.parse::<f32>() {
                    self.zoom = z.clamp(0.2, 8.0);
                }
            } else if let Some(v) = line.strip_prefix("pan_x=") {
                if let Ok(x) = v.parse::<f32>() {
                    self.pan_x = x;
                }
            } else if let Some(v) = line.strip_prefix("pan_y=") {
                if let Ok(y) = v.parse::<f32>() {
                    self.pan_y = y;
                }
            } else if let Some(v) = line.strip_prefix("pane_source=") {
                self.show_source = matches!(v, "1" | "true" | "on");
            } else if let Some(v) = line.strip_prefix("pane_layers=") {
                self.show_layers = matches!(v, "1" | "true" | "on");
            } else if let Some(v) = line.strip_prefix("pane_preview=") {
                self.show_preview = matches!(v, "1" | "true" | "on");
            } else if let Some(v) = line.strip_prefix("pane_props=") {
                self.show_props = matches!(v, "1" | "true" | "on");
            } else if let Some(v) = line.strip_prefix("float_source=") {
                self.float_source = matches!(v, "1" | "true" | "on");
            } else if let Some(v) = line.strip_prefix("float_layers=") {
                self.float_layers = matches!(v, "1" | "true" | "on");
            } else if let Some(v) = line.strip_prefix("float_preview=") {
                self.float_preview = matches!(v, "1" | "true" | "on");
            } else if let Some(v) = line.strip_prefix("float_props=") {
                self.float_props = matches!(v, "1" | "true" | "on");
            } else if line.eq_ignore_ascii_case("dark") {
                // Backward compatible with old single-word theme file.
                self.theme = UiTheme::Dark;
            }
        }
    }

    pub(crate) fn save(self) {
        let Some(path) = theme_prefs_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let body = format!(
            "theme={}\ngrid={}\nzoom={:.4}\npan_x={:.2}\npan_y={:.2}\npane_source={}\npane_layers={}\npane_preview={}\npane_props={}\nfloat_source={}\nfloat_layers={}\nfloat_preview={}\nfloat_props={}\n",
            match self.theme {
                UiTheme::Light => "light",
                UiTheme::Dark => "dark",
            },
            if self.show_grid { "1" } else { "0" },
            self.zoom.clamp(0.2, 8.0),
            self.pan_x,
            self.pan_y,
            if self.show_source { "1" } else { "0" },
            if self.show_layers { "1" } else { "0" },
            if self.show_preview { "1" } else { "0" },
            if self.show_props { "1" } else { "0" },
            if self.float_source { "1" } else { "0" },
            if self.float_layers { "1" } else { "0" },
            if self.float_preview { "1" } else { "0" },
            if self.float_props { "1" } else { "0" },
        );
        let _ = fs::write(path, body);
    }
}

/// Editor visuals tuned so IME preedit is not a near-black slab.
pub(crate) fn apply_ui_theme(ctx: &egui::Context, theme: UiTheme) {
    let mut visuals = match theme {
        UiTheme::Light => egui::Visuals::light(),
        UiTheme::Dark => egui::Visuals::dark(),
    };
    visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(60, 140, 230, 90);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(90, 160, 230));
    visuals.extreme_bg_color = match theme {
        UiTheme::Light => egui::Color32::from_gray(250),
        // Slightly above panel so TextEdit / source pane stay readable under IME.
        UiTheme::Dark => egui::Color32::from_rgb(36, 36, 40),
    };
    ctx.set_visuals(visuals);
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let dir = std::env::temp_dir().join(format!("reciplexa-prefs-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let prev_local = env::var_os("LOCALAPPDATA");
        let prev_xdg = env::var_os("XDG_CONFIG_HOME");
        let prev_home = env::var_os("HOME");
        unsafe {
            env::set_var("LOCALAPPDATA", &dir);
            env::remove_var("XDG_CONFIG_HOME");
            env::remove_var("HOME");
        }

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

        unsafe {
            match prev_local {
                Some(v) => env::set_var("LOCALAPPDATA", v),
                None => env::remove_var("LOCALAPPDATA"),
            }
            match prev_xdg {
                Some(v) => env::set_var("XDG_CONFIG_HOME", v),
                None => env::remove_var("XDG_CONFIG_HOME"),
            }
            match prev_home {
                Some(v) => env::set_var("HOME", v),
                None => env::remove_var("HOME"),
            }
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn prefs_load_missing_file_returns_defaults() {
        let dir =
            std::env::temp_dir().join(format!("reciplexa-prefs-missing-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let prev = env::var_os("LOCALAPPDATA");
        unsafe {
            env::set_var("LOCALAPPDATA", &dir);
        }
        let prefs = GuiPrefs::load();
        assert_eq!(prefs.theme, UiTheme::Light);
        assert!((prefs.zoom - 1.0).abs() < 1e-6);
        unsafe {
            match prev {
                Some(v) => env::set_var("LOCALAPPDATA", v),
                None => env::remove_var("LOCALAPPDATA"),
            }
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_ui_theme_light_and_dark() {
        let ctx = egui::Context::default();
        apply_ui_theme(&ctx, UiTheme::Light);
        assert!(!ctx.style().visuals.dark_mode);
        apply_ui_theme(&ctx, UiTheme::Dark);
        assert!(ctx.style().visuals.dark_mode);
    }
}
