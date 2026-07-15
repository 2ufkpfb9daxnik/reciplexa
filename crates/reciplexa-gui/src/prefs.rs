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
}

impl GuiPrefs {
    pub(crate) fn load() -> Self {
        let mut prefs = Self {
            theme: UiTheme::Light,
            show_grid: false,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            show_source: true,
            show_layers: true,
            show_preview: true,
        };
        let Some(path) = theme_prefs_path() else {
            return prefs;
        };
        let Ok(raw) = fs::read_to_string(path) else {
            return prefs;
        };
        for line in raw.lines() {
            let line = line.trim();
            if let Some(v) = line.strip_prefix("theme=") {
                if v.eq_ignore_ascii_case("dark") {
                    prefs.theme = UiTheme::Dark;
                } else {
                    prefs.theme = UiTheme::Light;
                }
            } else if let Some(v) = line.strip_prefix("grid=") {
                prefs.show_grid = matches!(v, "1" | "true" | "on");
            } else if let Some(v) = line.strip_prefix("zoom=") {
                if let Ok(z) = v.parse::<f32>() {
                    prefs.zoom = z.clamp(0.2, 8.0);
                }
            } else if let Some(v) = line.strip_prefix("pan_x=") {
                if let Ok(x) = v.parse::<f32>() {
                    prefs.pan_x = x;
                }
            } else if let Some(v) = line.strip_prefix("pan_y=") {
                if let Ok(y) = v.parse::<f32>() {
                    prefs.pan_y = y;
                }
            } else if let Some(v) = line.strip_prefix("pane_source=") {
                prefs.show_source = matches!(v, "1" | "true" | "on");
            } else if let Some(v) = line.strip_prefix("pane_layers=") {
                prefs.show_layers = matches!(v, "1" | "true" | "on");
            } else if let Some(v) = line.strip_prefix("pane_preview=") {
                prefs.show_preview = matches!(v, "1" | "true" | "on");
            } else if line.eq_ignore_ascii_case("dark") {
                // Backward compatible with old single-word theme file.
                prefs.theme = UiTheme::Dark;
            }
        }
        prefs
    }

    pub(crate) fn save(self) {
        let Some(path) = theme_prefs_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let body = format!(
            "theme={}\ngrid={}\nzoom={:.4}\npan_x={:.2}\npan_y={:.2}\npane_source={}\npane_layers={}\npane_preview={}\n",
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
