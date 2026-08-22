//! Positioned text IR and scene adapter (GlyphRun so layout GIDs survive).

use reciplexa_scene::{Color, Document, GlyphRunShape, Shape, Text};

use crate::bidi::visual_positions_em;
use crate::complex::shape_run_complex;
use crate::error::LayoutError;
use crate::font::{FontId, LoadedFont};
use crate::ja::LineSegment;
use crate::math_layout::PositionedMath;
use crate::policy::require_emit_matches_shaped_run;
use crate::protocol::{infer_script, ShapingAttributes};
use crate::shape::ShapedGlyph;

/// Writing mode for positioned output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritingMode {
    HorizontalTb,
    VerticalRl,
}

/// Direction of the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Ltr,
    Rtl,
}

/// Specification-shaped glyph run (IR-001 spirit): font identity, GIDs, clusters.
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRun {
    pub font: FontId,
    pub size_mm: f64,
    pub direction: Direction,
    pub writing_mode: WritingMode,
    pub language: String,
    pub content: String,
    pub glyphs: Vec<PositionedGlyph>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PositionedGlyph {
    pub gid: u16,
    pub ch: char,
    pub cluster_start: usize,
    pub cluster_end: usize,
    pub x_mm: f64,
    pub y_mm: f64,
    pub advance_mm: f64,
    /// Face digest for export (per-glyph when fallback mixes faces).
    pub font_digest: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PositionedLine {
    pub run: GlyphRun,
    pub x_mm: f64,
    pub y_mm: f64,
    pub width_mm: f64,
}

impl PositionedLine {
    pub fn from_segment(
        font: FontId,
        seg: &LineSegment,
        origin_x_mm: f64,
        origin_y_mm: f64,
        size_mm: f64,
        language: &str,
    ) -> Self {
        let mut attrs = ShapingAttributes::horizontal_ltr(language);
        attrs.script = infer_script(&seg.text);
        let positions = visual_positions_em(&seg.text, &seg.glyphs, attrs.direction);
        let mut glyphs = Vec::with_capacity(positions.len());
        let mut width_em = 0.0f64;
        for (i, x_em) in positions {
            let g = &seg.glyphs[i];
            let adv = g.advance_em * size_mm;
            glyphs.push(PositionedGlyph {
                gid: g.gid,
                ch: g.ch,
                cluster_start: g.cluster_start.saturating_sub(seg.start_byte),
                cluster_end: g.cluster_end.saturating_sub(seg.start_byte),
                x_mm: origin_x_mm + x_em * size_mm,
                y_mm: origin_y_mm,
                advance_mm: adv,
                font_digest: g.font_id.digest.clone(),
            });
            width_em = width_em.max(x_em + g.advance_em);
        }
        Self {
            run: GlyphRun {
                font,
                size_mm,
                direction: attrs.direction,
                writing_mode: attrs.writing_mode,
                language: language.to_string(),
                content: seg.text.clone(),
                glyphs,
            },
            x_mm: origin_x_mm,
            y_mm: origin_y_mm,
            width_mm: width_em * size_mm,
        }
    }

    /// Position a whole shaped run at `size_mm` (no hang/trim).
    pub fn from_shaped_run(
        font: FontId,
        run: &crate::shape::ShapedRun,
        origin_x_mm: f64,
        origin_y_mm: f64,
        size_mm: f64,
        language: &str,
    ) -> Self {
        Self::from_shaped_run_with_attrs(
            font,
            run,
            origin_x_mm,
            origin_y_mm,
            size_mm,
            &ShapingAttributes::horizontal_ltr(language),
        )
    }

    /// Position a shaped run using paragraph direction for visual reorder.
    pub fn from_shaped_run_with_attrs(
        font: FontId,
        run: &crate::shape::ShapedRun,
        origin_x_mm: f64,
        origin_y_mm: f64,
        size_mm: f64,
        attrs: &ShapingAttributes,
    ) -> Self {
        let positions = crate::bidi::visual_positions_em(&run.text, &run.glyphs, attrs.direction);
        let mut out_glyphs = Vec::with_capacity(positions.len());
        let mut width_em = 0.0f64;
        for (i, x_em) in positions {
            let g = &run.glyphs[i];
            let adv = g.advance_em * size_mm;
            out_glyphs.push(PositionedGlyph {
                gid: g.gid,
                ch: g.ch,
                cluster_start: g.cluster_start,
                cluster_end: g.cluster_end,
                x_mm: origin_x_mm + x_em * size_mm,
                y_mm: origin_y_mm,
                advance_mm: adv,
                font_digest: g.font_id.digest.clone(),
            });
            width_em = width_em.max(x_em + g.advance_em);
        }
        Self {
            run: GlyphRun {
                font,
                size_mm,
                direction: attrs.direction,
                writing_mode: attrs.writing_mode,
                language: attrs.language.clone(),
                content: run.text.clone(),
                glyphs: out_glyphs,
            },
            x_mm: origin_x_mm,
            y_mm: origin_y_mm,
            width_mm: width_em * size_mm,
        }
    }
}

/// Coarse adapter: one scene [`Text`] per line (string, not GIDs).
pub fn positioned_line_to_scene_text(line: &PositionedLine, fill: Color) -> Text {
    Text {
        x_mm: line.x_mm,
        y_mm: line.y_mm,
        size_mm: line.run.size_mm,
        width_mm: None,
        height_mm: None,
        content: line.run.content.clone(),
        fill,
    }
}

/// Product adapter: one [`GlyphRunShape`] per glyph so trim/justify x and GIDs reach export.
pub fn positioned_line_to_glyph_texts(line: &PositionedLine, fill: Color) -> Vec<Text> {
    if line.run.glyphs.is_empty() {
        return vec![positioned_line_to_scene_text(line, fill)];
    }
    line.run
        .glyphs
        .iter()
        .map(|g| Text {
            x_mm: g.x_mm,
            y_mm: g.y_mm,
            size_mm: line.run.size_mm,
            width_mm: None,
            height_mm: None,
            content: g.ch.to_string(),
            fill,
        })
        .collect()
}

pub(crate) fn positioned_glyph_to_shape(g: &PositionedGlyph, size_mm: f64, fill: Color) -> Shape {
    Shape::GlyphRun(GlyphRunShape::one(
        g.x_mm,
        g.y_mm,
        size_mm,
        g.ch.to_string(),
        fill,
        g.gid,
        &g.font_digest,
        g.advance_mm,
    ))
}

pub fn positioned_line_to_glyph_shapes(line: &PositionedLine, fill: Color) -> Vec<Shape> {
    if line.run.glyphs.is_empty() {
        return vec![Shape::Text(positioned_line_to_scene_text(line, fill))];
    }
    line.run
        .glyphs
        .iter()
        .map(|g| positioned_glyph_to_shape(g, line.run.size_mm, fill))
        .collect()
}

/// Export coordinates from a positioned line (preview/export parity checks).
pub fn positioned_line_glyph_coords(line: &PositionedLine) -> Vec<(f64, f64, u16, f64, String)> {
    line.run
        .glyphs
        .iter()
        .map(|g| (g.x_mm, g.y_mm, g.gid, g.advance_mm, g.font_digest.clone()))
        .collect()
}

/// Export coordinates from product glyph shapes (one cluster per shape).
pub fn glyph_shapes_export_coords(shapes: &[Shape]) -> Vec<(f64, f64, u16, f64, String)> {
    shapes
        .iter()
        .filter_map(|s| match s {
            Shape::GlyphRun(g) => Some((
                g.x_mm,
                g.y_mm,
                g.gids.first().copied().unwrap_or(0),
                g.advances_mm.first().copied().unwrap_or(0.0),
                g.font_digest.clone(),
            )),
            _ => None,
        })
        .collect()
}

pub fn positioned_lines_to_shapes(lines: &[PositionedLine], fill: Color) -> Vec<Shape> {
    lines
        .iter()
        .flat_map(|l| positioned_line_to_glyph_shapes(l, fill))
        .collect()
}

/// Lower a math layout to scene glyph paints + rules.
///
/// One [`GlyphRunShape`] per glyph so intra-row x and layout GIDs survive.
/// Page space is y-up (scene/PDF); engine `y_em` is also up from the
/// math baseline, so `y_mm = origin.1 + y_em * em_to_mm`.
pub fn positioned_math_to_shapes(
    math: &PositionedMath,
    origin: (f64, f64),
    em_to_mm: f64,
    fill: Color,
    font: &FontId,
) -> Vec<Shape> {
    use reciplexa_scene::Line;
    let mut shapes = Vec::new();
    for g in &math.glyphs {
        let size_mm = g.scale * em_to_mm;
        shapes.push(Shape::GlyphRun(GlyphRunShape::one(
            origin.0 + g.x_em * em_to_mm,
            origin.1 + g.y_em * em_to_mm,
            size_mm,
            g.glyph.ch.to_string(),
            fill,
            g.glyph.gid,
            font.digest.clone(),
            g.glyph.advance_em * size_mm,
        )));
    }
    for r in &math.rules {
        shapes.push(Shape::Line(Line {
            x1_mm: origin.0 + r.x0_em * em_to_mm,
            y1_mm: origin.1 + r.y0_em * em_to_mm,
            x2_mm: origin.0 + r.x1_em * em_to_mm,
            y2_mm: origin.1 + r.y1_em * em_to_mm,
            stroke: fill,
            width_mm: (r.thickness_em * em_to_mm).max(0.05),
        }));
    }
    shapes
}

/// Build a GlyphRun from already-shaped glyphs at a baseline.
pub fn glyph_run_from_shaped(
    font: FontId,
    glyphs: &[ShapedGlyph],
    origin_x_mm: f64,
    origin_y_mm: f64,
    size_mm: f64,
    language: &str,
) -> Result<GlyphRun, LayoutError> {
    if glyphs.is_empty() {
        return Err(LayoutError::Engine {
            detail: "empty glyph run".into(),
        });
    }
    let content: String = glyphs.iter().map(|g| g.ch).collect();
    let start = glyphs[0].cluster_start;
    let mut x = 0.0;
    let mut out = Vec::new();
    for g in glyphs {
        let adv = g.advance_em * size_mm;
        out.push(PositionedGlyph {
            gid: g.gid,
            ch: g.ch,
            cluster_start: g.cluster_start.saturating_sub(start),
            cluster_end: g.cluster_end.saturating_sub(start),
            x_mm: origin_x_mm + x,
            y_mm: origin_y_mm,
            advance_mm: adv,
            font_digest: g.font_id.digest.clone(),
        });
        x += adv;
    }
    Ok(GlyphRun {
        font,
        size_mm,
        direction: Direction::Ltr,
        writing_mode: WritingMode::HorizontalTb,
        language: language.to_string(),
        content,
        glyphs: out,
    })
}

/// Rewrite [`Shape::Text`] leaves to cluster [`Shape::GlyphRun`] using `font`.
///
/// One authoring text node stays one scene node so GUI package layers still
/// align. Groups/opacity are walked. Missing glyphs fail.
pub fn productize_shape_text(shape: Shape, font: &LoadedFont) -> Result<Shape, LayoutError> {
    productize_shape_text_emit(shape, font, font)
}

/// Rewrite text leaves using `layout_font` for shaping and `emit_font` for paint identity.
pub fn productize_shape_text_emit(
    shape: Shape,
    layout_font: &LoadedFont,
    emit_font: &LoadedFont,
) -> Result<Shape, LayoutError> {
    match shape {
        Shape::Text(t) => scene_text_to_glyph_run(layout_font, emit_font, &t),
        Shape::Group {
            transform,
            children,
        } => Ok(Shape::Group {
            transform,
            children: children
                .into_iter()
                .map(|c| productize_shape_text_emit(c, layout_font, emit_font))
                .collect::<Result<Vec<_>, _>>()?,
        }),
        Shape::Opacity { alpha, children } => Ok(Shape::Opacity {
            alpha,
            children: children
                .into_iter()
                .map(|c| productize_shape_text_emit(c, layout_font, emit_font))
                .collect::<Result<Vec<_>, _>>()?,
        }),
        other => Ok(other),
    }
}

/// Rewrite every page's text leaves (see [`productize_shape_text`]).
pub fn productize_document_text(
    mut doc: Document,
    font: &LoadedFont,
) -> Result<Document, LayoutError> {
    productize_document_text_emit(&mut doc, font, font)?;
    Ok(doc)
}

/// Document rewrite with distinct layout and emit faces (export registry parity).
pub fn productize_document_text_emit(
    doc: &mut Document,
    layout_font: &LoadedFont,
    emit_font: &LoadedFont,
) -> Result<(), LayoutError> {
    for page in &mut doc.pages {
        let shapes = std::mem::take(&mut page.shapes);
        page.shapes = shapes
            .into_iter()
            .map(|s| productize_shape_text_emit(s, layout_font, emit_font))
            .collect::<Result<Vec<_>, _>>()?;
    }
    Ok(())
}

fn shaping_attrs_for_text(text: &str) -> ShapingAttributes {
    let mut attrs = ShapingAttributes::horizontal_ltr("und");
    attrs.script = infer_script(text);
    attrs
}

fn scene_text_to_glyph_run(
    layout_font: &LoadedFont,
    emit_font: &LoadedFont,
    t: &Text,
) -> Result<Shape, LayoutError> {
    if !t.is_drawable() {
        return Ok(Shape::Text(t.clone()));
    }
    if t.content.chars().all(|c| c == '\n' || c == '\r') {
        return Ok(Shape::Text(t.clone()));
    }
    let attrs = shaping_attrs_for_text(&t.content);
    let run = shape_run_complex(layout_font, &t.content, &attrs)?;
    if run.glyphs.is_empty() {
        return Ok(Shape::Text(t.clone()));
    }
    require_emit_matches_shaped_run(&run, emit_font)?;
    let order = crate::bidi::visual_glyph_indices(&run.text, &run.glyphs, attrs.direction);
    Ok(Shape::GlyphRun(GlyphRunShape {
        x_mm: t.x_mm,
        y_mm: t.y_mm,
        size_mm: t.size_mm,
        content: t.content.clone(),
        fill: t.fill,
        gids: order.iter().map(|&i| run.glyphs[i].gid).collect(),
        font_digest: emit_font.id.digest.clone(),
        advances_mm: order
            .iter()
            .map(|&i| run.glyphs[i].advance_em * t.size_mm)
            .collect(),
    }))
}
