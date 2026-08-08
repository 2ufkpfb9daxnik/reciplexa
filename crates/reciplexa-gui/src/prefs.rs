//! UI theme and persistent GUI preferences (OS config path boundary).

use std::env;
use std::fs;
use std::path::PathBuf;

use eframe::egui;

/// Light / dark chrome. Paper itself stays light (print preview).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTheme {
    Light,
    Dark,
}

impl UiTheme {
    pub fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    pub fn save_prefs(self) {
        let mut prefs = GuiPrefs::load();
        prefs.theme = self;
        prefs.save();
    }
}

fn theme_prefs_dir() -> Option<PathBuf> {
    let base = env::var_os("LOCALAPPDATA")
        .or_else(|| env::var_os("XDG_CONFIG_HOME"))
        .or_else(|| env::var_os("HOME"))?;
    Some(PathBuf::from(base).join("reciplexa"))
}

fn theme_prefs_path() -> Option<PathBuf> {
    Some(theme_prefs_dir()?.join("prefs.txt"))
}

#[derive(Debug, Clone, Copy)]
pub struct GuiPrefs {
    pub theme: UiTheme,
    pub show_grid: bool,
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub show_source: bool,
    pub show_layers: bool,
    pub show_preview: bool,
    pub show_props: bool,
    pub float_source: bool,
    pub float_layers: bool,
    pub float_preview: bool,
    pub float_props: bool,
}

impl GuiPrefs {
    pub fn load() -> Self {
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

    pub fn default_prefs() -> Self {
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

    pub fn apply_prefs_text(&mut self, raw: &str) {
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

    pub fn save(self) {
        let Some(dir) = theme_prefs_dir() else {
            return;
        };
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("prefs.txt");
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
pub fn apply_ui_theme(ctx: &egui::Context, theme: UiTheme) {
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
