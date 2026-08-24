//! Font-backed horizontal and vertical ruby (Step 8 item 1; Step 12 slices 2–4).
//!
//! Stub [`reciplexa_std::japanese::Ruby::estimate_box`] stays the fontless reference.

use reciplexa_scene::{Color, Shape};
use reciplexa_std::japanese::{Ruby, RubyKind, RUBY_ANNOTATION_SCALE, VERTICAL_RUBY_SIDE_EM};

use crate::error::LayoutError;
use crate::font::LoadedFont;
use crate::position::{positioned_line_to_glyph_shapes, GlyphRun, PositionedGlyph, PositionedLine};
use crate::shape::shape_run;
use crate::vert::{layout_vertical_run, positioned_vertical_to_shapes, PositionedVertical};

/// Extra em between the parent em-square top and the annotation baseline.
/// Stub `RUBY_HEIGHT_BUMP_EM` stays the fontless box; this is product spacing.
pub const RUBY_PARENT_GAP_EM: f64 = 0.25;

/// Positioned horizontal ruby: base line plus annotation line above it (page Y-up).
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedRuby {
    pub base: PositionedLine,
    pub annotation: PositionedLine,
    /// Inline measure advance (mm): parent base width (JLReq 親文字送り).
    pub advance_mm: f64,
    /// Full ink width including annotation overhang: `max(base, annotation)`.
    pub ink_width_mm: f64,
}

/// Split `annotation` across `base` grapheme clusters for jukugo placement.
///
/// Uses proportional character-count buckets (JLReq product subset; not morphological).
pub fn distribute_jukugo_annotation(base: &str, annotation: &str) -> Vec<String> {
    let base_len = base.chars().count();
    let ann_chars: Vec<char> = annotation.chars().collect();
    if base_len == 0 || ann_chars.is_empty() {
        return Vec::new();
    }
    let na = ann_chars.len();
    (0..base_len)
        .map(|i| {
            let start = i * na / base_len;
            let end = (i + 1) * na / base_len;
            ann_chars[start..end].iter().collect()
        })
        .collect()
}

/// Layout simple or jukugo ruby with font advances.
pub fn layout_ruby(
    font: &LoadedFont,
    ruby: &Ruby,
    origin_x_mm: f64,
    origin_y_mm: f64,
    base_size_mm: f64,
) -> Result<PositionedRuby, LayoutError> {
    if ruby.base.is_empty() || ruby.annotation.is_empty() {
        return Err(LayoutError::Engine {
            detail: "ruby base and annotation must be non-empty".into(),
        });
    }
    match ruby.kind {
        RubyKind::Simple => {
            layout_simple_ruby_inner(font, ruby, origin_x_mm, origin_y_mm, base_size_mm)
        }
        RubyKind::Jukugo => {
            layout_jukugo_ruby_inner(font, ruby, origin_x_mm, origin_y_mm, base_size_mm)
        }
    }
}

/// Layout [`RubyKind::Simple`] with font advances.
///
/// Annotation is drawn at [`RUBY_ANNOTATION_SCALE`] of `base_size_mm`, centered
/// on the parent base width. Its baseline sits one parent em plus
/// [`RUBY_PARENT_GAP_EM`] above the parent baseline.
///
/// Line measure uses the base width only; wider annotations overhang (親文字送り).
pub fn layout_simple_ruby(
    font: &LoadedFont,
    ruby: &Ruby,
    origin_x_mm: f64,
    origin_y_mm: f64,
    base_size_mm: f64,
) -> Result<PositionedRuby, LayoutError> {
    layout_ruby(font, ruby, origin_x_mm, origin_y_mm, base_size_mm)
}

fn layout_simple_ruby_inner(
    font: &LoadedFont,
    ruby: &Ruby,
    origin_x_mm: f64,
    origin_y_mm: f64,
    base_size_mm: f64,
) -> Result<PositionedRuby, LayoutError> {
    let base_run = shape_run(font, &ruby.base)?;
    let ann_run = shape_run(font, &ruby.annotation)?;
    let ann_size = base_size_mm * RUBY_ANNOTATION_SCALE;
    let base_w = base_run.width_em() * base_size_mm;
    let ann_w = ann_run.width_em() * ann_size;
    let advance_mm = base_w;
    let ink_width_mm = base_w.max(ann_w);
    let base_x = origin_x_mm;
    let ann_x = origin_x_mm + (base_w - ann_w) / 2.0;
    let ann_y = origin_y_mm + base_size_mm * (1.0 + RUBY_PARENT_GAP_EM);
    Ok(PositionedRuby {
        base: PositionedLine::from_shaped_run(
            font.id.clone(),
            &base_run,
            base_x,
            origin_y_mm,
            base_size_mm,
            "ja",
        ),
        annotation: PositionedLine::from_shaped_run(
            font.id.clone(),
            &ann_run,
            ann_x,
            ann_y,
            ann_size,
            "ja",
        ),
        advance_mm,
        ink_width_mm,
    })
}

