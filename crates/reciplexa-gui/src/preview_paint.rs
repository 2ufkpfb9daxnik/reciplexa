//! Preview paint and selection/hit geometry helpers (egui paper surface).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eframe::egui;
use reciplexa_view::{text_corners_mm, PaperLayout, WorldShape};

#[derive(Debug, Clone, Copy)]
pub enum ScaleCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Copy)]
pub enum ScaleEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub enum ScaleGrab {
    Corner(ScaleCorner),
    Edge(ScaleEdge),
}

#[derive(Debug, Clone, Copy)]
pub struct BoxDrag {
    pub grab: ScaleGrab,
    /// Page AABB at drag start: (x0, y0, x1, y1) with y-up.
    pub start_bounds: (f64, f64, f64, f64),
}

pub fn paint_hover_frame(
    painter: &egui::Painter,
    rect: egui::Rect,
    layout: &PaperLayout,
    bounds: (f64, f64, f64, f64),
) {
    let (x0, y0, x1, y1) = bounds;
    let (ax, ay) = layout.mm_to_px(x0, y1);
    let (bx, by) = layout.mm_to_px(x1, y0);
    let frame = egui::Rect::from_min_max(
        rect.min + egui::vec2(ax.min(bx) - 2.0, ay.min(by) - 2.0),
        rect.min + egui::vec2(ax.max(bx) + 2.0, ay.max(by) + 2.0),
    );
    painter.rect_stroke(
        frame,
        0.0,
        egui::Stroke::new(
            1.0_f32,
            egui::Color32::from_rgba_unmultiplied(30, 120, 220, 120),
        ),
        egui::StrokeKind::Outside,
    );
}

pub fn paint_selection_frame(
    painter: &egui::Painter,
    rect: egui::Rect,
    layout: &PaperLayout,
    bounds: (f64, f64, f64, f64),
) {
    let (x0, y0, x1, y1) = bounds;
    let (ax, ay) = layout.mm_to_px(x0, y1);
    let (bx, by) = layout.mm_to_px(x1, y0);
    let frame = egui::Rect::from_min_max(
        rect.min + egui::vec2(ax.min(bx) - 3.0, ay.min(by) - 3.0),
        rect.min + egui::vec2(ax.max(bx) + 3.0, ay.max(by) + 3.0),
    );
    painter.rect_stroke(
        frame,
        0.0,
        egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(30, 120, 220)),
        egui::StrokeKind::Outside,
    );
    let handle = 5.0;
    let corners = [
        frame.left_top(),
        frame.right_top(),
        frame.left_bottom(),
        frame.right_bottom(),
    ];
    for c in corners {
        let hr = egui::Rect::from_center_size(c, egui::vec2(handle * 2.0, handle * 2.0));
        painter.rect_filled(hr, 0.0, egui::Color32::WHITE);
        painter.rect_stroke(
            hr,
            0.0,
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(30, 120, 220)),
            egui::StrokeKind::Outside,
        );
    }
    let edges = [
        frame.center_top(),
        frame.center_bottom(),
        frame.left_center(),
        frame.right_center(),
    ];
    for c in edges {
        let hr = egui::Rect::from_center_size(c, egui::vec2(handle * 1.6, handle * 1.6));
        painter.rect_filled(hr, 0.0, egui::Color32::WHITE);
        painter.rect_stroke(
            hr,
            0.0,
            egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(30, 120, 220)),
            egui::StrokeKind::Outside,
        );
    }
    // Rotate knob: above the top-center of the selection frame.
    let knob = rotate_handle_pos(frame);
    painter.line_segment(
        [egui::pos2(frame.center().x, frame.top()), knob],
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(30, 120, 220)),
    );
    painter.circle_filled(knob, 5.0, egui::Color32::WHITE);
    painter.circle_stroke(
        knob,
        5.0,
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(30, 120, 220)),
    );
}

fn selection_frame_local(layout: &PaperLayout, bounds: (f64, f64, f64, f64)) -> egui::Rect {
    let (x0, y0, x1, y1) = bounds;
    let (ax, ay) = layout.mm_to_px(x0, y1);
    let (bx, by) = layout.mm_to_px(x1, y0);
    egui::Rect::from_min_max(
        egui::pos2(ax.min(bx) - 3.0, ay.min(by) - 3.0),
        egui::pos2(ax.max(bx) + 3.0, ay.max(by) + 3.0),
    )
}

