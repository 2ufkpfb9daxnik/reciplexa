//! Font-backed bou / 傍点 (Step 8 item 3): emphasis dots beside or above glyphs.
//!
//! Stub [`reciplexa_std::japanese::bou_estimate_box`] stays the reference.
//! CSS `text-emphasis` and sesame-glyph substitution remain OPEN.

use reciplexa_scene::{Circle, Color, Shape};
use reciplexa_std::japanese::BOU_MARK_SIZE_EM;

use crate::error::LayoutError;
use crate::font::LoadedFont;
use crate::position::{positioned_line_to_glyph_shapes, PositionedLine};
use crate::shape::shape_run;
use crate::vert::{layout_vertical_run, positioned_vertical_to_shapes, PositionedVertical};

/// Extra em between the parent em-square and the mark (product only).
/// Stub `BOU_SIDE_OFFSET_EM` stays the fontless box.
pub const BOU_PARENT_GAP_EM: f64 = 0.25;

/// Horizontal bou: body GlyphRuns plus filled-circle marks above the em-square.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedBouHorizontal {
    pub body: PositionedLine,
}

/// Vertical bou: stacked body plus marks to the side of the column.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedBouVertical {
    pub body: PositionedVertical,
}

fn bou_body_ok(body: &str) -> Result<(), LayoutError> {
    if body.chars().all(|c| c == '\n' || c == '\r' || c == ' ') {
        return Err(LayoutError::Engine {
            detail: "bou body must be non-empty".into(),
        });
    }
    Ok(())
}

pub fn layout_bou_horizontal(
    font: &LoadedFont,
    body: &str,
    origin_x_mm: f64,
    origin_y_mm: f64,
    size_mm: f64,
) -> Result<PositionedBouHorizontal, LayoutError> {
    bou_body_ok(body)?;
    let run = shape_run(font, body)?;
    Ok(PositionedBouHorizontal {
        body: PositionedLine::from_shaped_run(
            font.id.clone(),
            &run,
            origin_x_mm,
            origin_y_mm,
            size_mm,
            "ja",
        ),
    })
}

pub fn layout_bou_vertical(
    font: &LoadedFont,
    body: &str,
    origin_x_mm: f64,
    origin_y_mm: f64,
    size_mm: f64,
) -> Result<PositionedBouVertical, LayoutError> {
    bou_body_ok(body)?;
    Ok(PositionedBouVertical {
        body: layout_vertical_run(font, body, origin_x_mm, origin_y_mm, size_mm)?,
    })
}

fn mark_circle(x_mm: f64, y_mm: f64, radius_mm: f64, fill: Color) -> Shape {
    Shape::Circle(Circle {
        x_mm,
        y_mm,
        radius_mm,
        fill,
    })
}

fn mark_radius_mm(size_mm: f64) -> f64 {
    size_mm * BOU_MARK_SIZE_EM * 0.5
}

/// Distance from the em-square edge to the mark center.
fn mark_clearance_mm(size_mm: f64) -> f64 {
    size_mm * BOU_PARENT_GAP_EM + mark_radius_mm(size_mm)
}

pub fn positioned_bou_horizontal_to_shapes(
    bou: &PositionedBouHorizontal,
    fill: Color,
) -> Vec<Shape> {
    let size = bou.body.run.size_mm;
    let radius = mark_radius_mm(size);
    let clear = mark_clearance_mm(size);
    let mut shapes = positioned_line_to_glyph_shapes(&bou.body, fill);
    for g in &bou.body.run.glyphs {
        let cx = g.x_mm + g.advance_mm * 0.5;
        let cy = g.y_mm + size + clear;
        shapes.push(mark_circle(cx, cy, radius, fill));
    }
    shapes
}

pub fn positioned_bou_vertical_to_shapes(bou: &PositionedBouVertical, fill: Color) -> Vec<Shape> {
    let size = bou.body.run.size_mm;
    let radius = mark_radius_mm(size);
    let clear = mark_clearance_mm(size);
    let mut shapes = positioned_vertical_to_shapes(&bou.body, fill);
    for g in &bou.body.run.glyphs {
        let cx = g.x_mm + size + clear;
        let cy = g.y_mm + size * 0.5;
        shapes.push(mark_circle(cx, cy, radius, fill));
    }
    shapes
}
