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
    pub fill: Color,
}

/// Convex polygon in page millimeters (rect corners after affine).
#[derive(Debug, Clone, PartialEq)]
pub struct WorldPolygon {
    pub points_mm: Vec<(f64, f64)>,
    pub fill: Color,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorldShape {
    Circle(WorldCircle),
    Polygon(WorldPolygon),
}

/// Flatten a document's first page (preview shows page 0).
pub fn flatten_first_page(doc: &Document) -> Option<(&Page, Vec<WorldShape>)> {
    let page = doc.pages.first()?;
    Some((page, flatten_shapes(&page.shapes, Affine::identity())))
}

pub fn flatten_shapes(shapes: &[Shape], parent: Affine) -> Vec<WorldShape> {
    let mut out = Vec::new();
    for shape in shapes {
        flatten_shape(shape, parent, &mut out);
    }
    out
}

fn flatten_shape(shape: &Shape, parent: Affine, out: &mut Vec<WorldShape>) {
    match shape {
        Shape::Circle(c) => {
            let (x, y) = parent.transform_point(c.x_mm, c.y_mm);
            let scale = linear_scale(parent);
            out.push(WorldShape::Circle(WorldCircle {
                x_mm: x,
                y_mm: y,
                radius_mm: c.radius_mm * scale,
                fill: c.fill,
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
                fill: r.fill,
            }));
        }
        Shape::Text(t) => {
            let (x, y) = parent.transform_point(t.x_mm, t.y_mm);
            let scale = linear_scale(parent);
            let w = t.content.len() as f64 * t.size_mm * 0.5 * scale;
            let h = t.size_mm * scale;
            out.push(WorldShape::Polygon(WorldPolygon {
                points_mm: vec![
                    (x, y),
                    (x + w, y),
                    (x + w, y + h),
                    (x, y + h),
                ],
                fill: t.fill,
            }));
        }
        Shape::Line(l) => {
            let (x1, y1) = parent.transform_point(l.x1_mm, l.y1_mm);
            let (x2, y2) = parent.transform_point(l.x2_mm, l.y2_mm);
            // Fat segment as a thin quad for hit-testing / preview stroke proxy.
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx * dx + dy * dy).sqrt().max(1e-9);
            let px = -dy / len * (l.width_mm.max(0.5) * 0.5);
            let py = dx / len * (l.width_mm.max(0.5) * 0.5);
            out.push(WorldShape::Polygon(WorldPolygon {
                points_mm: vec![
                    (x1 + px, y1 + py),
                    (x2 + px, y2 + py),
                    (x2 - px, y2 - py),
                    (x1 - px, y1 - py),
                ],
                fill: l.stroke,
            }));
        }
        Shape::Group {
            transform,
            children,
        } => {
            let combined = transform.then(parent);
            for child in children {
                flatten_shape(child, combined, out);
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

fn shape_contains(shape: &WorldShape, x: f64, y: f64) -> bool {
    match shape {
        WorldShape::Circle(c) => {
            let dx = x - c.x_mm;
            let dy = y - c.y_mm;
            dx * dx + dy * dy <= c.radius_mm * c.radius_mm
        }
        WorldShape::Polygon(p) => point_in_polygon(x, y, &p.points_mm),
    }
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
    use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Rect, Shape};

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
        match &shapes[0] {
            WorldShape::Circle(c) => {
                assert_eq!(c.x_mm, 105.0);
                assert_eq!(c.radius_mm, 40.0);
            }
            _ => panic!("expected circle"),
        }
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
        match &shapes[0] {
            WorldShape::Polygon(p) => {
                assert_eq!(p.points_mm.len(), 4);
                assert_eq!(p.points_mm[2], (10.0, 5.0));
            }
            _ => panic!("expected polygon"),
        }
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
        match &out[0] {
            WorldShape::Circle(c) => {
                assert!((c.x_mm - 12.0).abs() < 1e-9);
                assert!((c.y_mm - 22.0).abs() < 1e-9);
                assert!((c.radius_mm - 10.0).abs() < 1e-9);
            }
            _ => panic!("expected circle"),
        }
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
                fill: Color::BLACK,
            }),
            WorldShape::Circle(WorldCircle {
                x_mm: 0.0,
                y_mm: 0.0,
                radius_mm: 5.0,
                fill: Color::RED,
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
        match &out[0] {
            WorldShape::Circle(c) => assert_eq!(c.radius_mm, 0.0),
            _ => panic!("expected circle"),
        }
    }
}
