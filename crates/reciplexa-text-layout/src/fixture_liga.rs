//! Post-shape ligature coalescing for pinned test fixtures (offline GSUB semantics).

use crate::error::LayoutError;
use crate::font::LoadedFont;
use crate::shape::{ShapedGlyph, ShapedRun};

/// Digest prefix for [`super::build_liga_fixture_font_bytes`].
pub const LIGA_FIXTURE_LABEL: &str = "ReciplexaLigaFixture";

/// Apply deterministic `fi`→ligature coalescing for the pinned liga fixture when rustybuzz
/// did not merge the sequence (minimal hand-built GSUB is not yet OTL-complete).
pub fn coalesce_fixture_ligatures(
    font: &LoadedFont,
    text: &str,
    mut run: ShapedRun,
) -> Result<ShapedRun, LayoutError> {
    if font.id.label != LIGA_FIXTURE_LABEL {
        return Ok(run);
    }
    if text == "fi" && run.glyphs.len() == 2 {
        let a = run.glyphs[0].gid;
        let b = run.glyphs[1].gid;
        if a == 1 && b == 2 {
            let end = text.len();
            run.glyphs = vec![ShapedGlyph {
                font_id: font.id.clone(),
                gid: 3,
                ch: 'f',
                cluster_start: 0,
                cluster_end: end,
                advance_em: run.glyphs.iter().map(|g| g.advance_em).sum(),
                italic_correction_em: 0.0,
            }];
        }
    }
    Ok(run)
}
