//! Deterministic font fallback (spec `FontFallbackPolicy` spirit).

use crate::error::LayoutError;
use crate::font::LoadedFont;
use crate::protocol::ShapingAttributes;
use crate::shape::{shape_run_single_font, ShapedGlyph, ShapedRun};

/// Ordered font chain. Same chain + text → same per-glyph font choices.
#[derive(Debug, Clone)]
pub struct FontFallbackChain {
    fonts: Vec<LoadedFont>,
}

impl FontFallbackChain {
    pub fn new(primary: LoadedFont) -> Self {
        Self {
            fonts: vec![primary],
        }
    }

    pub fn with_fallback(mut self, font: LoadedFont) -> Self {
        if !self.fonts.iter().any(|f| f.id.digest == font.id.digest) {
            self.fonts.push(font);
        }
        self
    }

    pub fn primary(&self) -> &LoadedFont {
        &self.fonts[0]
    }

    pub fn fonts(&self) -> &[LoadedFont] {
        &self.fonts
    }

    /// Shape `text`, resolving each scalar against the chain in order.
    pub fn shape(&self, text: &str, _attrs: &ShapingAttributes) -> Result<ShapedRun, LayoutError> {
        if self.fonts.is_empty() {
            return Err(LayoutError::Engine {
                detail: "empty font fallback chain".into(),
            });
        }
        if self.fonts.len() == 1 {
            return shape_run_single_font(&self.fonts[0], text);
        }
        let primary_id = self.fonts[0].id.clone();
        let mut glyphs = Vec::new();
        let mut byte = 0usize;
        for ch in text.chars() {
            let len = ch.len_utf8();
            if ch == '\n' || ch == '\r' {
                byte += len;
                continue;
            }
            let (font, gid, advance_em, italic_correction_em) = resolve_glyph(self.fonts(), ch)?;
            glyphs.push(ShapedGlyph {
                font_id: font.id.clone(),
                gid,
                ch,
                cluster_start: byte,
                cluster_end: byte + len,
                advance_em,
                italic_correction_em,
            });
            byte += len;
        }
        debug_assert_eq!(byte, text.len());
        Ok(ShapedRun {
            primary_font_id: primary_id,
            text: text.to_string(),
            glyphs,
        })
    }
}

fn resolve_glyph(
    fonts: &[LoadedFont],
    ch: char,
) -> Result<(&LoadedFont, u16, f64, f64), LayoutError> {
    let mut last_err = None;
    for font in fonts {
        match font.glyph_id(ch) {
            Ok(gid) => {
                let advance_em = font.hor_advance_em(ch)?;
                let italic_correction_em = font.italic_correction_em(ch).unwrap_or(0.0);
                return Ok((font, gid, advance_em, italic_correction_em));
            }
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| LayoutError::Engine {
        detail: "empty font chain".into(),
    }))
}

/// Register every face from `chain` that supplied glyphs in `run`.
pub fn register_run_fonts(
    registry: &mut crate::policy::FontRegistry,
    chain: &FontFallbackChain,
    run: &ShapedRun,
) {
    use std::collections::BTreeSet;
    let digests: BTreeSet<&str> = run
        .glyphs
        .iter()
        .map(|g| g.font_id.digest.as_str())
        .collect();
    for font in chain.fonts() {
        if digests.contains(font.id.digest.as_str()) {
            registry.insert(font.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::build_fallback_only_font_bytes;
    use crate::shape::clusters_cover_input;

    #[test]
    fn fallback_chain_is_deterministic() {
        let primary = LoadedFont::fixture();
        let fallback = LoadedFont::from_bytes(
            build_fallback_only_font_bytes().to_vec(),
            "ReciplexaFallbackOnly",
        )
        .expect("fallback font");
        let chain = FontFallbackChain::new(primary.clone()).with_fallback(fallback.clone());
        let text = "A☺";
        let a1 = chain
            .shape(text, &ShapingAttributes::default())
            .expect("shape");
        let a2 = chain
            .shape(text, &ShapingAttributes::default())
            .expect("shape");
        assert_eq!(a1, a2);
        assert!(clusters_cover_input(&a1));
        assert_eq!(a1.glyphs.len(), 2);
        assert_eq!(a1.glyphs[0].font_id.digest, primary.id.digest);
        assert_eq!(a1.glyphs[1].font_id.digest, fallback.id.digest);
    }

    #[test]
    fn missing_glyph_when_chain_exhausted() {
        let chain = FontFallbackChain::new(LoadedFont::fixture());
        let err = chain
            .shape("☺", &ShapingAttributes::default())
            .expect_err("no fallback");
        assert!(matches!(err, LayoutError::MissingGlyph { scalar: '☺', .. }));
    }
}
