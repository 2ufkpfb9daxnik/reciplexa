//! Font-backed horizontal simple ruby (Step 8 item 1).
//!
//! Jukugo distribution, vertical ruby, and JLReq overhang remain OPEN.
//! Stub [`reciplexa_std::japanese::Ruby::estimate_box`] stays the reference.

use reciplexa_scene::{Color, Shape};
use reciplexa_std::japanese::{Ruby, RubyKind, RUBY_ANNOTATION_SCALE, RUBY_HEIGHT_BUMP_EM};

use crate::error::LayoutError;
use crate::font::LoadedFont;
use crate::position::{positioned_line_to_glyph_shapes, PositionedLine};
use crate::shape::shape_run;

/// Positioned simple ruby: base line plus annotation line above it (page Y-up).
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedRuby {
    pub base: PositionedLine,
    pub annotation: PositionedLine,
    /// Combined inline advance (mm): `max(base, annotation)`.
    pub advance_mm: f64,
}

/// Layout [`RubyKind::Simple`] with font advances.
///
/// Annotation is drawn at [`RUBY_ANNOTATION_SCALE`] of `base_size_mm`, centered
/// on the combined advance. Its baseline sits one parent em above the parent
/// baseline so the half-size ruby band occupies [`RUBY_HEIGHT_BUMP_EM`] *above*
/// the parent em-square instead of overlapping it.
pub fn layout_simple_ruby(
    font: &LoadedFont,
    ruby: &Ruby,
    origin_x_mm: f64,
    origin_y_mm: f64,
    base_size_mm: f64,
) -> Result<PositionedRuby, LayoutError> {
    if ruby.kind != RubyKind::Simple {
        return Err(LayoutError::Engine {
            detail: format!(
                "simple-ruby slice does not layout kind `{}`",
                ruby.kind.as_str()
            ),
        });
    }
    if ruby.base.is_empty() || ruby.annotation.is_empty() {
        return Err(LayoutError::Engine {
            detail: "ruby base and annotation must be non-empty".into(),
        });
    }
    let base_run = shape_run(font, &ruby.base)?;
    let ann_run = shape_run(font, &ruby.annotation)?;
    let ann_size = base_size_mm * RUBY_ANNOTATION_SCALE;
    let base_w = base_run.width_em() * base_size_mm;
    let ann_w = ann_run.width_em() * ann_size;
    let advance_mm = base_w.max(ann_w);
    let base_x = origin_x_mm + (advance_mm - base_w) / 2.0;
    let ann_x = origin_x_mm + (advance_mm - ann_w) / 2.0;
    // Parent em-square is 1 em; the bump band is the annotation itself.
    let ann_y = origin_y_mm + base_size_mm * (1.0 + RUBY_HEIGHT_BUMP_EM - RUBY_ANNOTATION_SCALE);
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
    })
}

/// Lower [`PositionedRuby`] to per-glyph [`Shape::GlyphRun`].
pub fn positioned_ruby_to_shapes(ruby: &PositionedRuby, fill: Color) -> Vec<Shape> {
    let mut shapes = positioned_line_to_glyph_shapes(&ruby.annotation, fill);
    shapes.extend(positioned_line_to_glyph_shapes(&ruby.base, fill));
    shapes
}
