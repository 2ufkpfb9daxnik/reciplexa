//! Paragraph-level bidi: logical clusters → visual paint order (Step 11 slice 2).

use crate::position::Direction;
use crate::shape::ShapedGlyph;

/// Byte range with a resolved paragraph direction level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BidiSegment {
    pub start_byte: usize,
    pub end_byte: usize,
    pub direction: Direction,
}

/// Split `text` into directional runs (strong L / strong R; neutrals follow previous).
pub fn analyze_paragraph(text: &str, base: Direction) -> Vec<BidiSegment> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut segments = Vec::new();
    let mut seg_start = 0usize;
    let mut seg_dir = base;
    let mut byte = 0usize;
    for ch in text.chars() {
        let len = ch.len_utf8();
        let end = byte + len;
        if ch == '\n' || ch == '\r' {
            push_segment(&mut segments, seg_start, byte, seg_dir);
            push_segment(&mut segments, byte, end, base);
            seg_start = end;
            seg_dir = base;
        } else if let Some(strong) = strong_direction(ch) {
            if byte > seg_start && strong != seg_dir {
                push_segment(&mut segments, seg_start, byte, seg_dir);
                seg_start = byte;
            }
            seg_dir = strong;
        }
        byte = end;
    }
    if seg_start < text.len() {
        push_segment(&mut segments, seg_start, text.len(), seg_dir);
    }
    segments
}

fn push_segment(segments: &mut Vec<BidiSegment>, start: usize, end: usize, direction: Direction) {
    if start >= end {
        return;
    }
    if let Some(last) = segments.last_mut() {
        if last.direction == direction && last.end_byte == start {
            last.end_byte = end;
            return;
        }
    }
    segments.push(BidiSegment {
        start_byte: start,
        end_byte: end,
        direction,
    });
}

fn strong_direction(ch: char) -> Option<Direction> {
    if ch.is_ascii_alphabetic()
        || ('\u{4E00}'..='\u{9FFF}').contains(&ch)
        || ('\u{3040}'..='\u{309F}').contains(&ch)
        || ('\u{30A0}'..='\u{30FF}').contains(&ch)
    {
        return Some(Direction::Ltr);
    }
    if ('\u{0590}'..='\u{05FF}').contains(&ch)
        || ('\u{0600}'..='\u{06FF}').contains(&ch)
        || ('\u{0750}'..='\u{077F}').contains(&ch)
    {
        return Some(Direction::Rtl);
    }
    None
}

/// Glyph indices in left-to-right visual paint order (clusters preserved).
pub fn visual_glyph_indices(text: &str, glyphs: &[ShapedGlyph], base: Direction) -> Vec<usize> {
    let segments = analyze_paragraph(text, base);
    if segments.is_empty() {
        return (0..glyphs.len()).collect();
    }
    // Latin-only runs in an RTL paragraph paint in reverse visual order.
    if base == Direction::Rtl && segments.iter().all(|s| s.direction == Direction::Ltr) {
        return (0..glyphs.len()).rev().collect();
    }
    let mut visual = Vec::with_capacity(glyphs.len());
    for seg in segments {
        let mut idxs: Vec<usize> = glyphs
            .iter()
            .enumerate()
            .filter(|(_, g)| {
                g.cluster_start >= seg.start_byte
                    && g.cluster_end <= seg.end_byte
                    && g.cluster_start < g.cluster_end
            })
            .map(|(i, _)| i)
            .collect();
        if seg.direction == Direction::Rtl {
            idxs.reverse();
        }
        visual.extend(idxs);
    }
    for (i, g) in glyphs.iter().enumerate() {
        if !visual.contains(&i) && g.cluster_start < g.cluster_end {
            visual.push(i);
        }
    }
    visual
}

/// Assign x offsets in em along the visual progression.
pub fn visual_positions_em(
    text: &str,
    glyphs: &[ShapedGlyph],
    base: Direction,
) -> Vec<(usize, f64)> {
    let order = visual_glyph_indices(text, glyphs, base);
    let mut x = 0.0;
    let mut out = Vec::with_capacity(order.len());
    for i in order {
        out.push((i, x));
        x += glyphs[i].advance_em;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::FontId;

    fn test_glyphs(text: &str) -> Vec<ShapedGlyph> {
        let id = FontId {
            label: "test".into(),
            digest: "test".into(),
        };
        let mut glyphs = Vec::new();
        let mut byte = 0usize;
        for ch in text.chars() {
            let len = ch.len_utf8();
            glyphs.push(ShapedGlyph {
                font_id: id.clone(),
                gid: 1,
                ch,
                cluster_start: byte,
                cluster_end: byte + len,
                advance_em: 1.0,
                italic_correction_em: 0.0,
            });
            byte += len;
        }
        glyphs
    }

    #[test]
    fn embedded_rtl_run_reverses_visual_order() {
        let text = "HiابCD";
        let glyphs = test_glyphs(text);
        let order = visual_glyph_indices(text, &glyphs, Direction::Ltr);
        let chars: String = order.iter().map(|&i| glyphs[i].ch).collect();
        assert_eq!(chars, "HiباCD");
    }

    #[test]
    fn rtl_paragraph_reverses_whole_line() {
        let text = "ABC";
        let glyphs = test_glyphs(text);
        let order = visual_glyph_indices(text, &glyphs, Direction::Rtl);
        let chars: String = order.iter().map(|&i| glyphs[i].ch).collect();
        assert_eq!(chars, "CBA");
    }

    #[test]
    fn japanese_ltr_with_rtl_neighbor() {
        let text = "日اب本";
        let glyphs = test_glyphs(text);
        let order = visual_glyph_indices(text, &glyphs, Direction::Ltr);
        let chars: String = order.iter().map(|&i| glyphs[i].ch).collect();
        assert_eq!(chars, "日با本");
    }
}
