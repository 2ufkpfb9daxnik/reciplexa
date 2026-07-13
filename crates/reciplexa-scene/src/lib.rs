//! Pure scene data: paper and shapes with no rendering backend.
//!
//! Keeping geometry here lets PDF, future SVG, and GUI hit-testing share one
//! model and be unit-tested without windowing or file I/O.

#![forbid(unsafe_code)]

/// sRGB color channels in `0.0 ..= 1.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };

    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    pub fn is_channel_valid(self) -> bool {
        [self.r, self.g, self.b]
            .into_iter()
            .all(|c| (0.0..=1.0).contains(&c))
    }
}

/// ISO A4 portrait size in millimeters.
pub const A4_WIDTH_MM: f64 = 210.0;
pub const A4_HEIGHT_MM: f64 = 297.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaperSize {
    pub width_mm: f64,
    pub height_mm: f64,
}

impl PaperSize {
    pub const fn a4() -> Self {
        Self {
            width_mm: A4_WIDTH_MM,
            height_mm: A4_HEIGHT_MM,
        }
    }

    pub fn is_positive(self) -> bool {
        self.width_mm > 0.0 && self.height_mm > 0.0
    }
}

/// Axis-aligned filled circle. Position is the center in page millimeters.
///
/// Coordinate origin is the **bottom-left** of the page (PDF space), so GUI
/// and PDF backends share one convention from day one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub x_mm: f64,
    pub y_mm: f64,
    pub radius_mm: f64,
    pub fill: Color,
}

impl Circle {
    pub fn is_drawable(self) -> bool {
        self.radius_mm > 0.0 && self.fill.is_channel_valid()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Circle(Circle),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub paper: PaperSize,
    pub shapes: Vec<Shape>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    pub pages: Vec<Page>,
}

impl Document {
    pub fn single_page(page: Page) -> Self {
        Self { pages: vec![page] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validity ---

    #[test]
    fn a4_constants_match_iso() {
        let a4 = PaperSize::a4();
        assert_eq!(a4.width_mm, 210.0);
        assert_eq!(a4.height_mm, 297.0);
        assert!(a4.is_positive());
    }

    #[test]
    fn black_circle_on_a4_is_drawable() {
        let c = Circle {
            x_mm: 105.0,
            y_mm: 148.5,
            radius_mm: 40.0,
            fill: Color::BLACK,
        };
        assert!(c.is_drawable());
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Circle(c)],
        });
        assert_eq!(doc.pages.len(), 1);
    }

    // --- defect ---

    #[test]
    fn zero_or_negative_radius_is_not_drawable() {
        let mut c = Circle {
            x_mm: 0.0,
            y_mm: 0.0,
            radius_mm: 0.0,
            fill: Color::BLACK,
        };
        assert!(!c.is_drawable());
        c.radius_mm = -1.0;
        assert!(!c.is_drawable());
    }

    #[test]
    fn out_of_range_color_channel_is_invalid() {
        assert!(!Color::new(1.1, 0.0, 0.0).is_channel_valid());
        assert!(!Color::new(0.0, -0.01, 0.0).is_channel_valid());
    }

    #[test]
    fn non_positive_paper_rejected() {
        assert!(!PaperSize {
            width_mm: 0.0,
            height_mm: 10.0
        }
        .is_positive());
    }
}