fn rotate_handle_pos(frame: egui::Rect) -> egui::Pos2 {
    egui::pos2(frame.center().x, frame.top() - 22.0)
}

pub fn hit_scale_grab(
    layout: &PaperLayout,
    bounds: (f64, f64, f64, f64),
    local_px: egui::Pos2,
) -> Option<ScaleGrab> {
    let frame = selection_frame_local(layout, bounds);
    let hit_r2 = 10.0_f32 * 10.0;
    let corners = [
        (frame.left_top(), ScaleGrab::Corner(ScaleCorner::TopLeft)),
        (frame.right_top(), ScaleGrab::Corner(ScaleCorner::TopRight)),
        (
            frame.left_bottom(),
            ScaleGrab::Corner(ScaleCorner::BottomLeft),
        ),
        (
            frame.right_bottom(),
            ScaleGrab::Corner(ScaleCorner::BottomRight),
        ),
    ];
    if let Some((_, grab)) = corners
        .into_iter()
        .find(|(c, _)| local_px.distance_sq(*c) <= hit_r2)
    {
        return Some(grab);
    }
    let edges = [
        (frame.center_top(), ScaleGrab::Edge(ScaleEdge::Top)),
        (frame.center_bottom(), ScaleGrab::Edge(ScaleEdge::Bottom)),
        (frame.left_center(), ScaleGrab::Edge(ScaleEdge::Left)),
        (frame.right_center(), ScaleGrab::Edge(ScaleEdge::Right)),
    ];
    edges
        .into_iter()
        .find(|(c, _)| local_px.distance_sq(*c) <= hit_r2)
        .map(|(_, grab)| grab)
}

pub fn box_from_grab(tb: BoxDrag, mx: f64, my: f64) -> (f64, f64, f64, f64) {
    let (x0, y0, x1, y1) = tb.start_bounds;
    const MIN: f64 = 0.5;
    match tb.grab {
        ScaleGrab::Corner(ScaleCorner::TopLeft) => {
            let w = (x1 - mx).max(MIN);
            let h = (my - y0).max(MIN);
            (x1 - w, y0, w, h)
        }
        ScaleGrab::Corner(ScaleCorner::TopRight) => {
            let w = (mx - x0).max(MIN);
            let h = (my - y0).max(MIN);
            (x0, y0, w, h)
        }
        ScaleGrab::Corner(ScaleCorner::BottomLeft) => {
            let w = (x1 - mx).max(MIN);
            let h = (y1 - my).max(MIN);
            (x1 - w, y1 - h, w, h)
        }
        ScaleGrab::Corner(ScaleCorner::BottomRight) => {
            let w = (mx - x0).max(MIN);
            let h = (y1 - my).max(MIN);
            (x0, y1 - h, w, h)
        }
        ScaleGrab::Edge(ScaleEdge::Left) => {
            let w = (x1 - mx).max(MIN);
            (x1 - w, y0, w, y1 - y0)
        }
        ScaleGrab::Edge(ScaleEdge::Right) => {
            let w = (mx - x0).max(MIN);
            (x0, y0, w, y1 - y0)
        }
        ScaleGrab::Edge(ScaleEdge::Top) => {
            let h = (my - y0).max(MIN);
            (x0, y0, x1 - x0, h)
        }
        ScaleGrab::Edge(ScaleEdge::Bottom) => {
            let h = (y1 - my).max(MIN);
            (x0, y1 - h, x1 - x0, h)
        }
    }
}

/// Apply Shift aspect-ratio lock for corner resizes (keeps the fixed edges).
pub fn apply_aspect_lock(
    tb: BoxDrag,
    mut nx: f64,
    mut ny: f64,
    mut nw: f64,
    mut nh: f64,
) -> (f64, f64, f64, f64) {
    let ScaleGrab::Corner(corner) = tb.grab else {
        return (nx, ny, nw, nh);
    };
    let (sx0, sy0, sx1, sy1) = tb.start_bounds;
    let sw = (sx1 - sx0).max(1e-9);
    let sh = (sy1 - sy0).max(1e-9);
    let aspect = sw / sh;
    const MIN: f64 = 0.5;
    if (nw - sw).abs() >= (nh - sh).abs() {
        nh = (nw / aspect).max(MIN);
        nw = (nh * aspect).max(MIN);
    } else {
        nw = (nh * aspect).max(MIN);
        nh = (nw / aspect).max(MIN);
    }
    match corner {
        ScaleCorner::TopLeft => {
            nx = sx1 - nw;
            ny = sy0;
        }
        ScaleCorner::TopRight => {
            nx = sx0;
            ny = sy0;
        }
        ScaleCorner::BottomLeft => {
            nx = sx1 - nw;
            ny = sy1 - nh;
        }
        ScaleCorner::BottomRight => {
            nx = sx0;
            ny = sy1 - nh;
        }
    }
    (nx, ny, nw, nh)
}

