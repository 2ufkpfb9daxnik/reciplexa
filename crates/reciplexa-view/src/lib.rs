//! Project [`reciplexa_scene`] documents into drawable primitives.
//!
//! Kept free of egui/eframe so preview math and (later) hit-testing can be
//! unit-tested headlessly. The GUI crate only maps these primitives to pixels.

#![forbid(unsafe_code)]

use reciplexa_scene::{Affine, Color, Document, Page, Shape};

/// Circle in page millimeters after applying ancestor transforms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldCircle {
    pub x_mm: f64,
    pub y_mm: f64,
    pub radius_mm: f64,
    pub color: Color,
    /// `None` = filled; `Some(w)` = stroked outline of width `w` mm.
    pub stroke_width_mm: Option<f64>,
    /// Accumulated opacity from ancestor `(opacity …)` nodes (`0..=1`).
    pub alpha: f64,
}

/// Convex polygon in page millimeters (rect corners after affine).
#[derive(Debug, Clone, PartialEq)]
pub struct WorldPolygon {
    pub points_mm: Vec<(f64, f64)>,
    pub color: Color,
    /// `None` = filled; `Some(w)` = stroked outline of width `w` mm.
    pub stroke_width_mm: Option<f64>,
    pub alpha: f64,
}

/// Text baseline in page millimeters.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldText {
    pub x_mm: f64,
    pub y_mm: f64,
    pub size_mm: f64,
    /// Layout box width/height in page mm (after parent scale).
    pub width_mm: f64,
    pub height_mm: f64,
    /// Counter-clockwise degrees from parent affine (page space).
    pub rotation_deg: f64,
    pub content: String,
    pub fill: Color,
    pub alpha: f64,
    /// Layout GID when this world text came from [`reciplexa_scene::Shape::GlyphRun`].
    pub glyph_id: Option<u16>,
    pub font_digest: Option<String>,
    pub glyph_advance_mm: Option<f64>,
}

/// Open or closed stroked path in page millimeters.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldPath {
    pub points_mm: Vec<(f64, f64)>,
    pub stroke: Color,
    pub width_mm: f64,
    pub closed: bool,
    pub alpha: f64,
}

/// Image quad in page millimeters (corners after affine).
#[derive(Debug, Clone, PartialEq)]
pub struct WorldImage {
    pub path: String,
    /// Bottom-left, bottom-right, top-right, top-left in page mm.
    pub corners_mm: [(f64, f64); 4],
    pub alpha: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorldShape {
    Circle(WorldCircle),
    Polygon(WorldPolygon),
    Text(WorldText),
    Path(WorldPath),
    Image(WorldImage),
}

/// Options controlling flatten (Phase 7 output profile inputs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlattenOptions {
    /// Polygon side count used when approximating ellipses.
    pub ellipse_sides: u32,
}

impl Default for FlattenOptions {
    fn default() -> Self {
        Self { ellipse_sides: 32 }
    }
}

/// Flatten a document's first page (preview shows page 0).
pub fn flatten_first_page(doc: &Document) -> Option<(&Page, Vec<WorldShape>)> {
    flatten_page(doc, 0)
}

/// Flatten page `index` (0-based).
pub fn flatten_page(doc: &Document, index: usize) -> Option<(&Page, Vec<WorldShape>)> {
    flatten_page_with_options(doc, index, FlattenOptions::default())
}

/// Flatten page `index` with profile-driven options.
pub fn flatten_page_with_options(
    doc: &Document,
    index: usize,
    options: FlattenOptions,
) -> Option<(&Page, Vec<WorldShape>)> {
    let page = doc.pages.get(index)?;
    Some((
        page,
        flatten_shapes_with_options(&page.shapes, Affine::identity(), options),
    ))
}

pub fn flatten_shapes(shapes: &[Shape], parent: Affine) -> Vec<WorldShape> {
    flatten_shapes_with_options(shapes, parent, FlattenOptions::default())
}

pub fn flatten_shapes_with_options(
    shapes: &[Shape],
    parent: Affine,
    options: FlattenOptions,
) -> Vec<WorldShape> {
    let mut out = Vec::new();
    for shape in shapes {
        flatten_shape(shape, parent, 1.0, options, &mut out);
    }
    out
}

