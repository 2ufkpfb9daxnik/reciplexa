//! In-process shaping: cmap + hmtx clusters (no HarfBuzz).

use crate::error::LayoutError;
use crate::font::{FontId, LoadedFont};

/// One shaped glyph tied to a source Unicode cluster.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedGlyph {
    /// Face that supplied this glyph (fallback chains may mix digests).
    pub font_id: FontId,
    pub gid: u16,
    pub ch: char,
    /// UTF-8 byte start in the input (inclusive).
    pub cluster_start: usize,
    /// UTF-8 byte end in the input (exclusive).
    pub cluster_end: usize,
    /// Advance width in em.
    pub advance_em: f64,
    /// Italic correction in em (MATH, else 0).
    pub italic_correction_em: f64,
}

/// A shaped run covering `text` completely (no loss / overlap).
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedRun {
    /// Primary face for the run (chain head); per-glyph `font_id` may differ.
    pub primary_font_id: FontId,
    pub text: String,
    pub glyphs: Vec<ShapedGlyph>,
}

impl ShapedRun {
    pub fn width_em(&self) -> f64 {
        self.glyphs.iter().map(|g| g.advance_em).sum()
    }

    /// Primary chain head (`FontId` before Step 11 per-glyph fallback).
    pub fn font_id(&self) -> &FontId {
        &self.primary_font_id
    }
}

/// Shape `text` with 1:1 clusters (each scalar → one glyph). Missing glyphs fail.
pub fn shape_run(font: &LoadedFont, text: &str) -> Result<ShapedRun, LayoutError> {
    shape_run_single_font(font, text)
}

/// Shape with a single face (internal helper for fallback chain).
pub fn shape_run_single_font(font: &LoadedFont, text: &str) -> Result<ShapedRun, LayoutError> {
    let mut glyphs = Vec::new();
    let mut byte = 0usize;
    for ch in text.chars() {
        let len = ch.len_utf8();
        if ch == '\n' || ch == '\r' {
            byte += len;
            continue;
        }
        let gid = font.glyph_id(ch)?;
        let advance_em = font.hor_advance_em(ch)?;
        let italic_correction_em = font.italic_correction_em(ch).unwrap_or(0.0);
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
        primary_font_id: font.id.clone(),
        text: text.to_string(),
        glyphs,
    })
}

/// Clusters cover `[0, text.len())` without gaps or overlap (CR/LF skipped).
pub fn clusters_cover_input(run: &ShapedRun) -> bool {
    let mut covered = vec![false; run.text.len()];
    for g in &run.glyphs {
        if g.cluster_end > run.text.len() || g.cluster_start >= g.cluster_end {
            return false;
        }
        for slot in covered.iter_mut().take(g.cluster_end).skip(g.cluster_start) {
            if *slot {
                return false;
            }
            *slot = true;
        }
    }
    for (i, ch) in run.text.char_indices() {
        let span = i..i + ch.len_utf8();
        if ch == '\n' || ch == '\r' {
            continue;
        }
        if span.clone().any(|j| !covered[j]) {
            return false;
        }
    }
    true
}
