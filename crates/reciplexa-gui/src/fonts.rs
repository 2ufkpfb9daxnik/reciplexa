//! CJK font discovery and install for the egui preview (OS font-file boundary).

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;

pub fn install_cjk_fonts(ctx: &egui::Context) {
    let windir = PathBuf::from(env::var_os("WINDIR").unwrap_or_else(|| r"C:\Windows".into()));
    let fonts_dir = windir.join("Fonts");
    // Prefer static TTF/TTC. Variable fonts often break egui glyph metrics, which
    // makes IME preedit look like a solid black block over the whole TextEdit.
    let candidates = [
        fonts_dir.join("YuGothR.ttc"),
        fonts_dir.join("BIZ-UDGothicR.ttc"),
        fonts_dir.join("meiryo.ttc"),
        fonts_dir.join("msgothic.ttc"),
        fonts_dir.join("NotoSans-Regular.ttf"),
        // VF last — usable for PDF exports elsewhere, risky as the primary GUI face.
        fonts_dir.join("NotoSansJP-VF.ttf"),
        fonts_dir.join("NotoSansJP-VariableFont_wght.ttf"),
    ];
    let Some(path) = candidates.into_iter().find(|p| p.is_file()) else {
        eprintln!("warn: no CJK-capable TTF/TTC found for GUI preview");
        return;
    };
    let Ok(bytes) = fs::read(&path) else {
        eprintln!("warn: could not read font {}", path.display());
        return;
    };
    let mut fonts = egui::FontDefinitions::default();
    // Keep egui's bundled Latin faces first; CJK is a fallback for both families
    // so monospace / code editors keep stable metrics for ASCII.
    fonts.font_data.insert(
        "reciplexa_cjk".into(),
        Arc::new(egui::FontData::from_owned(bytes)),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("reciplexa_cjk".into());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("reciplexa_cjk".into());
    ctx.set_fonts(fonts);
    eprintln!("gui font: {}", path.display());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_cjk_fonts_runs_on_default_context() {
        let ctx = egui::Context::default();
        // On Windows CI/dev boxes a system CJK face usually exists; elsewhere this
        // takes the early-return "no font found" path — both are coverage wins.
        install_cjk_fonts(&ctx);
    }

    #[test]
    fn install_cjk_fonts_handles_missing_windir_fallback() {
        let prev = env::var_os("WINDIR");
        unsafe {
            env::set_var("WINDIR", r"D:\definitely-missing-windows-root");
        }
        let ctx = egui::Context::default();
        install_cjk_fonts(&ctx);
        unsafe {
            match prev {
                Some(v) => env::set_var("WINDIR", v),
                None => env::remove_var("WINDIR"),
            }
        }
    }

    #[test]
    fn install_cjk_fonts_restores_when_windir_was_unset() {
        let prev = env::var_os("WINDIR");
        unsafe {
            env::remove_var("WINDIR");
        }
        // Point at missing root via default C:\Windows may still find fonts; force miss:
        unsafe {
            env::set_var("WINDIR", r"D:\definitely-missing-windows-root-2");
        }
        let ctx = egui::Context::default();
        install_cjk_fonts(&ctx);
        unsafe {
            match prev {
                Some(v) => env::set_var("WINDIR", v),
                None => env::remove_var("WINDIR"),
            }
        }
    }

    #[test]
    fn install_cjk_fonts_read_failure_on_directory_named_like_font() {
        // is_file() is false for directories, so create a zero-permission file when possible.
        let root = std::env::temp_dir().join("rpx_font_cov");
        let fonts = root.join("Fonts");
        let _ = fs::create_dir_all(&fonts);
        let font_path = fonts.join("YuGothR.ttc");
        // Write then make unreadable by opening exclusive lock (Windows): leave empty file
        // that fs::read can still read — instead use a path that is_file but delete mid-way
        // is racy. Prefer a named pipe-like miss: create file, then replace with directory
        // after candidate selection is hard. Cover read Err by using an invalid reparse.
        // Practical approach: empty file is readable; use ACL deny if available.
        fs::write(&font_path, b"not-a-real-font").unwrap();
        // On Windows, open the file with share-mode none so fs::read fails.
        #[cfg(windows)]
        {
            use std::fs::OpenOptions;
            use std::os::windows::fs::OpenOptionsExt;
            const FILE_SHARE_NONE: u32 = 0;
            let _lock = OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_NONE)
                .open(&font_path)
                .ok();
            let prev = env::var_os("WINDIR");
            unsafe {
                env::set_var("WINDIR", root.as_os_str());
            }
            let ctx = egui::Context::default();
            install_cjk_fonts(&ctx);
            unsafe {
                match prev {
                    Some(v) => env::set_var("WINDIR", v),
                    None => env::remove_var("WINDIR"),
                }
            }
            drop(_lock);
        }
        let _ = fs::remove_dir_all(&root);
    }
}