pub fn snap_mm(v: f64, grid: f64) -> f64 {
    (v / grid).round() * grid
}

pub fn paint_line_endpoints(
    painter: &egui::Painter,
    rect: egui::Rect,
    layout: &PaperLayout,
    points_mm: &[(f64, f64)],
) {
    for &(x, y) in points_mm {
        let (px, py) = layout.mm_to_px(x, y);
        let c = rect.min + egui::vec2(px, py);
        painter.circle_filled(c, 5.0, egui::Color32::WHITE);
        painter.circle_stroke(
            c,
            5.0,
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(30, 120, 220)),
        );
    }
}

pub fn hit_line_endpoint(
    layout: &PaperLayout,
    points_mm: &[(f64, f64)],
    local_px: egui::Pos2,
) -> Option<usize> {
    let hit_r2 = 10.0_f32 * 10.0;
    points_mm.iter().enumerate().find_map(|(i, &(x, y))| {
        let (px, py) = layout.mm_to_px(x, y);
        let c = egui::pos2(px, py);
        if local_px.distance_sq(c) <= hit_r2 {
            Some(i)
        } else {
            None
        }
    })
}

pub fn paint_paper_grid(
    painter: &egui::Painter,
    rect: egui::Rect,
    layout: &PaperLayout,
    paper_w_mm: f64,
    paper_h_mm: f64,
) {
    const STEP: f64 = 10.0;
    let stroke = egui::Stroke::new(
        1.0_f32,
        egui::Color32::from_rgba_unmultiplied(40, 80, 140, 35),
    );
    let mut x = 0.0;
    while x <= paper_w_mm + 1e-6 {
        let (px0, py0) = layout.mm_to_px(x, 0.0);
        let (px1, py1) = layout.mm_to_px(x, paper_h_mm);
        painter.line_segment(
            [
                rect.min + egui::vec2(px0, py0),
                rect.min + egui::vec2(px1, py1),
            ],
            stroke,
        );
        x += STEP;
    }
    let mut y = 0.0;
    while y <= paper_h_mm + 1e-6 {
        let (px0, py0) = layout.mm_to_px(0.0, y);
        let (px1, py1) = layout.mm_to_px(paper_w_mm, y);
        painter.line_segment(
            [
                rect.min + egui::vec2(px0, py0),
                rect.min + egui::vec2(px1, py1),
            ],
            stroke,
        );
        y += STEP;
    }
}

pub fn hit_rotate_handle(
    layout: &PaperLayout,
    bounds: (f64, f64, f64, f64),
    local_px: egui::Pos2,
) -> bool {
    let frame = selection_frame_local(layout, bounds);
    let knob = rotate_handle_pos(frame);
    local_px.distance_sq(knob) <= 12.0_f32 * 12.0
}