fn flatten_shape(
    shape: &Shape,
    parent: Affine,
    alpha: f64,
    options: FlattenOptions,
    out: &mut Vec<WorldShape>,
) {
    match shape {
        Shape::Circle(c) => {
            let (x, y) = parent.transform_point(c.x_mm, c.y_mm);
            let scale = linear_scale(parent);
            out.push(WorldShape::Circle(WorldCircle {
                x_mm: x,
                y_mm: y,
                radius_mm: c.radius_mm * scale,
                color: c.fill,
                stroke_width_mm: None,
                alpha,
            }));
        }
        Shape::Rect(r) => {
            let corners = [
                (r.x_mm, r.y_mm),
                (r.x_mm + r.width_mm, r.y_mm),
                (r.x_mm + r.width_mm, r.y_mm + r.height_mm),
                (r.x_mm, r.y_mm + r.height_mm),
            ];
            let points_mm = corners
                .into_iter()
                .map(|(x, y)| parent.transform_point(x, y))
                .collect();
            out.push(WorldShape::Polygon(WorldPolygon {
                points_mm,
                color: r.fill,
                stroke_width_mm: None,
                alpha,
            }));
        }
        Shape::Ellipse(e) => {
            let n = options.ellipse_sides.max(3) as usize;
            let mut points_mm = Vec::with_capacity(n);
            for i in 0..n {
                let t = std::f64::consts::TAU * (i as f64) / (n as f64);
                let lx = e.x_mm + e.rx_mm * t.cos();
                let ly = e.y_mm + e.ry_mm * t.sin();
                points_mm.push(parent.transform_point(lx, ly));
            }
            out.push(WorldShape::Polygon(WorldPolygon {
                points_mm,
                color: e.fill,
                stroke_width_mm: None,
                alpha,
            }));
        }
        Shape::Ring(r) => {
            let (x, y) = parent.transform_point(r.x_mm, r.y_mm);
            let scale = linear_scale(parent);
            out.push(WorldShape::Circle(WorldCircle {
                x_mm: x,
                y_mm: y,
                radius_mm: r.radius_mm * scale,
                color: r.stroke,
                stroke_width_mm: Some(r.width_mm * scale),
                alpha,
            }));
        }
        Shape::Frame(f) => {
            let corners = [
                (f.x_mm, f.y_mm),
                (f.x_mm + f.width_mm, f.y_mm),
                (f.x_mm + f.width_mm, f.y_mm + f.height_mm),
                (f.x_mm, f.y_mm + f.height_mm),
            ];
            let points_mm = corners
                .into_iter()
                .map(|(x, y)| parent.transform_point(x, y))
                .collect();
            let scale = linear_scale(parent);
            out.push(WorldShape::Polygon(WorldPolygon {
                points_mm,
                color: f.stroke,
                stroke_width_mm: Some(f.stroke_width_mm * scale),
                alpha,
            }));
        }
        Shape::Text(t) => {
            let (x, y) = parent.transform_point(t.x_mm, t.y_mm);
            let scale = linear_scale(parent);
            let (w, h) = match (t.width_mm, t.height_mm) {
                (Some(w), Some(h)) => (w * scale, h * scale),
                _ => {
                    let (ew, eh) = text_extent_mm(&t.content, t.size_mm * scale);
                    (ew, eh)
                }
            };
            out.push(WorldShape::Text(WorldText {
                x_mm: x,
                y_mm: y,
                size_mm: t.size_mm * scale,
                width_mm: w,
                height_mm: h,
                rotation_deg: parent.rotation_deg(),
                content: t.content.clone(),
                fill: t.fill,
                alpha,
                glyph_id: None,
                font_digest: None,
                glyph_advance_mm: None,
            }));
        }
        Shape::GlyphRun(g) => {
            let (x, y) = parent.transform_point(g.x_mm, g.y_mm);
            let scale = linear_scale(parent);
            let size = g.size_mm * scale;
            let (ew, eh) = text_extent_mm(&g.content, size);
            out.push(WorldShape::Text(WorldText {
                x_mm: x,
                y_mm: y,
                size_mm: size,
                width_mm: ew,
                height_mm: eh,
                rotation_deg: parent.rotation_deg(),
                content: g.content.clone(),
                fill: g.fill,
                alpha,
                glyph_id: Some(g.gid),
                font_digest: Some(g.font_digest.clone()),
                glyph_advance_mm: Some(g.advance_mm * scale),
            }));
        }
        Shape::Line(l) => {
            let (x1, y1) = parent.transform_point(l.x1_mm, l.y1_mm);
            let (x2, y2) = parent.transform_point(l.x2_mm, l.y2_mm);
            let scale = linear_scale(parent);
            out.push(WorldShape::Path(WorldPath {
                points_mm: vec![(x1, y1), (x2, y2)],
                stroke: l.stroke,
                width_mm: l.width_mm * scale,
                closed: false,
                alpha,
            }));
        }
        Shape::Polyline(p) => {
            let scale = linear_scale(parent);
            let points_mm = p
                .points_mm
                .iter()
                .map(|&(x, y)| parent.transform_point(x, y))
                .collect();
            out.push(WorldShape::Path(WorldPath {
                points_mm,
                stroke: p.stroke,
                width_mm: p.width_mm * scale,
                closed: false,
                alpha,
            }));
        }
        Shape::Polygon(p) => {
            let points_mm = p
                .points_mm
                .iter()
                .map(|&(x, y)| parent.transform_point(x, y))
                .collect();
            out.push(WorldShape::Polygon(WorldPolygon {
                points_mm,
                color: p.fill,
                stroke_width_mm: None,
                alpha,
            }));
        }
        Shape::Image(img) => {
            let corners = [
                (img.x_mm, img.y_mm),
                (img.x_mm + img.width_mm, img.y_mm),
                (img.x_mm + img.width_mm, img.y_mm + img.height_mm),
                (img.x_mm, img.y_mm + img.height_mm),
            ];
            let corners_mm = [
                parent.transform_point(corners[0].0, corners[0].1),
                parent.transform_point(corners[1].0, corners[1].1),
                parent.transform_point(corners[2].0, corners[2].1),
                parent.transform_point(corners[3].0, corners[3].1),
            ];
            out.push(WorldShape::Image(WorldImage {
                path: img.path.clone(),
                corners_mm,
                alpha,
            }));
        }
        Shape::Opacity {
            alpha: child_alpha,
            children,
        } => {
            let combined = alpha * (*child_alpha);
            for child in children {
                flatten_shape(child, parent, combined, options, out);
            }
        }
        Shape::Group {
            transform,
            children,
        } => {
            let combined = transform.then(parent);
            for child in children {
                flatten_shape(child, combined, alpha, options, out);
            }
        }
    }
}

fn linear_scale(a: Affine) -> f64 {
    let det = a.a * a.d - a.b * a.c;
    det.abs().sqrt()
}

/// Map page mm (origin bottom-left) to top-left pixel space inside a paper rect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaperLayout {
    pub origin_x_px: f32,
    pub origin_y_px: f32,
    pub width_px: f32,
    pub height_px: f32,
    pub page_width_mm: f64,
    pub page_height_mm: f64,
}

impl PaperLayout {
    pub fn fit(avail_w: f32, avail_h: f32, margin: f32, page_w_mm: f64, page_h_mm: f64) -> Self {
        let inner_w = (avail_w - 2.0 * margin).max(1.0);
        let inner_h = (avail_h - 2.0 * margin).max(1.0);
        let page_aspect = (page_w_mm / page_h_mm) as f32;
        let box_aspect = inner_w / inner_h;
        let (width_px, height_px) = if page_aspect > box_aspect {
            (inner_w, inner_w / page_aspect)
        } else {
            (inner_h * page_aspect, inner_h)
        };
        Self {
            origin_x_px: margin + (inner_w - width_px) * 0.5,
            origin_y_px: margin + (inner_h - height_px) * 0.5,
            width_px,
            height_px,
            page_width_mm: page_w_mm,
            page_height_mm: page_h_mm,
        }
    }

    /// Apply interactive zoom (about paper center) and pan in preview pixels.
    pub fn with_view(mut self, zoom: f32, pan_x: f32, pan_y: f32) -> Self {
        let zoom = zoom.clamp(0.2, 8.0);
        let cx = self.origin_x_px + self.width_px * 0.5;
        let cy = self.origin_y_px + self.height_px * 0.5;
        self.width_px *= zoom;
        self.height_px *= zoom;
        self.origin_x_px = cx - self.width_px * 0.5 + pan_x;
        self.origin_y_px = cy - self.height_px * 0.5 + pan_y;
        self
    }

    /// Pan offset so that `anchor_mm` stays at `anchor_local_px` after [`Self::with_view`].
    pub fn pan_for_zoom_anchor(
        base: &Self,
        zoom: f32,
        anchor_local_px: (f32, f32),
        anchor_mm: (f64, f64),
    ) -> (f32, f32) {
        let zoom = zoom.clamp(0.2, 8.0);
        let w = base.width_px * zoom;
        let h = base.height_px * zoom;
        let sx = w / base.page_width_mm as f32;
        let sy = h / base.page_height_mm as f32;
        let target_ox = anchor_local_px.0 - anchor_mm.0 as f32 * sx;
        let target_oy = anchor_local_px.1 - (base.page_height_mm - anchor_mm.1) as f32 * sy;
        let pan_x = target_ox - base.origin_x_px - base.width_px * 0.5 + w * 0.5;
        let pan_y = target_oy - base.origin_y_px - base.height_px * 0.5 + h * 0.5;
        (pan_x, pan_y)
    }

