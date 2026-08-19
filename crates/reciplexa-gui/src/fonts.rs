//! Install the same layout faces the host PDF path uses into egui.

use std::sync::Arc;

use eframe::egui;
use reciplexa_text_layout::{host_math_font, host_product_font, LoadedFont};

/// Load [`host_product_font`] and [`host_math_font`] into egui.
///
/// Preview `GlyphRun` families are keyed by layout digest. The same faces are
/// appended as CJK fallbacks for Proportional/Monospace so the source editor
/// keeps egui's Latin metrics first.
pub fn install_cjk_fonts(ctx: &egui::Context) {
    ctx.set_fonts(host_layout_font_definitions());
}

/// Font definitions sharing digest identity with PDF `host_*_font`.
pub fn host_layout_font_definitions() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    match host_product_font() {
        Ok(font) => insert_layout_face(&mut fonts, &font),
        Err(e) => eprintln!("warn: host product font: {e}"),
    }
    match host_math_font() {
        Ok(font) => insert_layout_face(&mut fonts, &font),
        Err(e) => eprintln!("warn: host math font: {e}"),
    }
    fonts
}

/// Named family whose only face is the layout font with this digest.
pub fn layout_font_family(digest: &str) -> egui::FontFamily {
    egui::FontFamily::Name(digest.into())
}

fn insert_layout_face(fonts: &mut egui::FontDefinitions, font: &LoadedFont) {
    let key = font.id.digest.clone();
    if fonts.font_data.contains_key(&key) {
        return;
    }
    let mut data = egui::FontData::from_owned(font.bytes().to_vec());
    data.index = font.face_index();
    fonts.font_data.insert(key.clone(), Arc::new(data));
    fonts
        .families
        .insert(layout_font_family(&key), vec![key.clone()]);
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push(key.clone());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push(key);
    eprintln!("gui font: {}", font.id.as_key());
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_text_layout::host_product_font;

    #[test]
    fn install_registers_host_product_digest() {
        let defs = host_layout_font_definitions();
        let digest = host_product_font().expect("product font").id.digest;
        assert!(
            defs.font_data.contains_key(&digest),
            "layout digest {digest} missing from egui font_data"
        );
        assert_eq!(
            defs.families.get(&layout_font_family(&digest)),
            Some(&vec![digest.clone()])
        );
        let proportional = defs
            .families
            .get(&egui::FontFamily::Proportional)
            .expect("proportional");
        assert_ne!(
            proportional.first().map(String::as_str),
            Some(digest.as_str())
        );
        assert!(proportional.contains(&digest));
    }

    #[test]
    fn install_cjk_fonts_runs_on_default_context() {
        let ctx = egui::Context::default();
        install_cjk_fonts(&ctx);
    }
}