pub fn paint_shape(
    painter: &egui::Painter,
    rect: egui::Rect,
    layout: &PaperLayout,
    shape: &WorldShape,
    textures: &mut HashMap<String, egui::TextureHandle>,
    ctx: &egui::Context,
    base: Option<&std::path::Path>,
) {
    match shape {
        WorldShape::Circle(c) => {
            let (x, y) = layout.mm_to_px(c.x_mm, c.y_mm);
            let r = layout.radius_mm_to_px(c.radius_mm);
            let center = rect.min + egui::vec2(x, y);
            match c.stroke_width_mm {
                None => {
                    painter.circle_filled(center, r, color32(c.color, c.alpha));
                }
                Some(w) => {
                    painter.circle_stroke(
                        center,
                        r,
                        egui::Stroke::new(
                            layout.radius_mm_to_px(w).max(1.0),
                            color32(c.color, c.alpha),
                        ),
                    );
                }
            }
        }
        WorldShape::Polygon(p) => {
            if p.points_mm.len() < 3 {
                return;
            }
            let mut points = Vec::with_capacity(p.points_mm.len());
            for &(x_mm, y_mm) in &p.points_mm {
                let (x, y) = layout.mm_to_px(x_mm, y_mm);
                points.push(rect.min + egui::vec2(x, y));
            }
            match p.stroke_width_mm {
                None => {
                    painter.add(egui::Shape::convex_polygon(
                        points,
                        color32(p.color, p.alpha),
                        egui::Stroke::NONE,
                    ));
                }
                Some(w) => {
                    let stroke = egui::Stroke::new(
                        layout.radius_mm_to_px(w).max(1.0),
                        color32(p.color, p.alpha),
                    );
                    for i in 0..points.len() {
                        let a = points[i];
                        let b = points[(i + 1) % points.len()];
                        painter.line_segment([a, b], stroke);
                    }
                }
            }
        }
        WorldShape::Text(t) => {
            let font_px = layout.radius_mm_to_px(t.size_mm).max(0.5);
            let color = color32(t.fill, t.alpha);
            let wrap_px = layout.radius_mm_to_px(t.width_mm).max(1.0);
            // Soft-wrap to the layout box width; hard newlines still break.
            let galley = painter.layout(
                t.content.clone(),
                egui::FontId::proportional(font_px),
                color,
                wrap_px,
            );
            // Baseline in screen pixels (page Y-up → screen Y-down).
            let (bx, by) = layout.mm_to_px(t.x_mm, t.y_mm);
            let baseline = rect.min + egui::vec2(bx, by);
            // Page CCW angle appears as clockwise in Y-down screen space.
            let angle = t.rotation_deg.to_radians() as f32;
            let h = galley.size().y;
            // Unrotated top-left is above the baseline; rotate that offset around baseline.
            let tl_rel = egui::vec2(0.0, -h);
            let (s, c) = (angle.sin(), angle.cos());
            let top_left =
                baseline + egui::vec2(tl_rel.x * c + tl_rel.y * s, -tl_rel.x * s + tl_rel.y * c);
            // Clip to the layout box AABB so height shrinks hide overflow.
            let box_corners = text_corners_mm(t);
            let mut clip = egui::Rect::NOTHING;
            for &(xmm, ymm) in &box_corners {
                let (px, py) = layout.mm_to_px(xmm, ymm);
                clip = clip.union(egui::Rect::from_center_size(
                    rect.min + egui::vec2(px, py),
                    egui::vec2(1.0, 1.0),
                ));
            }
            clip = clip.expand(2.0).intersect(painter.clip_rect());
            let clipped = painter.with_clip_rect(clip);
            clipped.add(egui::epaint::TextShape::new(top_left, galley, color).with_angle(angle));
        }
        WorldShape::Path(p) => {
            if p.points_mm.len() < 2 {
                return;
            }
            let stroke = egui::Stroke::new(
                layout.radius_mm_to_px(p.width_mm).max(1.0),
                color32(p.stroke, p.alpha),
            );
            let mut pts = Vec::with_capacity(p.points_mm.len());
            for &(x_mm, y_mm) in &p.points_mm {
                let (x, y) = layout.mm_to_px(x_mm, y_mm);
                pts.push(rect.min + egui::vec2(x, y));
            }
            let n = pts.len();
            let segs = if p.closed { n } else { n - 1 };
            for i in 0..segs {
                painter.line_segment([pts[i], pts[(i + 1) % n]], stroke);
            }
        }
        WorldShape::Image(img) => {
            let mut screen = [egui::pos2(0.0, 0.0); 4];
            for (i, &(x_mm, y_mm)) in img.corners_mm.iter().enumerate() {
                let (x, y) = layout.mm_to_px(x_mm, y_mm);
                screen[i] = rect.min + egui::vec2(x, y);
            }
            // corners: BL, BR, TR, TL — UVs match image space (V grows down).
            let uvs = [
                egui::pos2(0.0, 1.0),
                egui::pos2(1.0, 1.0),
                egui::pos2(1.0, 0.0),
                egui::pos2(0.0, 0.0),
            ];
            if let Some(tex) = ensure_texture(textures, ctx, &img.path, base) {
                let tint = egui::Color32::from_rgba_unmultiplied(
                    255,
                    255,
                    255,
                    (img.alpha * 255.0).round().clamp(0.0, 255.0) as u8,
                );
                let mut mesh = egui::Mesh::with_texture(tex.id());
                let i0 = mesh.vertices.len() as u32;
                for i in 0..4 {
                    mesh.vertices.push(egui::epaint::Vertex {
                        pos: screen[i],
                        uv: uvs[i],
                        color: tint,
                    });
                }
                mesh.indices
                    .extend_from_slice(&[i0, i0 + 1, i0 + 2, i0, i0 + 2, i0 + 3]);
                painter.add(egui::Shape::mesh(mesh));
            } else {
                let stroke = egui::Stroke::new(1.5_f32, egui::Color32::from_gray(60));
                for i in 0..4 {
                    painter.line_segment([screen[i], screen[(i + 1) % 4]], stroke);
                }
                painter.line_segment(
                    [screen[0], screen[2]],
                    egui::Stroke::new(1.0_f32, egui::Color32::from_gray(140)),
                );
            }
        }
    }
}