    /// Pan for a zoom change while keeping the paper point under `anchor_local_px` fixed.
    pub fn pan_for_zoom_change(
        base: &Self,
        old_zoom: f32,
        old_pan: (f32, f32),
        new_zoom: f32,
        anchor_local_px: (f32, f32),
    ) -> (f32, f32) {
        let old_layout = base.with_view(old_zoom, old_pan.0, old_pan.1);
        let anchor_mm = old_layout.px_to_mm(anchor_local_px.0, anchor_local_px.1);
        Self::pan_for_zoom_anchor(base, new_zoom, anchor_local_px, anchor_mm)
    }

    /// Zoom and pan so `bounds_mm` fits inside `avail_px` with `padding_px`, centered.
    pub fn viewport_fit_bounds(
        base: &Self,
        avail_px: (f32, f32),
        bounds_mm: (f64, f64, f64, f64),
        padding_px: f32,
    ) -> (f32, f32, f32) {
        let (x0, y0, x1, y1) = bounds_mm;
        let w_mm = (x1 - x0).max(1.0);
        let h_mm = (y1 - y0).max(1.0);
        let inner_w = (avail_px.0 - 2.0 * padding_px).max(1.0);
        let inner_h = (avail_px.1 - 2.0 * padding_px).max(1.0);
        let mm_per_px_x = base.page_width_mm / base.width_px as f64;
        let mm_per_px_y = base.page_height_mm / base.height_px as f64;
        let zoom_w = inner_w as f64 * mm_per_px_x / w_mm;
        let zoom_h = inner_h as f64 * mm_per_px_y / h_mm;
        let zoom = (zoom_w.min(zoom_h) as f32).clamp(0.2, 8.0);
        let cx_mm = (x0 + x1) * 0.5;
        let cy_mm = (y0 + y1) * 0.5;
        let anchor = (avail_px.0 * 0.5, avail_px.1 * 0.5);
        let (px, py) = Self::pan_for_zoom_anchor(base, zoom, anchor, (cx_mm, cy_mm));
        (zoom, px, py)
    }

    /// Axis-aligned bounds of a flattened shape in page mm `(min_x, min_y, max_x, max_y)`.
    pub fn shape_bounds_mm(shape: &WorldShape) -> Option<(f64, f64, f64, f64)> {
        match shape {
            WorldShape::Circle(c) => {
                let pad = c.stroke_width_mm.unwrap_or(0.0) * 0.5;
                let r = c.radius_mm + pad;
                Some((c.x_mm - r, c.y_mm - r, c.x_mm + r, c.y_mm + r))
            }
            WorldShape::Polygon(p) => bounds_of_points(&p.points_mm),
            WorldShape::Path(p) => bounds_of_points(&p.points_mm),
            WorldShape::Text(t) => bounds_of_points(&text_corners_mm(t)),
            WorldShape::Image(img) => bounds_of_points(&img.corners_mm),
        }
    }

    pub fn mm_to_px(&self, x_mm: f64, y_mm: f64) -> (f32, f32) {
        let sx = self.width_px as f64 / self.page_width_mm;
        let sy = self.height_px as f64 / self.page_height_mm;
        let x = self.origin_x_px as f64 + x_mm * sx;
        let y = self.origin_y_px as f64 + (self.page_height_mm - y_mm) * sy;
        (x as f32, y as f32)
    }

    pub fn radius_mm_to_px(&self, r_mm: f64) -> f32 {
        let sx = self.width_px as f64 / self.page_width_mm;
        (r_mm * sx) as f32
    }

    pub fn px_to_mm(&self, x_px: f32, y_px: f32) -> (f64, f64) {
        let sx = self.page_width_mm / self.width_px as f64;
        let sy = self.page_height_mm / self.height_px as f64;
        let x_mm = (x_px - self.origin_x_px) as f64 * sx;
        let y_from_top = (y_px - self.origin_y_px) as f64 * sy;
        let y_mm = self.page_height_mm - y_from_top;
        (x_mm, y_mm)
    }
}

/// Hit-test shapes in page mm; returns the topmost (last drawn) match.
pub fn hit_test_shapes(shapes: &[WorldShape], x_mm: f64, y_mm: f64) -> Option<usize> {
    shapes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(i, s)| shape_contains(s, x_mm, y_mm).then_some(i))
}

/// Flatten indices whose AABB intersects `aabb_mm` `(min_x, min_y, max_x, max_y)`.
pub fn shapes_intersecting_aabb(
    shapes: &[WorldShape],
    aabb_mm: (f64, f64, f64, f64),
) -> Vec<usize> {
    let (ax0, ay0, ax1, ay1) = normalize_aabb(aabb_mm);
    shapes
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let (bx0, by0, bx1, by1) = PaperLayout::shape_bounds_mm(s)?;
            aabb_intersects((ax0, ay0, ax1, ay1), (bx0, by0, bx1, by1)).then_some(i)
        })
        .collect()
}

fn normalize_aabb(a: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let (x0, y0, x1, y1) = a;
    (x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1))
}

fn aabb_intersects(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    let (ax0, ay0, ax1, ay1) = a;
    let (bx0, by0, bx1, by1) = b;
    ax0 <= bx1 && ax1 >= bx0 && ay0 <= by1 && ay1 >= by0
}

fn shape_contains(shape: &WorldShape, x: f64, y: f64) -> bool {
    match shape {
        WorldShape::Circle(c) => {
            let dx = x - c.x_mm;
            let dy = y - c.y_mm;
            let dist2 = dx * dx + dy * dy;
            match c.stroke_width_mm {
                None => dist2 <= c.radius_mm * c.radius_mm,
                Some(w) => {
                    let r = dist2.sqrt();
                    (r - c.radius_mm).abs() <= w.max(1.0)
                }
            }
        }
        WorldShape::Polygon(p) => {
            if p.stroke_width_mm.is_some() {
                near_polyline(
                    x,
                    y,
                    &p.points_mm,
                    true,
                    p.stroke_width_mm.unwrap_or(1.0).max(1.0),
                )
            } else {
                point_in_polygon(x, y, &p.points_mm)
            }
        }
        WorldShape::Text(t) => {
            let pad = (t.size_mm * 0.25).max(1.5);
            let locals = [
                (-pad, -pad),
                (t.width_mm + pad, -pad),
                (t.width_mm + pad, t.height_mm + pad),
                (-pad, t.height_mm + pad),
            ];
            let rad = t.rotation_deg.to_radians();
            let (sine, cosine) = (rad.sin(), rad.cos());
            let corners: Vec<(f64, f64)> = locals
                .into_iter()
                .map(|(lx, ly)| {
                    let rx = lx * cosine - ly * sine;
                    let ry = lx * sine + ly * cosine;
                    (t.x_mm + rx, t.y_mm + ry)
                })
                .collect();
            point_in_polygon(x, y, &corners)
        }
        WorldShape::Path(p) => {
            near_polyline(x, y, &p.points_mm, p.closed, p.width_mm.max(1.0) + 2.0)
        }
        WorldShape::Image(img) => point_in_polygon(x, y, &img.corners_mm),
    }
}