fn layout_jukugo_ruby_inner(
    font: &LoadedFont,
    ruby: &Ruby,
    origin_x_mm: f64,
    origin_y_mm: f64,
    base_size_mm: f64,
) -> Result<PositionedRuby, LayoutError> {
    let segments = distribute_jukugo_annotation(&ruby.base, &ruby.annotation);
    let base_chars: Vec<char> = ruby.base.chars().collect();
    if segments.len() != base_chars.len() {
        return Err(LayoutError::Engine {
            detail: "jukugo ruby distribution failed".into(),
        });
    }
    let base_run = shape_run(font, &ruby.base)?;
    let ann_size = base_size_mm * RUBY_ANNOTATION_SCALE;
    let ann_y = origin_y_mm + base_size_mm * (1.0 + RUBY_PARENT_GAP_EM);
    let base_w = base_run.width_em() * base_size_mm;
    let advance_mm = base_w;

    let base = PositionedLine::from_shaped_run(
        font.id.clone(),
        &base_run,
        origin_x_mm,
        origin_y_mm,
        base_size_mm,
        "ja",
    );

    let mut ann_glyphs = Vec::new();
    let mut byte_off = 0usize;
    let mut ink_right = origin_x_mm;

    let base_glyphs: Vec<_> = base.run.glyphs.iter().collect();
    if base_glyphs.len() != segments.len() {
        return Err(LayoutError::Engine {
            detail: format!(
                "jukugo ruby base glyph count {} != char count {}",
                base_glyphs.len(),
                segments.len()
            ),
        });
    }

    for (seg, parent) in segments.iter().zip(base_glyphs.iter()) {
        let cell_w = parent.advance_mm;
        let seg_run = shape_run(font, seg)?;
        let seg_w = seg_run.width_em() * ann_size;
        let seg_x = parent.x_mm + (cell_w - seg_w) / 2.0;
        let line = PositionedLine::from_shaped_run(
            font.id.clone(),
            &seg_run,
            seg_x,
            ann_y,
            ann_size,
            "ja",
        );
        for g in line.run.glyphs {
            ink_right = ink_right.max(g.x_mm + g.advance_mm);
            ann_glyphs.push(PositionedGlyph {
                cluster_start: g.cluster_start + byte_off,
                cluster_end: g.cluster_end + byte_off,
                ..g
            });
        }
        byte_off += seg.len();
    }

    let ink_width_mm = (ink_right - origin_x_mm).max(base_w);

    let annotation = PositionedLine {
        run: GlyphRun {
            font: font.id.clone(),
            size_mm: ann_size,
            direction: base.run.direction,
            writing_mode: base.run.writing_mode,
            language: "ja".into(),
            content: ruby.annotation.clone(),
            glyphs: ann_glyphs,
        },
        x_mm: origin_x_mm,
        y_mm: ann_y,
        width_mm: ink_width_mm,
    };

    Ok(PositionedRuby {
        base,
        annotation,
        advance_mm,
        ink_width_mm,
    })
}

/// Lower [`PositionedRuby`] to per-glyph [`Shape::GlyphRun`].
pub fn positioned_ruby_to_shapes(ruby: &PositionedRuby, fill: Color) -> Vec<Shape> {
    let mut shapes = positioned_line_to_glyph_shapes(&ruby.annotation, fill);
    shapes.extend(positioned_line_to_glyph_shapes(&ruby.base, fill));
    shapes
}

/// Vertical-rl ruby: base column plus side annotation (page Y-up, column stacks downward).
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedVerticalRuby {
    pub base: PositionedVertical,
    pub annotation: PositionedVertical,
    /// Vertical measure advance (mm): parent base column height (親文字送り).
    pub advance_mm: f64,
    /// Inline extent (mm) including the side ruby band.
    pub inline_mm: f64,
}

/// Layout [`RubyKind::Simple`] in a vertical-rl column with font-backed side annotation.
///
/// Annotation is drawn at [`RUBY_ANNOTATION_SCALE`] beside the base column
/// ([`VERTICAL_RUBY_SIDE_EM`] gap). Vertical measure uses the base height only;
/// taller annotations overhang at the column ends.
pub fn layout_vertical_ruby(
    font: &LoadedFont,
    ruby: &Ruby,
    origin_x_mm: f64,
    origin_y_mm: f64,
    base_size_mm: f64,
) -> Result<PositionedVerticalRuby, LayoutError> {
    if ruby.kind != RubyKind::Simple {
        return Err(LayoutError::Engine {
            detail: format!(
                "vertical-ruby slice does not layout kind `{}`",
                ruby.kind.as_str()
            ),
        });
    }
    if ruby.base.is_empty() || ruby.annotation.is_empty() {
        return Err(LayoutError::Engine {
            detail: "ruby base and annotation must be non-empty".into(),
        });
    }
    let base = layout_vertical_run(font, &ruby.base, origin_x_mm, origin_y_mm, base_size_mm)?;
    let ann_size = base_size_mm * RUBY_ANNOTATION_SCALE;
    let ann_x = origin_x_mm + base_size_mm + base_size_mm * VERTICAL_RUBY_SIDE_EM;
    let probe = layout_vertical_run(font, &ruby.annotation, ann_x, origin_y_mm, ann_size)?;
    let ann_origin_y = origin_y_mm + (base.height_mm - probe.height_mm) / 2.0;
    let annotation = layout_vertical_run(font, &ruby.annotation, ann_x, ann_origin_y, ann_size)?;
    let advance_mm = base.height_mm;
    let inline_mm = (ann_x - origin_x_mm) + ann_size;
    Ok(PositionedVerticalRuby {
        base,
        annotation,
        advance_mm,
        inline_mm,
    })
}

/// Lower [`PositionedVerticalRuby`] to per-glyph [`Shape::GlyphRun`].
pub fn positioned_vertical_ruby_to_shapes(
    ruby: &PositionedVerticalRuby,
    fill: Color,
) -> Vec<Shape> {
    let mut shapes = positioned_vertical_to_shapes(&ruby.annotation, fill);
    shapes.extend(positioned_vertical_to_shapes(&ruby.base, fill));
    shapes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distribute_tokyo_reading() {
        let segs = distribute_jukugo_annotation("東京", "とうきょう");
        assert_eq!(segs, vec!["とう", "きょう"]);
    }

    #[test]
    fn distribute_single_base_gets_all() {
        let segs = distribute_jukugo_annotation("漢", "かんじ");
        assert_eq!(segs, vec!["かんじ"]);
    }
}
