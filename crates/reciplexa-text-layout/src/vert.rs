//! Font-backed vertical-rl (Step 8 item 2): OpenType `vert` + stacked cells.
//!
//! Tate-chu-yoko / bou are Step 8 item 3 (`layout_tate_chu_yoko`, bou marks).
//! Stub [`reciplexa_std::japanese::lines_to_vertical_text_shapes`] stays the reference.

use reciplexa_scene::{Affine, Color, Shape};
use reciplexa_std::japanese::{
    classify_char, needs_tate_rotation, reciprocal_punctuation_widths_em, CharClass,
};
use rustybuzz::{shape, BufferClusterLevel, Face, Feature, UnicodeBuffer};
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

/// Apply GSUB `vert` / `vrt2` via rustybuzz, then fall back to single-subst scan.
pub fn vert_substitute_gid(font: &LoadedFont, gid: u16) -> u16 {
    vert_substitute_gid_for_char(font, '\0', gid)
}

/// Shape one scalar with the `vert` feature when the font provides a substitute.
pub fn vert_shape_gid(font: &LoadedFont, ch: char) -> Result<u16, LayoutError> {
    let cmap = font.glyph_id(ch)?;
    Ok(vert_substitute_gid_for_char(font, ch, cmap))
}

fn vert_substitute_gid_for_char(font: &LoadedFont, ch: char, cmap: u16) -> u16 {
    if ch != '\0' {
        if let Some(gid) = rustybuzz_vert_gid(font, ch) {
            return gid;
        }
    }
    manual_vert_substitute_gid(font, cmap)
}

fn rustybuzz_vert_gid(font: &LoadedFont, ch: char) -> Option<u16> {
    let face = Face::from_slice(font.bytes(), font.face_index())?;
    let text = ch.to_string();
    for tag in [b"vert", b"vrt2"] {
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(&text);
        buffer.set_cluster_level(BufferClusterLevel::MonotoneCharacters);
        buffer.reset_clusters();
        buffer.guess_segment_properties();
        let features = [Feature::new(Tag::from_bytes(tag), 1, 0..1)];
        let output = shape(&face, &features, buffer);
        let gid = output.glyph_infos().first()?.glyph_id;
        let gid = u16::try_from(gid).ok()?;
        let cmap = font.glyph_id(ch).ok()?;
        if gid != cmap {
            return Some(gid);
        }
    }
    None
}

fn manual_vert_substitute_gid(font: &LoadedFont, gid: u16) -> u16 {
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
/// Fullwidth reciprocal punctuation: JLReq vertical ink at body top-right; horizontal
/// glyph metrics place ink at bottom-left, so shift by the full solid/mirror halves.
pub fn vert_glyph_paint_offset_em(ch: char) -> (f64, f64) {
    if needs_tate_rotation(ch) || classify_char(ch) == CharClass::ProlongedSoundMark {
        // Tate-rotated Latin and ー: the −90° cell sits one em left and down of
        // the stack pen (Y-up: −X, −Y). Rotate about this cell's center.
        return (-1.0, -1.0);
    }
    if let Some((solid, mirror)) = reciprocal_punctuation_widths_em(ch) {
        return (mirror, solid);
    }
    (0.0, 0.0)
}

/// Em-box center for a glyph whose paint pen is already at `(x_mm, y_mm)`.
///
/// Paint offset is included in the pen; this does not add it back.
pub fn vert_glyph_cell_center_mm(x_mm: f64, y_mm: f64, _ch: char, size_mm: f64) -> (f64, f64) {
    (x_mm + size_mm * 0.5, y_mm + size_mm * 0.5)
}

/// Rotation for vertical-rl (−90° clockwise per JLReq tate-mochi).
///
/// Reciprocal punctuation (`。` `、` …) stays upright: OpenType `vert` supplies the
/// vertical glyph and [`vert_glyph_paint_offset_em`] shifts ink inside the cell.
pub fn vert_glyph_rotation_deg(ch: char, vert_substituted: bool) -> f64 {
    if needs_vert_cell_rotation(ch, vert_substituted) {
        -90.0
    } else {
        0.0
    }
}

fn needs_vert_cell_rotation(ch: char, vert_substituted: bool) -> bool {
    if needs_tate_rotation(ch) {
        return true;
    }
    // Prolonged sound: prefer `vert` glyph; rotate only when the font has no substitute.
    classify_char(ch) == CharClass::ProlongedSoundMark && !vert_substituted
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
        let gid = vert_shape_gid(font, ch)?;
        let vert_substituted = gid != cmap;
        let rotated = needs_vert_cell_rotation(ch, vert_substituted);
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
        rotation_deg.push(vert_glyph_rotation_deg(ch, vert_substituted));
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

/// Lower to [`Shape::GlyphRun`], wrapping tate-rotated glyphs in a −90° group.
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
                let (cx, cy) = vert_glyph_cell_center_mm(g.x_mm, g.y_mm, g.ch, size);
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