/// Em-width estimate for one codepoint at `size_mm` (egui proportional / CJK fallback).
fn char_width_mm(ch: char, size_mm: f64) -> f64 {
    if ch == '\t' {
        size_mm * 2.0
    } else if ch.is_ascii() {
        size_mm
            * match ch {
                'i' | 'l' | '!' | '|' | '.' | ',' | ':' | ';' | ' ' => 0.35,
                'm' | 'w' | 'M' | 'W' => 0.85,
                _ => 0.55,
            }
    } else {
        size_mm
    }
}

fn line_width_mm(line: &str, size_mm: f64) -> f64 {
    line.chars().map(|ch| char_width_mm(ch, size_mm)).sum()
}

/// Soft-wrap `content` to `max_width_mm` (hard `\n` preserved). Empty → one empty line.
pub fn wrap_text_to_width(content: &str, size_mm: f64, max_width_mm: f64) -> Vec<String> {
    let max_w = max_width_mm.max(size_mm * 0.01);
    let mut out = Vec::new();
    let paragraphs: Vec<&str> = if content.is_empty() {
        vec![""]
    } else {
        content
            .split('\n')
            .map(|l| l.trim_end_matches('\r'))
            .collect()
    };
    for para in paragraphs {
        if para.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut line = String::new();
        let mut line_w = 0.0_f64;
        for ch in para.chars() {
            let cw = char_width_mm(ch, size_mm);
            if !line.is_empty() && line_w + cw > max_w {
                // Prefer break after whitespace when possible.
                if let Some(idx) = line.rfind(|c: char| c.is_whitespace()) {
                    let (keep, rest) = line.split_at(idx + 1);
                    let rest = rest.to_string();
                    out.push(keep.trim_end().to_string());
                    line = rest;
                    line_w = line_width_mm(&line, size_mm);
                } else {
                    out.push(std::mem::take(&mut line));
                    line_w = 0.0;
                }
            }
            line.push(ch);
            line_w += cw;
        }
        out.push(line);
    }
    out
}

/// Laid-out line count and height for wrapped text (1 em leading per line).
pub fn wrapped_text_height_mm(content: &str, size_mm: f64, max_width_mm: f64) -> f64 {
    let lines = wrap_text_to_width(content, size_mm, max_width_mm);
    (lines.len() as f64 * size_mm).max(size_mm * 0.01)
}

/// Text box size in mm from baseline-left (y up). Matches GUI/PDF hard line breaks (1 em leading).
pub fn text_extent_mm(content: &str, size_mm: f64) -> (f64, f64) {
    let lines: Vec<&str> = if content.is_empty() {
        vec![""]
    } else {
        content
            .split('\n')
            .map(|l| l.trim_end_matches('\r'))
            .collect()
    };
    let w = lines
        .iter()
        .map(|line| line_width_mm(line, size_mm))
        .fold(0.0_f64, f64::max);
    let h = lines.len() as f64 * size_mm;
    let min = size_mm * 0.01;
    (w.max(min), h.max(min))
}

/// Glyph box corners in page mm (baseline-left origin, y up).
pub fn text_corners_mm(t: &WorldText) -> [(f64, f64); 4] {
    let w = t.width_mm;
    let h = t.height_mm;
    let locals = [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)];
    let rad = t.rotation_deg.to_radians();
    let (sine, cosine) = (rad.sin(), rad.cos());
    locals.map(|(lx, ly)| {
        let rx = lx * cosine - ly * sine;
        let ry = lx * sine + ly * cosine;
        (t.x_mm + rx, t.y_mm + ry)
    })
}

fn bounds_of_points(pts: &[(f64, f64)]) -> Option<(f64, f64, f64, f64)> {
    let &(x0, y0) = pts.first()?;
    let mut min_x = x0;
    let mut min_y = y0;
    let mut max_x = x0;
    let mut max_y = y0;
    for &(x, y) in pts.iter().skip(1) {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    Some((min_x, min_y, max_x, max_y))
}

fn near_polyline(x: f64, y: f64, pts: &[(f64, f64)], closed: bool, tol: f64) -> bool {
    if pts.len() < 2 {
        return false;
    }
    let n = pts.len();
    let segs = if closed { n } else { n - 1 };
    for i in 0..segs {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % n];
        if dist_point_segment(x, y, x1, y1, x2, y2) <= tol {
            return true;
        }
    }
    false
}

fn dist_point_segment(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len2 = dx * dx + dy * dy;
    if len2 < 1e-18 {
        let ex = px - x1;
        let ey = py - y1;
        return (ex * ex + ey * ey).sqrt();
    }
    let t = ((px - x1) * dx + (py - y1) * dy) / len2;
    let t = t.clamp(0.0, 1.0);
    let qx = x1 + t * dx;
    let qy = y1 + t * dy;
    let ex = px - qx;
    let ey = py - qy;
    (ex * ex + ey * ey).sqrt()
}