fn ensure_texture(
    textures: &mut HashMap<String, egui::TextureHandle>,
    ctx: &egui::Context,
    path: &str,
    base: Option<&Path>,
) -> Option<egui::TextureHandle> {
    if let Some(tex) = textures.get(path) {
        return Some(tex.clone());
    }
    let resolved = match base {
        Some(b) => b.join(path),
        None => PathBuf::from(path),
    };
    let raster = reciplexa_pdf::load_raster_file(&resolved).ok()?;
    let mut rgba = Vec::with_capacity(raster.rgb.len() / 3 * 4);
    for px in raster.rgb.chunks_exact(3) {
        rgba.push(px[0]);
        rgba.push(px[1]);
        rgba.push(px[2]);
        rgba.push(255);
    }
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [raster.width as usize, raster.height as usize],
        &rgba,
    );
    let tex = ctx.load_texture(path.to_string(), color_image, egui::TextureOptions::LINEAR);
    textures.insert(path.to_string(), tex.clone());
    Some(tex)
}

fn color32(c: reciplexa_scene::Color, alpha: f64) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (c.r * 255.0).round().clamp(0.0, 255.0) as u8,
        (c.g * 255.0).round().clamp(0.0, 255.0) as u8,
        (c.b * 255.0).round().clamp(0.0, 255.0) as u8,
        (alpha * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::Color;
    use reciplexa_view::{WorldCircle, WorldImage, WorldPath, WorldPolygon, WorldText};

    fn layout() -> PaperLayout {
        PaperLayout::fit(400.0, 600.0, 10.0, 210.0, 297.0)
    }

    fn with_painter(f: impl FnOnce(&egui::Context, egui::Rect, &egui::Painter)) {
        let ctx = egui::Context::default();
        let mut f = Some(f);
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(f) = f.take() {
                    let rect = ui.max_rect();
                    f(ctx, rect, ui.painter());
                }
            });
        });
    }

    #[test]
    fn snap_mm_rounds_to_grid() {
        assert!((snap_mm(12.0, 5.0) - 10.0).abs() < 1e-9);
        assert!((snap_mm(13.0, 5.0) - 15.0).abs() < 1e-9);
    }

    #[test]
    fn box_from_grab_all_corners_and_edges() {
        let bounds = (10.0, 20.0, 30.0, 40.0);
        let cases = [
            (ScaleGrab::Corner(ScaleCorner::TopLeft), 5.0, 50.0),
            (ScaleGrab::Corner(ScaleCorner::TopRight), 50.0, 50.0),
            (ScaleGrab::Corner(ScaleCorner::BottomLeft), 5.0, 10.0),
            (ScaleGrab::Corner(ScaleCorner::BottomRight), 50.0, 10.0),
            (ScaleGrab::Edge(ScaleEdge::Left), 5.0, 30.0),
            (ScaleGrab::Edge(ScaleEdge::Right), 50.0, 30.0),
            (ScaleGrab::Edge(ScaleEdge::Top), 20.0, 50.0),
            (ScaleGrab::Edge(ScaleEdge::Bottom), 20.0, 10.0),
        ];
        for (grab, mx, my) in cases {
            let tb = BoxDrag {
                grab,
                start_bounds: bounds,
            };
            let (x, y, w, h) = box_from_grab(tb, mx, my);
            assert!(w >= 0.5 && h >= 0.5, "{grab:?} -> {x},{y},{w},{h}");
        }
    }

    #[test]
    fn apply_aspect_lock_corners_and_edge_noop() {
        let bounds = (0.0, 0.0, 20.0, 10.0);
        for corner in [
            ScaleCorner::TopLeft,
            ScaleCorner::TopRight,
            ScaleCorner::BottomLeft,
            ScaleCorner::BottomRight,
        ] {
            let tb = BoxDrag {
                grab: ScaleGrab::Corner(corner),
                start_bounds: bounds,
            };
            let (nx, ny, nw, nh) = apply_aspect_lock(tb, 0.0, 0.0, 40.0, 10.0);
            assert!((nw / nh - 2.0).abs() < 1e-6, "{corner:?} {nw}x{nh}");
            let _ = (nx, ny);
            // Height-dominant delta path.
            let (nx2, ny2, nw2, nh2) = apply_aspect_lock(tb, 0.0, 0.0, 10.0, 40.0);
            assert!((nw2 / nh2 - 2.0).abs() < 1e-6, "{corner:?} {nw2}x{nh2}");
            let _ = (nx2, ny2);
        }
        let edge = BoxDrag {
            grab: ScaleGrab::Edge(ScaleEdge::Left),
            start_bounds: bounds,
        };
        assert_eq!(apply_aspect_lock(edge, 1.0, 2.0, 3.0, 4.0), (1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn hit_helpers_corners_edges_rotate_and_line() {
        let layout = layout();
        let bounds = (20.0, 40.0, 80.0, 120.0);
        let frame = selection_frame_local(&layout, bounds);
        assert!(hit_scale_grab(&layout, bounds, frame.left_top()).is_some());
        assert!(hit_scale_grab(&layout, bounds, frame.right_top()).is_some());
        assert!(hit_scale_grab(&layout, bounds, frame.left_bottom()).is_some());
        assert!(hit_scale_grab(&layout, bounds, frame.right_bottom()).is_some());
        assert!(hit_scale_grab(&layout, bounds, frame.center_top()).is_some());
        assert!(hit_scale_grab(&layout, bounds, frame.center_bottom()).is_some());
        assert!(hit_scale_grab(&layout, bounds, frame.left_center()).is_some());
        assert!(hit_scale_grab(&layout, bounds, frame.right_center()).is_some());
        assert!(hit_scale_grab(&layout, bounds, egui::pos2(0.0, 0.0)).is_none());
        assert!(hit_rotate_handle(
            &layout,
            bounds,
            rotate_handle_pos(frame)
        ));
        assert!(!hit_rotate_handle(&layout, bounds, egui::pos2(0.0, 0.0)));

        let pts = [(10.0, 10.0), (50.0, 50.0)];
        let (px, py) = layout.mm_to_px(10.0, 10.0);
        assert_eq!(
            hit_line_endpoint(&layout, &pts, egui::pos2(px, py)),
            Some(0)
        );
        assert_eq!(
            hit_line_endpoint(&layout, &pts, egui::pos2(0.0, 0.0)),
            None
        );
    }

    #[test]
    fn color32_clamps_channels() {
        let c = color32(
            Color {
                r: 2.0,
                g: -1.0,
                b: 0.5,
            },
            2.0,
        );
        assert_eq!(c.r(), 255);
        assert_eq!(c.g(), 0);
        assert_eq!(c.a(), 255);
    }

    #[test]
    fn paint_frames_grid_endpoints_and_shapes() {
        let layout = layout();
        let bounds = (10.0, 20.0, 60.0, 90.0);
        with_painter(|ctx, rect, painter| {
            paint_hover_frame(painter, rect, &layout, bounds);
            paint_selection_frame(painter, rect, &layout, bounds);
            paint_paper_grid(painter, rect, &layout, 210.0, 297.0);
            paint_line_endpoints(painter, rect, &layout, &[(30.0, 40.0), (50.0, 60.0)]);

            let mut textures = HashMap::new();
            paint_shape(
                painter,
                rect,
                &layout,
                &WorldShape::Circle(WorldCircle {
                    x_mm: 40.0,
                    y_mm: 50.0,
                    radius_mm: 10.0,
                    color: Color::RED,
                    stroke_width_mm: None,
                    alpha: 1.0,
                }),
                &mut textures,
                ctx,
                None,
            );
            paint_shape(
                painter,
                rect,
                &layout,
                &WorldShape::Circle(WorldCircle {
                    x_mm: 40.0,
                    y_mm: 50.0,
                    radius_mm: 10.0,
                    color: Color::BLUE,
                    stroke_width_mm: Some(1.0),
                    alpha: 0.8,
                }),
                &mut textures,
                ctx,
                None,
            );
            paint_shape(
                painter,
                rect,
                &layout,
                &WorldShape::Polygon(WorldPolygon {
                    points_mm: vec![(10.0, 10.0), (20.0, 10.0)],
                    color: Color::GREEN,
                    stroke_width_mm: None,
                    alpha: 1.0,
                }),
                &mut textures,
                ctx,
                None,
            );
            paint_shape(
                painter,
                rect,
                &layout,
                &WorldShape::Polygon(WorldPolygon {
                    points_mm: vec![(10.0, 10.0), (30.0, 10.0), (20.0, 30.0)],
                    color: Color::GREEN,
                    stroke_width_mm: None,
                    alpha: 1.0,
                }),
                &mut textures,
                ctx,
                None,
            );
            paint_shape(
                painter,
                rect,
                &layout,
                &WorldShape::Polygon(WorldPolygon {
                    points_mm: vec![(10.0, 10.0), (30.0, 10.0), (20.0, 30.0)],
                    color: Color::GREEN,
                    stroke_width_mm: Some(0.5),
                    alpha: 1.0,
                }),
                &mut textures,
                ctx,
                None,
            );
            paint_shape(
                painter,
                rect,
                &layout,
                &WorldShape::Text(WorldText {
                    x_mm: 25.0,
                    y_mm: 270.0,
                    size_mm: 8.0,
                    width_mm: 100.0,
                    height_mm: 20.0,
                    rotation_deg: 15.0,
                    content: "Hello".into(),
                    fill: Color::BLACK,
                    alpha: 1.0,
                }),
                &mut textures,
                ctx,
                None,
            );
            paint_shape(
                painter,
                rect,
                &layout,
                &WorldShape::Path(WorldPath {
                    points_mm: vec![(1.0, 1.0)],
                    stroke: Color::BLACK,
                    width_mm: 1.0,
                    closed: false,
                    alpha: 1.0,
                }),
                &mut textures,
                ctx,
                None,
            );
            paint_shape(
                painter,
                rect,
                &layout,
                &WorldShape::Path(WorldPath {
                    points_mm: vec![(10.0, 10.0), (40.0, 40.0), (10.0, 40.0)],
                    stroke: Color::RED,
                    width_mm: 1.0,
                    closed: true,
                    alpha: 1.0,
                }),
                &mut textures,
                ctx,
                None,
            );
            paint_shape(
                painter,
                rect,
                &layout,
                &WorldShape::Image(WorldImage {
                    path: "missing.png".into(),
                    corners_mm: [
                        (10.0, 10.0),
                        (40.0, 10.0),
                        (40.0, 40.0),
                        (10.0, 40.0),
                    ],
                    alpha: 0.5,
                }),
                &mut textures,
                ctx,
                None,
            );
        });
    }

    #[test]
    fn ensure_texture_loads_real_raster_when_present() {
        let demo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/figures/demo.png");
        if !demo.is_file() {
            return;
        }
        let layout = layout();
        with_painter(|ctx, rect, painter| {
            let mut textures = HashMap::new();
            let base = demo.parent().unwrap();
            paint_shape(
                painter,
                rect,
                &layout,
                &WorldShape::Image(WorldImage {
                    path: "demo.png".into(),
                    corners_mm: [
                        (10.0, 10.0),
                        (40.0, 10.0),
                        (40.0, 40.0),
                        (10.0, 40.0),
                    ],
                    alpha: 1.0,
                }),
                &mut textures,
                ctx,
                Some(base),
            );
            assert!(textures.contains_key("demo.png"));
            // Cache hit path.
            paint_shape(
                painter,
                rect,
                &layout,
                &WorldShape::Image(WorldImage {
                    path: "demo.png".into(),
                    corners_mm: [
                        (10.0, 10.0),
                        (40.0, 10.0),
                        (40.0, 40.0),
                        (10.0, 40.0),
                    ],
                    alpha: 1.0,
                }),
                &mut textures,
                ctx,
                Some(base),
            );
        });
    }
}
