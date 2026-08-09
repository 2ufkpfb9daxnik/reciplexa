//! Raster Image backend — scene → RGB8 / PNG with explicit capability losses.
//!
//! Phase 12 order: SVG → **Raster** → PDF → PPTX → Video.
//! Preview vs Final share the same sampler; this crate is Final-oriented export.

#![forbid(unsafe_code)]

use std::io::Cursor;

use reciplexa_scene::{Color, Document};
use reciplexa_view::{flatten_page, WorldShape};

/// Export options (Final Output profile knobs).
#[derive(Debug, Clone, PartialEq)]
pub struct RasterOptions {
    /// Pixels per millimeter (e.g. 300 DPI ≈ 11.811 px/mm).
    pub px_per_mm: f64,
    pub background: Color,
}

impl Default for RasterOptions {
    fn default() -> Self {
        Self {
            px_per_mm: 96.0 / 25.4,
            background: Color::WHITE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RasterError {
    EmptyDocument,
    BadOptions(String),
    PageOutOfRange(usize),
    Encode(String),
}

/// RGB8 framebuffer plus declared losses (never silent).
#[derive(Debug, Clone, PartialEq)]
pub struct RasterFrame {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<u8>,
    pub losses: Vec<RasterLoss>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RasterLoss {
    TextSkipped,
    ImageSkipped,
    PathAsPolyline,
}

/// Rasterize one page of `doc` into an RGB8 frame.
pub fn rasterize_page(
    doc: &Document,
    page_index: usize,
    opts: &RasterOptions,
) -> Result<RasterFrame, RasterError> {
    if doc.pages.is_empty() {
        return Err(RasterError::EmptyDocument);
    }
    if !opts.px_per_mm.is_finite() || opts.px_per_mm <= 0.0 {
        return Err(RasterError::BadOptions(
            "px_per_mm must be finite > 0".into(),
        ));
    }
    let (page, shapes) =
        flatten_page(doc, page_index).ok_or(RasterError::PageOutOfRange(page_index))?;
    let w_mm = page.paper.width_mm;
    let h_mm = page.paper.height_mm;
    let width = (w_mm * opts.px_per_mm).ceil().max(1.0) as u32;
    let height = (h_mm * opts.px_per_mm).ceil().max(1.0) as u32;
    let mut rgb = vec![0u8; (width as usize) * (height as usize) * 3];
    fill_solid(&mut rgb, opts.background);

    let mut losses = Vec::new();
    for shape in &shapes {
        match shape {
            WorldShape::Circle(c) => {
                fill_circle(
                    &mut rgb,
                    width,
                    height,
                    c.x_mm * opts.px_per_mm,
                    c.y_mm * opts.px_per_mm,
                    c.radius_mm * opts.px_per_mm,
                    c.color,
                );
            }
            WorldShape::Polygon(p) => {
                let pts: Vec<(f64, f64)> = p
                    .points_mm
                    .iter()
                    .map(|(x, y)| (x * opts.px_per_mm, y * opts.px_per_mm))
                    .collect();
                fill_polygon(&mut rgb, width, height, &pts, p.color);
            }
            WorldShape::Path(p) => {
                losses.push(RasterLoss::PathAsPolyline);
                let pts: Vec<(f64, f64)> = p
                    .points_mm
                    .iter()
                    .map(|(x, y)| (x * opts.px_per_mm, y * opts.px_per_mm))
                    .collect();
                stroke_polyline(
                    &mut rgb,
                    width,
                    height,
                    &pts,
                    p.stroke,
                    p.width_mm * opts.px_per_mm,
                );
            }
            WorldShape::Text(_) => losses.push(RasterLoss::TextSkipped),
            WorldShape::Image(_) => losses.push(RasterLoss::ImageSkipped),
        }
    }
    // Dedup loss kinds for stable reporting.
    losses.sort_by_key(|l| format!("{l:?}"));
    losses.dedup();

    Ok(RasterFrame {
        width,
        height,
        rgb,
        losses,
    })
}

/// Encode a frame as PNG bytes.
pub fn frame_to_png(frame: &RasterFrame) -> Result<Vec<u8>, RasterError> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut encoder = png::Encoder::new(&mut buf, frame.width, frame.height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| RasterError::Encode(e.to_string()))?;
        writer
            .write_image_data(&frame.rgb)
            .map_err(|e| RasterError::Encode(e.to_string()))?;
    }
    Ok(buf.into_inner())
}

/// Convenience: page 0 → PNG.
pub fn document_page_to_png(
    doc: &Document,
    page_index: usize,
    opts: &RasterOptions,
) -> Result<(Vec<u8>, Vec<RasterLoss>), RasterError> {
    let frame = rasterize_page(doc, page_index, opts)?;
    // `rasterize_page` always yields an RGB8 buffer sized for the encoder.
    let png = frame_to_png(&frame).expect("rasterize_page frame is PNG-encodable");
    Ok((png, frame.losses))
}

fn fill_solid(rgb: &mut [u8], c: Color) {
    let (r, g, b) = color_bytes(c);
    for px in rgb.chunks_exact_mut(3) {
        px[0] = r;
        px[1] = g;
        px[2] = b;
    }
}

fn color_bytes(c: Color) -> (u8, u8, u8) {
    (
        (c.r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.b.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn put_pixel(rgb: &mut [u8], width: u32, height: u32, x: i32, y: i32, c: Color) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }
    let i = ((y as u32 * width + x as u32) * 3) as usize;
    let (r, g, b) = color_bytes(c);
    rgb[i] = r;
    rgb[i + 1] = g;
    rgb[i + 2] = b;
}

fn fill_circle(rgb: &mut [u8], width: u32, height: u32, cx: f64, cy: f64, r: f64, fill: Color) {
    if r <= 0.0 {
        return;
    }
    let r2 = r * r;
    let min_x = (cx - r).floor() as i32;
    let max_x = (cx + r).ceil() as i32;
    let min_y = (cy - r).floor() as i32;
    let max_y = (cy + r).ceil() as i32;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f64 + 0.5 - cx;
            let dy = y as f64 + 0.5 - cy;
            if dx * dx + dy * dy <= r2 {
                put_pixel(rgb, width, height, x, y, fill);
            }
        }
    }
}

fn fill_polygon(rgb: &mut [u8], width: u32, height: u32, pts: &[(f64, f64)], fill: Color) {
    if pts.len() < 3 {
        return;
    }
    let min_x = pts
        .iter()
        .map(|p| p.0)
        .fold(f64::INFINITY, f64::min)
        .floor() as i32;
    let max_x = pts
        .iter()
        .map(|p| p.0)
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil() as i32;
    let min_y = pts
        .iter()
        .map(|p| p.1)
        .fold(f64::INFINITY, f64::min)
        .floor() as i32;
    let max_y = pts
        .iter()
        .map(|p| p.1)
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil() as i32;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if point_in_poly(x as f64 + 0.5, y as f64 + 0.5, pts) {
                put_pixel(rgb, width, height, x, y, fill);
            }
        }
    }
}

fn point_in_poly(x: f64, y: f64, pts: &[(f64, f64)]) -> bool {
    let mut inside = false;
    let n = pts.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = pts[i];
        let (xj, yj) = pts[j];
        let intersect =
            ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi + f64::EPSILON) + xi);
        if intersect {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn stroke_polyline(
    rgb: &mut [u8],
    width: u32,
    height: u32,
    pts: &[(f64, f64)],
    stroke: Color,
    width_px: f64,
) {
    if pts.len() < 2 {
        return;
    }
    let r = (width_px * 0.5).max(0.5);
    for w in pts.windows(2) {
        let (x0, y0) = w[0];
        let (x1, y1) = w[1];
        let steps = (((x1 - x0).hypot(y1 - y0)) / 0.5).ceil().max(1.0) as i32;
        for s in 0..=steps {
            let t = s as f64 / steps as f64;
            let x = x0 + (x1 - x0) * t;
            let y = y0 + (y1 - y0) * t;
            fill_circle(rgb, width, height, x, y, r, stroke);
        }
    }
}
