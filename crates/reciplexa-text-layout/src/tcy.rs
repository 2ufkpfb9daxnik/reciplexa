//! Font-backed tate-chu-yoko (Step 8 item 3): upright horizontal span in vertical-rl.
//!
//! Stub [`reciplexa_std::japanese::TateChuYoko::estimate_box`] stays the reference.
//! Longer mixed-script compression and CSS `text-combine-upright` remain OPEN.

use reciplexa_scene::{Color, Shape};
use reciplexa_std::japanese::TateChuYoko;

use crate::error::LayoutError;
use crate::font::LoadedFont;
use crate::position::{positioned_line_to_glyph_shapes, PositionedLine};
use crate::shape::shape_run;

/// Max ASCII scalars in this slice (JLReq common 2-digit, allow up to 4).
pub const TCY_MAX_CHARS: usize = 4;

/// Positioned tate-chu-yoko: horizontal glyphs in a 1 em vertical cell.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedTateChuYoko {
    pub line: PositionedLine,
    /// Vertical measure consumed in the column (surrounding em).
    pub block_mm: f64,
}

fn tcy_body_ok(body: &str) -> Result<(), LayoutError> {
    if body.is_empty() {
        return Err(LayoutError::Engine {
            detail: "tate-chu-yoko body must be non-empty".into(),
        });
    }
    let n = body.chars().count();
    if n > TCY_MAX_CHARS {
        return Err(LayoutError::Engine {
            detail: format!("tate-chu-yoko slice allows at most {TCY_MAX_CHARS} characters"),
        });
    }
    if !body.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(LayoutError::Engine {
            detail: "tate-chu-yoko slice is ASCII alnum only".into(),
        });
    }
    Ok(())
}

/// Layout [`TateChuYoko`] as a horizontal run centered on the vertical cell.
///
/// `origin_x_mm` is the left of the surrounding em-box; `origin_y_mm` is the
/// cell baseline. Glyphs stay upright (not tate-rotated). If natural width
/// exceeds one surrounding em, the run is scaled down to fit.
pub fn layout_tate_chu_yoko(
    font: &LoadedFont,
    span: &TateChuYoko,
    origin_x_mm: f64,
    origin_y_mm: f64,
    surrounding_size_mm: f64,
) -> Result<PositionedTateChuYoko, LayoutError> {
    tcy_body_ok(&span.body)?;
    let run = shape_run(font, &span.body)?;
    let natural_mm = run.width_em() * surrounding_size_mm;
    let scale = if natural_mm > surrounding_size_mm && natural_mm > 0.0 {
        surrounding_size_mm / natural_mm
    } else {
        1.0
    };
    let glyph_size = surrounding_size_mm * scale;
    let width_mm = run.width_em() * glyph_size;
    let x0 = origin_x_mm + (surrounding_size_mm - width_mm) / 2.0;
    Ok(PositionedTateChuYoko {
        line: PositionedLine::from_shaped_run(
            font.id.clone(),
            &run,
            x0,
            origin_y_mm,
            glyph_size,
            "ja",
        ),
        block_mm: surrounding_size_mm,
    })
}

pub fn positioned_tcy_to_shapes(tcy: &PositionedTateChuYoko, fill: Color) -> Vec<Shape> {
    positioned_line_to_glyph_shapes(&tcy.line, fill)
}
