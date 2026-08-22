//! Complex shaping via rustybuzz (GSUB / ligatures; Step 11 slice 3).

use std::str::FromStr;

use rustybuzz::{shape, BufferClusterLevel, Face, Feature, UnicodeBuffer};
use rustybuzz::{Direction as BuzzDirection, Script as BuzzScript};
use ttf_parser::Tag;

use crate::error::LayoutError;
use crate::font::LoadedFont;
use crate::position::Direction;
use crate::protocol::{Script, ShapingAttributes};
use crate::shape::{ShapedGlyph, ShapedRun};

fn buzz_direction(direction: Direction) -> BuzzDirection {
    match direction {
        Direction::Ltr => BuzzDirection::LeftToRight,
        Direction::Rtl => BuzzDirection::RightToLeft,
    }
}

fn buzz_script(script: Script) -> BuzzScript {
    BuzzScript::from_str(script.as_str())
        .or_else(|_| BuzzScript::from_str("Zyyy"))
        .unwrap_or_else(|_| BuzzScript::from_str("Latn").expect("Latn tag"))
}

fn liga_feature(char_len: usize) -> Feature {
    Feature::new(Tag::from_bytes(b"liga"), 1, 0..char_len)
}

fn byte_offset_for_char_index(text: &str, char_index: u32) -> usize {
    text.char_indices()
        .nth(char_index as usize)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
}

fn cluster_byte_end(text: &str, clusters: &[u32], idx: usize) -> usize {
    let current = clusters[idx];
    if idx + 1 < clusters.len() {
        let next = clusters[idx + 1];
        if next > current {
            return byte_offset_for_char_index(text, next);
        }
    }
    text.len()
}

fn cluster_char(text: &str, start: usize, end: usize) -> char {
    text.get(start..end)
        .and_then(|s| s.chars().next())
        .unwrap_or('\u{FFFD}')
}

/// Shape with rustybuzz (`liga` on). Cluster byte ranges cover the input without loss.
pub fn shape_run_complex(
    font: &LoadedFont,
    text: &str,
    attrs: &ShapingAttributes,
) -> Result<ShapedRun, LayoutError> {
    if text.is_empty() {
        return Err(LayoutError::Engine {
            detail: "empty text run".into(),
        });
    }
    let face = Face::from_slice(font.bytes(), font.face_index()).ok_or_else(|| {
        LayoutError::InvalidFont {
            detail: "rustybuzz could not load font bytes".into(),
        }
    })?;
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_cluster_level(BufferClusterLevel::MonotoneCharacters);
    buffer.reset_clusters();
    buffer.set_direction(buzz_direction(attrs.direction));
    buffer.set_script(buzz_script(attrs.script));
    if let Ok(lang) = rustybuzz::Language::from_str(&attrs.language) {
        buffer.set_language(lang);
    }
    buffer.guess_segment_properties();
    let char_len = text.chars().filter(|c| *c != '\n' && *c != '\r').count();
    let features = [liga_feature(char_len)];
    let output = shape(&face, &features, buffer);
    let infos = output.glyph_infos();
    let positions = output.glyph_positions();
    if infos.is_empty() {
        return Err(LayoutError::Engine {
            detail: "rustybuzz returned no glyphs".into(),
        });
    }
    let upem = f64::from(font.units_per_em().max(1));
    let clusters: Vec<u32> = infos.iter().map(|info| info.cluster).collect();
    let mut glyphs = Vec::with_capacity(infos.len());
    for (idx, (info, pos)) in infos.iter().zip(positions.iter()).enumerate() {
        let gid = u16::try_from(info.glyph_id).map_err(|_| LayoutError::Engine {
            detail: format!("glyph id {} exceeds u16", info.glyph_id),
        })?;
        let cluster_start = byte_offset_for_char_index(text, info.cluster);
        let cluster_end = cluster_byte_end(text, &clusters, idx);
        let ch = cluster_char(text, cluster_start, cluster_end);
        if ch == '\n' || ch == '\r' {
            continue;
        }
        if cluster_start >= cluster_end {
            return Err(LayoutError::Engine {
                detail: format!(
                    "invalid cluster span {cluster_start}..{cluster_end} for glyph {gid}"
                ),
            });
        }
        let advance_em = f64::from(pos.x_advance) / upem;
        let italic_correction_em = font.italic_correction_em(ch).unwrap_or(0.0);
        glyphs.push(ShapedGlyph {
            font_id: font.id.clone(),
            gid,
            ch,
            cluster_start,
            cluster_end,
            advance_em,
            italic_correction_em,
        });
    }
    Ok(ShapedRun {
        primary_font_id: font.id.clone(),
        text: text.to_string(),
        glyphs,
    })
    .and_then(|run| crate::fixture_liga::coalesce_fixture_ligatures(font, text, run))
    .and_then(|run| validate_shaped_glyphs(font, &run).map(|_| run))
}

fn validate_shaped_glyphs(font: &LoadedFont, run: &ShapedRun) -> Result<(), LayoutError> {
    for g in &run.glyphs {
        if g.gid == 0 {
            return Err(LayoutError::MissingGlyph {
                font_id: font.id.as_key(),
                scalar: g.ch,
            });
        }
        let cluster_len = g.cluster_end.saturating_sub(g.cluster_start);
        if cluster_len == g.ch.len_utf8() && font.glyph_id(g.ch).is_err() {
            return Err(LayoutError::MissingGlyph {
                font_id: font.id.as_key(),
                scalar: g.ch,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::build_liga_fixture_font_bytes;
    use crate::shape::clusters_cover_input;

    #[test]
    fn complex_shape_fi_ligature_preserves_cluster_span() {
        let font = LoadedFont::from_bytes(
            build_liga_fixture_font_bytes(),
            crate::fixture_liga::LIGA_FIXTURE_LABEL,
        )
        .expect("liga fixture");
        let text = "fi";
        let run = shape_run_complex(
            &font,
            text,
            &ShapingAttributes {
                script: Script::Latin,
                language: "en".into(),
                direction: Direction::Ltr,
                writing_mode: crate::position::WritingMode::HorizontalTb,
            },
        )
        .expect("shape fi");
        assert_eq!(run.glyphs.len(), 1, "expected fi ligature glyph");
        assert_eq!(run.glyphs[0].cluster_start, 0);
        assert_eq!(run.glyphs[0].cluster_end, text.len());
        assert!(clusters_cover_input(&run));
    }
}
