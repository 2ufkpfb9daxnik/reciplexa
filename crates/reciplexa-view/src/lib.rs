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

/// Flatten a document's first page (M6 preview shows page 0).
pub fn flatten_first_page(doc: &Document) -> Option<(&Page, Vec<WorldCircle>)> {
    let page = doc.pages.first()?;
    Some((page, flatten_shapes(&page.shapes, Affine::identity())))
}

pub fn flatten_shapes(shapes: &[Shape], parent: Affine) -> Vec<WorldCircle> {
    let mut out = Vec::new();
    for shape in shapes {
        flatten_shape(shape, parent, &mut out);
    }
    out
}

fn flatten_shape(shape: &Shape, parent: Affine, out: &mut Vec<WorldCircle>) {
    match shape {
        Shape::Circle(c) => {
            let (x, y) = parent.transform_point(c.x_mm, c.y_mm);
            let scale = linear_scale(parent);
            out.push(WorldCircle {
                x_mm: x,
                y_mm: y,
                radius_mm: c.radius_mm * scale,
                fill: c.fill,
            });
        }
        Shape::Group {
            transform,
            children,
        } => {
            // Child local → group → parent  ⇒  parent.then(*transform)?
            // Point p_local; group applies T; parent applies P.
            // p_world = P * T * p_local = (P.then(T))?
            // Our `then` is: self.then(next) means apply self first then next.
            // So T.then(P) would be wrong; we want apply T then P: T.then(P).
            // parent is already P. Combined = transform.then(parent).
            let combined = transform.then(parent);
            for child in children {
                flatten_shape(child, combined, out);
            }
        }
    }
}

/// Uniform scale factor induced by the linear part (√|det|).
/// Circles stay circles under rotation+uniform scale; non-uniform scale is
/// approximated (good enough for M6 preview).
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
    /// Fit the page into `avail` with margin, preserving aspect ratio and centering.
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
        // flip Y: page bottom-left → screen top-left
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

/// Hit-test circles in page mm; returns the topmost (last drawn) match.
pub fn hit_test_circles(circles: &[WorldCircle], x_mm: f64, y_mm: f64) -> Option<usize> {
    circles.iter().enumerate().rev().find_map(|(i, c)| {
        let dx = x_mm - c.x_mm;
        let dy = y_mm - c.y_mm;
        ((dx * dx + dy * dy) <= c.radius_mm * c.radius_mm).then_some(i)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Shape};

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
        let (_, circles) = flatten_first_page(&doc).unwrap();
        assert_eq!(circles.len(), 1);
        assert_eq!(circles[0].x_mm, 105.0);
        assert_eq!(circles[0].radius_mm, 40.0);
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
        let circles = flatten_shapes(&shapes, Affine::identity());
        assert_eq!(circles.len(), 1);
        // scale then translate: (1,1)*2 + (10,20) = (12,22); r=10
        assert!((circles[0].x_mm - 12.0).abs() < 1e-9);
        assert!((circles[0].y_mm - 22.0).abs() < 1e-9);
        assert!((circles[0].radius_mm - 10.0).abs() < 1e-9);
    }

    #[test]
    fn paper_layout_flips_y_and_maps_corners() {
        let layout = PaperLayout::fit(210.0, 297.0, 0.0, 210.0, 297.0);
        let (x0, y0) = layout.mm_to_px(0.0, 297.0); // top-left of page
        assert!((x0 - 0.0).abs() < 1e-3);
        assert!((y0 - 0.0).abs() < 1e-3);
        let (x1, y1) = layout.mm_to_px(0.0, 0.0); // bottom-left
        assert!((y1 - 297.0).abs() < 1e-2);
        assert!((x1 - 0.0).abs() < 1e-3);
    }

    // --- defect ---

    #[test]
    fn empty_document_has_no_first_page() {
        assert!(flatten_first_page(&Document::default()).is_none());
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
        let circles = vec![
            WorldCircle {
                x_mm: 0.0,
                y_mm: 0.0,
                radius_mm: 10.0,
                fill: Color::BLACK,
            },
            WorldCircle {
                x_mm: 0.0,
                y_mm: 0.0,
                radius_mm: 5.0,
                fill: Color::RED,
            },
        ];
        assert_eq!(hit_test_circles(&circles, 0.0, 0.0), Some(1));
        assert_eq!(hit_test_circles(&circles, 8.0, 0.0), Some(0));
        assert_eq!(hit_test_circles(&circles, 20.0, 0.0), None);
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
        let circles = flatten_shapes(&shapes, Affine::identity());
        assert_eq!(circles[0].radius_mm, 0.0);
    }
}
