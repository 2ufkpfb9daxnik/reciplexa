//! Font-backed vertical-rl (Step 8 item 2): OpenType `vert` + stacked cells.
//!
//! Tate-chu-yoko / bou are Step 8 item 3 (`layout_tate_chu_yoko`, bou marks).
//! Stub [`reciplexa_std::japanese::lines_to_vertical_text_shapes`] stays the reference.

use reciplexa_scene::{Affine, Color, Shape};
use reciplexa_std::japanese::{
    classify_char, needs_tate_rotation, reciprocal_punctuation_widths_em, CharClass,
};
use ttf_parser::gsub::{SingleSubstitution, SubstitutionSubtable};
use ttf_parser::{GlyphId, Tag};

use crate::error::LayoutError;
use crate::font::LoadedFont;
use crate::position::{
    positioned_glyph_to_shape, Direction, GlyphRun, PositionedGlyph, WritingMode,
};

/// One vertical-rl column: glyphs stacked down the page (Y-up, decreasing y).
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedVertical {
    pub run: GlyphRun,
    pub x_mm: f64,
    pub y_mm: f64,
    pub height_mm: f64,
    /// Per-glyph rotation in degrees (0 = upright, −90 = tate-rotated Latin).
    pub rotation_deg: Vec<f64>,
}

/// Apply GSUB `vert` (then `vrt2`) single substitution. Missing table → `gid`.
pub fn vert_substitute_gid(font: &LoadedFont, gid: u16) -> u16 {
    let face = font.face();
    let Some(gsub) = face.tables().gsub else {
        return gid;
    };
    for tag in [Tag::from_bytes(b"vert"), Tag::from_bytes(b"vrt2")] {
        if let Some(out) = apply_single_feature(&gsub, tag, gid) {
            return out;
        }
    }
    gid
}

/// Ink offset within a vertical cell (em). Stack anchor `y` is the cell top (+Y).
///
/// Reciprocal punctuation: solid half toward the preceding glyph (+Y), so paint
/// moves down into the upper half (`dy` negative) and left (`dx` negative).
pub fn vert_glyph_paint_offset_em(ch: char) -> (f64, f64) {
    if needs_vert_cell_rotation(ch) {
        return (0.0, 0.0);
    }
    if let Some((solid, mirror)) = reciprocal_punctuation_widths_em(ch) {
        return (-mirror * 0.5, -solid * 0.5);
    }
    (0.0, 0.0)
}

/// Rotation for vertical-rl: Latin and prolonged sound mark (−90°).
pub fn vert_glyph_rotation_deg(ch: char) -> f64 {
    if needs_vert_cell_rotation(ch) {
        -90.0
    } else {
        0.0
    }
}

fn needs_vert_cell_rotation(ch: char) -> bool {
    needs_tate_rotation(ch) || classify_char(ch) == CharClass::ProlongedSoundMark
}

fn apply_single_feature(
    gsub: &ttf_parser::opentype_layout::LayoutTable<'_>,
    tag: Tag,
    gid: u16,
) -> Option<u16> {
    let feature = gsub.features.find(tag).or_else(|| {
        (0..gsub.features.len()).find_map(|i| {
            let f = gsub.features.get(i)?;
            (f.tag == tag).then_some(f)
        })
    })?;
    for i in 0..feature.lookup_indices.len() {
        let lookup_index = feature.lookup_indices.get(i)?;
        let lookup = gsub.lookups.get(lookup_index)?;
        for sub in lookup.subtables.into_iter::<SubstitutionSubtable>() {
            if let SubstitutionSubtable::Single(single) = sub {
                if let Some(out) = apply_single(single, gid) {
                    return Some(out);
                }
            }
        }
    }
    None
}

fn apply_single(ss: SingleSubstitution<'_>, gid: u16) -> Option<u16> {
    let g = GlyphId(gid);
    match ss {
        SingleSubstitution::Format1 { coverage, delta } => {
            if coverage.contains(g) {
                let next = i32::from(gid) + i32::from(delta);
                u16::try_from(next).ok()
            } else {
                None
            }
        }
        SingleSubstitution::Format2 {
            coverage,
            substitutes,
        } => {
            let idx = coverage.get(g)?;
            substitutes.get(idx).map(|s| s.0)
        }
    }
}

/// Shape `text` as a vertical-rl column. First character is at `origin_y_mm`.
pub fn layout_vertical_run(
    font: &LoadedFont,
    text: &str,
    origin_x_mm: f64,
    origin_y_mm: f64,
    size_mm: f64,
) -> Result<PositionedVertical, LayoutError> {
    if text.chars().all(|c| c == '\n' || c == '\r') {
        return Err(LayoutError::Engine {
            detail: "vertical run is empty".into(),
        });
    }
    let mut glyphs = Vec::new();
    let mut rotation_deg = Vec::new();
    let mut y = origin_y_mm;
    let mut byte = 0usize;
    for ch in text.chars() {
        let len = ch.len_utf8();
        if ch == '\n' || ch == '\r' {
            byte += len;
            continue;
        }
        let cmap = font.glyph_id(ch)?;
        let gid = vert_substitute_gid(font, cmap);
        let rotated = needs_vert_cell_rotation(ch);
        let advance_em = if rotated {
            font.hor_advance_em_gid(gid)?
        } else {
            font.ver_advance_em_gid(gid)
        };
        let advance_mm = advance_em * size_mm;
        let (dx_em, dy_em) = vert_glyph_paint_offset_em(ch);
        glyphs.push(PositionedGlyph {
            gid,
            ch,
            cluster_start: byte,
            cluster_end: byte + len,
            x_mm: origin_x_mm + dx_em * size_mm,
            y_mm: y + dy_em * size_mm,
            advance_mm,
            font_digest: font.id.digest.clone(),
        });
        rotation_deg.push(vert_glyph_rotation_deg(ch));
        y -= advance_mm;
        byte += len;
    }
    let height_mm = origin_y_mm - y;
    let content: String = glyphs.iter().map(|g| g.ch).collect();
    Ok(PositionedVertical {
        run: GlyphRun {
            font: font.id.clone(),
            size_mm,
            direction: Direction::Ltr,
            writing_mode: WritingMode::VerticalRl,
            language: "ja".into(),
            content,
            glyphs,
        },
        x_mm: origin_x_mm,
        y_mm: origin_y_mm,
        height_mm,
        rotation_deg,
    })
}

/// Lower to [`Shape::GlyphRun`], wrapping tate-rotated Latin in a −90° group.
pub fn positioned_vertical_to_shapes(col: &PositionedVertical, fill: Color) -> Vec<Shape> {
    col.run
        .glyphs
        .iter()
        .zip(col.rotation_deg.iter())
        .map(|(g, rot)| {
            let paint = positioned_glyph_to_shape(g, col.run.size_mm, fill);
            if rot.abs() < 1e-9 {
                paint
            } else {
                let size = col.run.size_mm;
                let cx = g.x_mm + size * 0.5;
                let cy = g.y_mm + size * 0.5;
                Shape::Group {
                    transform: Affine::translate(-cx, -cy)
                        .then(Affine::rotate_deg(*rot))
                        .then(Affine::translate(cx, cy)),
                    children: vec![paint],
                }
            }
        })
        .collect()
}