fn point_in_polygon(x: f64, y: f64, pts: &[(f64, f64)]) -> bool {
    if pts.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = pts.len() - 1;
    for i in 0..pts.len() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::{
        Circle, Color, Document, Ellipse, Frame, Image, Line, Page, PaperSize, Polygon, Polyline,
        Rect, Ring, Shape, Text,
    };

    fn expect_circle(s: &WorldShape) -> &WorldCircle {
        match s {
            WorldShape::Circle(c) => c,
            _ => panic!("expected circle"),
        }
    }
    fn expect_polygon(s: &WorldShape) -> &WorldPolygon {
        match s {
            WorldShape::Polygon(p) => p,
            _ => panic!("expected polygon"),
        }
    }
    fn expect_text(s: &WorldShape) -> &WorldText {
        match s {
            WorldShape::Text(t) => t,
            _ => panic!("expected text"),
        }
    }
    fn expect_image(s: &WorldShape) -> &WorldImage {
        match s {
            WorldShape::Image(i) => i,
            _ => panic!("expected image"),
        }
    }
    fn expect_path(s: &WorldShape) -> &WorldPath {
        match s {
            WorldShape::Path(p) => p,
            _ => panic!("expected path"),
        }
    }

    // --- validity ---

    #[test]
    fn identity_circle_flattens_unchanged() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Circle(Circle {
                x_mm: 105.0,
                y_mm: 148.5,
                radius_mm: 40.0,
                fill: Color::BLACK,
            })],
        });
        let (_, shapes) = flatten_first_page(&doc).unwrap();
        let c = expect_circle(&shapes[0]);
        assert_eq!(c.x_mm, 105.0);
        assert_eq!(c.radius_mm, 40.0);
        assert!(c.stroke_width_mm.is_none());
    }

    #[test]
    fn rect_flattens_to_quad() {
        let shapes = flatten_shapes(
            &[Shape::Rect(Rect {
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 10.0,
                height_mm: 5.0,
                fill: Color::RED,
            })],
            Affine::identity(),
        );
        let p = expect_polygon(&shapes[0]);
        assert_eq!(p.points_mm.len(), 4);
        assert_eq!(p.points_mm[2], (10.0, 5.0));
    }

    #[test]
    fn text_flattens_to_world_text() {
        let shapes = flatten_shapes(
            &[Shape::Text(Text {
                x_mm: 10.0,
                y_mm: 20.0,
                size_mm: 5.0,
                width_mm: None,
                height_mm: None,
                content: "Hi".into(),
                fill: Color::BLACK,
            })],
            Affine::identity(),
        );
        let t = expect_text(&shapes[0]);
        assert_eq!(t.content, "Hi");
        assert_eq!(t.rotation_deg, 0.0);
        assert_eq!(hit_test_shapes(&shapes, 11.0, 22.0), Some(0));
    }

    #[test]
    fn text_carries_parent_rotation() {
        let shapes = flatten_shapes(
            &[Shape::Group {
                transform: Affine::rotate_deg(30.0),
                children: vec![Shape::Text(Text {
                    x_mm: 0.0,
                    y_mm: 0.0,
                    size_mm: 5.0,
                    width_mm: None,
                    height_mm: None,
                    content: "A".into(),
                    fill: Color::BLACK,
                })],
            }],
            Affine::identity(),
        );
        let t = expect_text(&shapes[0]);
        assert!((t.rotation_deg - 30.0).abs() < 1e-9);
    }

    #[test]
    fn image_corners_follow_rotation() {
        let shapes = flatten_shapes(
            &[Shape::Group {
                transform: Affine::translate(-10.0, -20.0)
                    .then(Affine::rotate_deg(90.0))
                    .then(Affine::translate(10.0, 20.0)),
                children: vec![Shape::Image(Image {
                    path: "x.png".into(),
                    x_mm: 10.0,
                    y_mm: 20.0,
                    width_mm: 4.0,
                    height_mm: 2.0,
                })],
            }],
            Affine::identity(),
        );
        let img = expect_image(&shapes[0]);
        // BL stays at pivot (10,20)
        assert!((img.corners_mm[0].0 - 10.0).abs() < 1e-9);
        assert!((img.corners_mm[0].1 - 20.0).abs() < 1e-9);
        // BR (14,20) → relative (4,0) → after 90° CCW (0,4) → (10,24)
        assert!((img.corners_mm[1].0 - 10.0).abs() < 1e-9);
        assert!((img.corners_mm[1].1 - 24.0).abs() < 1e-9);
    }

    #[test]
    fn translate_scale_compose_on_circle() {
        let shapes = vec![Shape::Group {
            transform: Affine::translate(10.0, 20.0),
            children: vec![Shape::Group {
                transform: Affine::scale_uniform(2.0),
                children: vec![Shape::Circle(Circle {
                    x_mm: 1.0,
                    y_mm: 1.0,
                    radius_mm: 5.0,
                    fill: Color::RED,
                })],
            }],
        }];
        let out = flatten_shapes(&shapes, Affine::identity());
        let c = expect_circle(&out[0]);
        assert!((c.x_mm - 12.0).abs() < 1e-9);
        assert!((c.y_mm - 22.0).abs() < 1e-9);
        assert!((c.radius_mm - 10.0).abs() < 1e-9);
    }

    #[test]
    fn rotate_about_object_center_keeps_circle_center() {
        // GUI writes (translate c (rotate deg (translate -c shape))).
        let cx = 10.0;
        let cy = 20.0;
        let shapes = vec![Shape::Group {
            transform: Affine::translate(cx, cy),
            children: vec![Shape::Group {
                transform: Affine::rotate_deg(90.0),
                children: vec![Shape::Group {
                    transform: Affine::translate(-cx, -cy),
                    children: vec![Shape::Circle(Circle {
                        x_mm: cx,
                        y_mm: cy,
                        radius_mm: 5.0,
                        fill: Color::BLACK,
                    })],
                }],
            }],
        }];
        let out = flatten_shapes(&shapes, Affine::identity());
        let c = expect_circle(&out[0]);
        assert!(
            (c.x_mm - cx).abs() < 1e-9 && (c.y_mm - cy).abs() < 1e-9,
            "center drifted to ({}, {})",
            c.x_mm,
            c.y_mm
        );
        assert!((c.radius_mm - 5.0).abs() < 1e-9);
    }

    #[test]
    fn paper_layout_flips_y_and_maps_corners() {
        let layout = PaperLayout::fit(210.0, 297.0, 0.0, 210.0, 297.0);
        let (x0, y0) = layout.mm_to_px(0.0, 297.0);
        assert!((x0 - 0.0).abs() < 1e-3);
        assert!((y0 - 0.0).abs() < 1e-3);
        let (x1, y1) = layout.mm_to_px(0.0, 0.0);
        assert!((y1 - 297.0).abs() < 1e-2);
        assert!((x1 - 0.0).abs() < 1e-3);
    }

    #[test]
    fn px_mm_roundtrip_center() {
        let layout = PaperLayout::fit(420.0, 594.0, 0.0, 210.0, 297.0);
        let (x, y) = layout.mm_to_px(105.0, 148.5);
        let (mx, my) = layout.px_to_mm(x, y);
        assert!((mx - 105.0).abs() < 1e-3);
        assert!((my - 148.5).abs() < 1e-3);
    }

    #[test]
    fn hit_test_prefers_topmost() {
        let shapes = vec![
            WorldShape::Circle(WorldCircle {
                x_mm: 0.0,
                y_mm: 0.0,
                radius_mm: 10.0,
                color: Color::BLACK,
                stroke_width_mm: None,
                alpha: 1.0,
            }),
            WorldShape::Circle(WorldCircle {
                x_mm: 0.0,
                y_mm: 0.0,
                radius_mm: 5.0,
                color: Color::RED,
                stroke_width_mm: None,
                alpha: 1.0,
            }),
        ];
        assert_eq!(hit_test_shapes(&shapes, 0.0, 0.0), Some(1));
        assert_eq!(hit_test_shapes(&shapes, 8.0, 0.0), Some(0));
        assert_eq!(hit_test_shapes(&shapes, 20.0, 0.0), None);
    }

    #[test]
    fn hit_test_rect_interior() {
        let shapes = flatten_shapes(
            &[Shape::Rect(Rect {
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 10.0,
                height_mm: 10.0,
                fill: Color::BLACK,
            })],
            Affine::identity(),
        );
        assert_eq!(hit_test_shapes(&shapes, 5.0, 5.0), Some(0));
        assert_eq!(hit_test_shapes(&shapes, 20.0, 5.0), None);
    }

    #[test]
    fn ellipse_flattens_and_hits_center() {
        let shapes = flatten_shapes(
            &[Shape::Ellipse(Ellipse {
                x_mm: 0.0,
                y_mm: 0.0,
                rx_mm: 10.0,
                ry_mm: 5.0,
                fill: Color::BLUE,
            })],
            Affine::identity(),
        );
        assert!(expect_polygon(&shapes[0]).points_mm.len() >= 8);
        assert_eq!(hit_test_shapes(&shapes, 0.0, 0.0), Some(0));
        assert_eq!(hit_test_shapes(&shapes, 20.0, 0.0), None);
    }

    #[test]
    fn opacity_multiplies_into_world_alpha() {
        let shapes = flatten_shapes(
            &[Shape::Opacity {
                alpha: 0.5,
                children: vec![Shape::Opacity {
                    alpha: 0.5,
                    children: vec![Shape::Circle(Circle {
                        x_mm: 0.0,
                        y_mm: 0.0,
                        radius_mm: 10.0,
                        fill: Color::RED,
                    })],
                }],
            }],
            Affine::identity(),
        );
        assert!((expect_circle(&shapes[0]).alpha - 0.25).abs() < 1e-9);
    }

    #[test]
    fn wrap_text_breaks_long_ascii() {
        let lines = wrap_text_to_width("hello world there", 10.0, 40.0);
        assert!(lines.len() >= 2, "expected wrap: {lines:?}");
        assert!(lines.iter().all(|l| line_width_mm(l, 10.0) <= 40.0 + 1e-6));
    }

    #[test]
    fn wrap_preserves_hard_newlines() {
        let lines = wrap_text_to_width("aa\nbb", 10.0, 100.0);
        assert_eq!(lines, vec!["aa".to_string(), "bb".to_string()]);
    }

    #[test]
    fn wrap_empty_and_narrow_box() {
        assert_eq!(wrap_text_to_width("", 10.0, 5.0), vec![String::new()]);
        let lines = wrap_text_to_width("あいう", 10.0, 5.0);
        assert!(
            lines.len() >= 2,
            "CJK wider than box should break: {lines:?}"
        );
    }

    #[test]
    fn text_extent_multiline_height() {
        let (w, h) = text_extent_mm("line one\nline two\nline three", 12.0);
        assert!((h - 36.0).abs() < 1e-9, "three lines × 12mm: {h}");
        assert!(w > 0.0);
        let (w1, h1) = text_extent_mm("short\nmuch longer line", 10.0);
        assert!((h1 - 20.0).abs() < 1e-9);
        let (w_short, _) = text_extent_mm("short", 10.0);
        assert!(w1 > w_short);
    }

    #[test]
    fn text_corners_multiline_box() {
        let t = WorldText {
            x_mm: 10.0,
            y_mm: 20.0,
            size_mm: 12.0,
            width_mm: text_extent_mm("A\nBC", 12.0).0,
            height_mm: text_extent_mm("A\nBC", 12.0).1,
            rotation_deg: 0.0,
            content: "A\nBC".into(),
            fill: Color::BLACK,
            alpha: 1.0,
            glyph_id: None,
            font_digest: None,
            glyph_advance_mm: None,
        };
        let corners = text_corners_mm(&t);
        let (x0, y0, x1, y1) = bounds_of_points(&corners).unwrap();
        assert!((x1 - x0 - t.width_mm).abs() < 1e-9);
        assert!((y1 - y0 - t.height_mm).abs() < 1e-9);
        assert!((x0 - 10.0).abs() < 1e-9);
        assert!((y0 - 20.0).abs() < 1e-9);
    }

    #[test]
    fn text_hit_uses_padded_box() {
        let shapes = flatten_shapes(
            &[Shape::Text(Text {
                x_mm: 10.0,
                y_mm: 20.0,
                size_mm: 5.0,
                width_mm: None,
                height_mm: None,
                content: "Hi".into(),
                fill: Color::BLACK,
            })],
            Affine::identity(),
        );
        // Near the baseline but not on a thin glyph stem.
        assert_eq!(hit_test_shapes(&shapes, 12.0, 22.0), Some(0));
        assert_eq!(hit_test_shapes(&shapes, 100.0, 20.0), None);
    }

    #[test]
    fn paper_layout_zoom_scales_size() {
        let base = PaperLayout::fit(400.0, 400.0, 0.0, 210.0, 297.0);
        let z = base.with_view(2.0, 0.0, 0.0);
        assert!((z.width_px - base.width_px * 2.0).abs() < 1e-3);
        assert!((z.height_px - base.height_px * 2.0).abs() < 1e-3);
    }

    #[test]
    fn viewport_fit_bounds_centers_selection() {
        let base = PaperLayout::fit(400.0, 300.0, 24.0, 210.0, 297.0);
        let bounds = (80.0, 100.0, 130.0, 150.0);
        let (zoom, px, py) = PaperLayout::viewport_fit_bounds(&base, (400.0, 300.0), bounds, 32.0);
        let layout = base.with_view(zoom, px, py);
        let (mx, my) = layout.px_to_mm(200.0, 150.0);
        let cx = (bounds.0 + bounds.2) * 0.5;
        let cy = (bounds.1 + bounds.3) * 0.5;
        assert!((mx - cx).abs() < 1.0);
        assert!((my - cy).abs() < 1.0);
    }

    #[test]
    fn zoom_anchor_keeps_paper_point_under_cursor() {
        let base = PaperLayout::fit(400.0, 400.0, 24.0, 210.0, 297.0);
        let anchor = (200.0_f32, 180.0_f32);
        let old_zoom = 1.0_f32;
        let old_pan = (0.0_f32, 0.0_f32);
        let old_layout = base.with_view(old_zoom, old_pan.0, old_pan.1);
        let anchor_mm = old_layout.px_to_mm(anchor.0, anchor.1);
        let new_zoom = 2.5_f32;
        let (px, py) = PaperLayout::pan_for_zoom_change(&base, old_zoom, old_pan, new_zoom, anchor);
        let new_layout = base.with_view(new_zoom, px, py);
        let (mx, my) = new_layout.px_to_mm(anchor.0, anchor.1);
        assert!((mx - anchor_mm.0).abs() < 0.05);
        assert!((my - anchor_mm.1).abs() < 0.05);
    }

    #[test]
    fn marquee_selects_intersecting_shapes() {
        let shapes = flatten_shapes(
            &[
                Shape::Circle(Circle {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    radius_mm: 5.0,
                    fill: Color::BLACK,
                }),
                Shape::Circle(Circle {
                    x_mm: 100.0,
                    y_mm: 100.0,
                    radius_mm: 5.0,
                    fill: Color::RED,
                }),
            ],
            Affine::identity(),
        );
        let hit = shapes_intersecting_aabb(&shapes, (0.0, 0.0, 20.0, 20.0));
        assert_eq!(hit, vec![0]);
        let both = shapes_intersecting_aabb(&shapes, (0.0, 0.0, 120.0, 120.0));
        assert_eq!(both, vec![0, 1]);
    }

    // --- defect ---

    #[test]
    fn empty_document_has_no_first_page() {
        assert!(flatten_first_page(&Document::default()).is_none());
    }

    #[test]
    fn zero_det_scale_collapses_radius() {
        let shapes = vec![Shape::Group {
            transform: Affine::scale(0.0, 1.0),
            children: vec![Shape::Circle(Circle {
                x_mm: 0.0,
                y_mm: 0.0,
                radius_mm: 10.0,
                fill: Color::BLACK,
            })],
        }];
        let out = flatten_shapes(&shapes, Affine::identity());
        assert_eq!(expect_circle(&out[0]).radius_mm, 0.0);
    }

    fn ellipse_polygon_sides(options: FlattenOptions) -> usize {
        let shapes = flatten_shapes_with_options(
            &[Shape::Ellipse(Ellipse {
                x_mm: 0.0,
                y_mm: 0.0,
                rx_mm: 10.0,
                ry_mm: 5.0,
                fill: Color::BLUE,
            })],
            Affine::identity(),
            options,
        );
        expect_polygon(&shapes[0]).points_mm.len()
    }

    #[test]
    fn ellipse_sides_zero_clamps_to_three() {
        assert_eq!(
            ellipse_polygon_sides(FlattenOptions { ellipse_sides: 0 }),
            3
        );
    }

    #[test]
    fn ellipse_sides_three_is_minimum() {
        assert_eq!(
            ellipse_polygon_sides(FlattenOptions { ellipse_sides: 3 }),
            3
        );
    }

    #[test]
    fn ellipse_sides_default_is_thirty_two() {
        assert_eq!(ellipse_polygon_sides(FlattenOptions::default()), 32);
    }

    #[test]
    fn ellipse_sides_sixty_four_is_supported() {
        assert_eq!(
            ellipse_polygon_sides(FlattenOptions { ellipse_sides: 64 }),
            64
        );
    }

    #[test]
    fn flatten_page_out_of_range_returns_none() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![],
        });
        assert!(flatten_page(&doc, 1).is_none());
    }

    #[test]
    fn ring_and_frame_flatten_with_stroke() {
        let shapes = flatten_shapes(
            &[
                Shape::Ring(Ring {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    radius_mm: 5.0,
                    width_mm: 0.5,
                    stroke: Color::RED,
                }),
                Shape::Frame(Frame {
                    x_mm: 0.0,
                    y_mm: 0.0,
                    width_mm: 20.0,
                    height_mm: 10.0,
                    stroke_width_mm: 0.3,
                    stroke: Color::BLUE,
                }),
            ],
            Affine::identity(),
        );
        assert!(expect_circle(&shapes[0]).stroke_width_mm.is_some());
        assert!(expect_polygon(&shapes[1]).stroke_width_mm.is_some());
    }

    #[test]
    fn line_and_polyline_flatten_to_paths() {
        let shapes = flatten_shapes(
            &[
                Shape::Line(Line {
                    x1_mm: 0.0,
                    y1_mm: 0.0,
                    x2_mm: 10.0,
                    y2_mm: 0.0,
                    stroke: Color::BLACK,
                    width_mm: 0.2,
                }),
                Shape::Polyline(Polyline {
                    points_mm: vec![(0.0, 0.0), (5.0, 5.0), (10.0, 0.0)],
                    stroke: Color::GREEN,
                    width_mm: 0.1,
                }),
            ],
            Affine::identity(),
        );
        for s in &shapes {
            assert!(!expect_path(s).closed);
        }
    }

    #[test]
    fn polygon_flatten_fills() {
        let shapes = flatten_shapes(
            &[Shape::Polygon(Polygon {
                points_mm: vec![(0.0, 0.0), (10.0, 0.0), (5.0, 10.0)],
                fill: Color::RED,
            })],
            Affine::identity(),
        );
        assert!(expect_polygon(&shapes[0]).stroke_width_mm.is_none());
    }

    #[test]
    fn text_with_explicit_box_dimensions() {
        let shapes = flatten_shapes(
            &[Shape::Text(Text {
                x_mm: 5.0,
                y_mm: 5.0,
                size_mm: 10.0,
                width_mm: Some(50.0),
                height_mm: Some(30.0),
                content: "boxed".into(),
                fill: Color::BLACK,
            })],
            Affine::identity(),
        );
        let t = expect_text(&shapes[0]);
        assert!((t.width_mm - 50.0).abs() < 1e-9);
        assert!((t.height_mm - 30.0).abs() < 1e-9);
    }

    #[test]
    fn hit_test_stroked_ring_near_outline() {
        let shapes = flatten_shapes(
            &[Shape::Ring(Ring {
                x_mm: 0.0,
                y_mm: 0.0,
                radius_mm: 10.0,
                width_mm: 2.0,
                stroke: Color::BLACK,
            })],
            Affine::identity(),
        );
        assert_eq!(hit_test_shapes(&shapes, 10.0, 0.0), Some(0));
        assert_eq!(hit_test_shapes(&shapes, 0.0, 0.0), None);
    }

    #[test]
    fn hit_test_line_segment() {
        let shapes = flatten_shapes(
            &[Shape::Line(Line {
                x1_mm: 0.0,
                y1_mm: 0.0,
                x2_mm: 100.0,
                y2_mm: 0.0,
                stroke: Color::BLACK,
                width_mm: 1.0,
            })],
            Affine::identity(),
        );
        assert_eq!(hit_test_shapes(&shapes, 50.0, 0.0), Some(0));
        assert_eq!(hit_test_shapes(&shapes, 50.0, 50.0), None);
    }

    #[test]
    fn hit_test_stroked_frame_border() {
        let shapes = flatten_shapes(
            &[Shape::Frame(Frame {
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 20.0,
                height_mm: 20.0,
                stroke_width_mm: 2.0,
                stroke: Color::BLACK,
            })],
            Affine::identity(),
        );
        assert_eq!(hit_test_shapes(&shapes, 1.0, 1.0), Some(0));
        assert_eq!(hit_test_shapes(&shapes, 10.0, 10.0), None);
    }

    #[test]
    fn wrapped_text_height_matches_line_count() {
        let h = wrapped_text_height_mm("one two three four", 10.0, 25.0);
        assert!(h >= 20.0);
    }

    #[test]
    fn wrap_prefers_whitespace_break() {
        let lines = wrap_text_to_width("hello world", 10.0, 30.0);
        assert!(lines.len() >= 2);
        assert!(lines[0].ends_with("hello") || lines[0] == "hello");
    }

    #[test]
    fn radius_mm_to_px_scales_with_layout() {
        let layout = PaperLayout::fit(210.0, 297.0, 0.0, 210.0, 297.0);
        let r = layout.radius_mm_to_px(10.0);
        assert!((r - 10.0).abs() < 1e-3);
    }

    #[test]
    fn pan_for_zoom_anchor_direct() {
        let base = PaperLayout::fit(400.0, 400.0, 0.0, 210.0, 297.0);
        let (px, py) = PaperLayout::pan_for_zoom_anchor(&base, 2.0, (100.0, 100.0), (50.0, 50.0));
        let layout = base.with_view(2.0, px, py);
        let (mx, my) = layout.px_to_mm(100.0, 100.0);
        assert!((mx - 50.0).abs() < 0.1);
        assert!((my - 50.0).abs() < 0.1);
    }

    #[test]
    fn shape_bounds_cover_all_kinds() {
        let shapes = flatten_shapes(
            &[
                Shape::Circle(Circle {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    radius_mm: 5.0,
                    fill: Color::BLACK,
                }),
                Shape::Image(Image {
                    path: "x.png".into(),
                    x_mm: 0.0,
                    y_mm: 0.0,
                    width_mm: 10.0,
                    height_mm: 10.0,
                }),
            ],
            Affine::identity(),
        );
        for s in &shapes {
            assert!(PaperLayout::shape_bounds_mm(s).is_some());
        }
        let path = WorldShape::Path(WorldPath {
            points_mm: vec![(0.0, 0.0), (5.0, 5.0)],
            stroke: Color::BLACK,
            width_mm: 1.0,
            closed: true,
            alpha: 1.0,
        });
        assert!(PaperLayout::shape_bounds_mm(&path).is_some());
        assert_eq!(hit_test_shapes(&[path], 2.0, 2.0), Some(0));
    }

    #[test]
    fn marquee_normalizes_reversed_aabb() {
        let shapes = flatten_shapes(
            &[Shape::Circle(Circle {
                x_mm: 10.0,
                y_mm: 10.0,
                radius_mm: 5.0,
                fill: Color::BLACK,
            })],
            Affine::identity(),
        );
        let hit = shapes_intersecting_aabb(&shapes, (20.0, 20.0, 0.0, 0.0));
        assert_eq!(hit, vec![0]);
    }

    #[test]
    fn paper_layout_wide_page_uses_width_constraint() {
        let layout = PaperLayout::fit(400.0, 200.0, 10.0, 400.0, 100.0);
        assert!(layout.width_px > layout.height_px);
    }

    #[test]
    fn shape_bounds_polygon_text_and_image_hit() {
        let poly = WorldShape::Polygon(WorldPolygon {
            points_mm: vec![(0.0, 0.0), (10.0, 0.0), (5.0, 10.0)],
            color: Color::RED,
            stroke_width_mm: None,
            alpha: 1.0,
        });
        let text = WorldShape::Text(WorldText {
            x_mm: 0.0,
            y_mm: 0.0,
            size_mm: 10.0,
            width_mm: 20.0,
            height_mm: 10.0,
            rotation_deg: 0.0,
            content: "x".into(),
            fill: Color::BLACK,
            alpha: 1.0,
            glyph_id: None,
            font_digest: None,
            glyph_advance_mm: None,
        });
        let img = WorldShape::Image(WorldImage {
            path: "a.png".into(),
            corners_mm: [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)],
            alpha: 1.0,
        });
        assert!(PaperLayout::shape_bounds_mm(&poly).is_some());
        assert!(PaperLayout::shape_bounds_mm(&text).is_some());
        assert_eq!(hit_test_shapes(&[img], 5.0, 5.0), Some(0));
    }

    #[test]
    fn char_width_tab_and_empty_text_extent() {
        assert!((char_width_mm('\t', 10.0) - 20.0).abs() < 1e-9);
        let (w, h) = text_extent_mm("", 10.0);
        assert!(w > 0.0 && h > 0.0);
    }

    #[test]
    fn wrap_blank_line_in_paragraph() {
        let lines = wrap_text_to_width("\n", 10.0, 50.0);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].is_empty());
    }

    #[test]
    fn hit_test_degenerate_path_and_polygon() {
        let one_pt = WorldShape::Path(WorldPath {
            points_mm: vec![(0.0, 0.0)],
            stroke: Color::BLACK,
            width_mm: 1.0,
            closed: false,
            alpha: 1.0,
        });
        assert_eq!(hit_test_shapes(&[one_pt], 0.0, 0.0), None);
        let two_pt = WorldShape::Polygon(WorldPolygon {
            points_mm: vec![(0.0, 0.0), (1.0, 0.0)],
            color: Color::BLACK,
            stroke_width_mm: None,
            alpha: 1.0,
        });
        assert_eq!(hit_test_shapes(&[two_pt], 0.5, 0.0), None);
    }

    #[test]
    fn near_polyline_zero_length_segment() {
        let path = WorldShape::Path(WorldPath {
            points_mm: vec![(0.0, 0.0), (0.0, 0.0), (10.0, 0.0)],
            stroke: Color::BLACK,
            width_mm: 2.0,
            closed: false,
            alpha: 1.0,
        });
        assert_eq!(hit_test_shapes(&[path], 0.0, 0.0), Some(0));
    }

    #[test]
    fn expect_helpers_reject_wrong_shape() {
        let circle = WorldShape::Circle(WorldCircle {
            x_mm: 0.0,
            y_mm: 0.0,
            radius_mm: 1.0,
            color: Color::BLACK,
            stroke_width_mm: None,
            alpha: 1.0,
        });
        let poly = WorldShape::Polygon(WorldPolygon {
            points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
            color: Color::RED,
            stroke_width_mm: None,
            alpha: 1.0,
        });
        use std::panic::{catch_unwind, AssertUnwindSafe};
        assert!(catch_unwind(AssertUnwindSafe(|| expect_circle(&poly))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| expect_polygon(&circle))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| expect_text(&circle))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| expect_image(&circle))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| expect_path(&circle))).is_err());
    }

    #[test]
    fn empty_polygon_and_path_have_no_bounds_and_skip_marquee() {
        let empty_poly = WorldShape::Polygon(WorldPolygon {
            points_mm: vec![],
            color: Color::BLACK,
            stroke_width_mm: None,
            alpha: 1.0,
        });
        let empty_path = WorldShape::Path(WorldPath {
            points_mm: vec![],
            stroke: Color::BLACK,
            width_mm: 1.0,
            closed: false,
            alpha: 1.0,
        });
        assert!(PaperLayout::shape_bounds_mm(&empty_poly).is_none());
        assert!(PaperLayout::shape_bounds_mm(&empty_path).is_none());
        assert!(bounds_of_points(&[]).is_none());
        let idx = shapes_intersecting_aabb(&[empty_poly, empty_path], (0.0, 0.0, 10.0, 10.0));
        assert!(idx.is_empty());
    }
}
