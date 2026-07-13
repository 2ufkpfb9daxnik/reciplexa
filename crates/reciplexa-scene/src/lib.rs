//! Pure scene data: paper and shapes with no rendering backend.
//!
//! Keeping geometry here lets PDF, future SVG, and GUI hit-testing share one
//! model and be unit-tested without windowing or file I/O.

#![forbid(unsafe_code)]

mod affine;

pub use affine::Affine;

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
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
    };

    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    pub fn named(name: &str) -> Option<Self> {
        Some(match name {
            "black" => Self::BLACK,
            "white" => Self::WHITE,
            "red" => Self::RED,
            "green" => Self::GREEN,
            "blue" => Self::BLUE,
            _ => return None,
        })
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

/// Axis-aligned filled circle in **local** millimeters (before parent transforms).
///
/// Page origin is the **bottom-left** (PDF space).
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

/// Axis-aligned filled rectangle in **local** millimeters (before transforms).
///
/// `(x_mm, y_mm)` is the **bottom-left** corner in local space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x_mm: f64,
    pub y_mm: f64,
    pub width_mm: f64,
    pub height_mm: f64,
    pub fill: Color,
}

impl Rect {
    pub fn is_drawable(self) -> bool {
        self.width_mm > 0.0 && self.height_mm > 0.0 && self.fill.is_channel_valid()
    }
}

/// Axis-aligned filled ellipse in **local** millimeters.
///
/// `(x_mm, y_mm)` is the center; `rx_mm` / `ry_mm` are radii.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse {
    pub x_mm: f64,
    pub y_mm: f64,
    pub rx_mm: f64,
    pub ry_mm: f64,
    pub fill: Color,
}

impl Ellipse {
    pub fn is_drawable(self) -> bool {
        self.rx_mm > 0.0 && self.ry_mm > 0.0 && self.fill.is_channel_valid()
    }
}

/// Filled text baseline position in local millimeters.
///
/// `size_mm` is the em-box height used for PDF `Tf` (converted to points).
/// Content is currently limited to PDF WinAnsi/ASCII at the emitter.
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub x_mm: f64,
    pub y_mm: f64,
    pub size_mm: f64,
    pub content: String,
    pub fill: Color,
}

impl Text {
    pub fn is_drawable(&self) -> bool {
        self.size_mm > 0.0 && !self.content.is_empty() && self.fill.is_channel_valid()
    }
}

/// Stroked line segment in local millimeters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub x1_mm: f64,
    pub y1_mm: f64,
    pub x2_mm: f64,
    pub y2_mm: f64,
    pub stroke: Color,
    pub width_mm: f64,
}

impl Line {
    pub fn is_drawable(self) -> bool {
        self.width_mm > 0.0
            && self.stroke.is_channel_valid()
            && (self.x1_mm != self.x2_mm || self.y1_mm != self.y2_mm)
    }
}

/// Drawable node. [`Shape::Group`] applies an affine to nested children—
/// the Glisp-style stack of translate / rotate / scale.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Circle(Circle),
    Rect(Rect),
    Ellipse(Ellipse),
    Text(Text),
    Line(Line),
    Group {
        transform: Affine,
        children: Vec<Shape>,
    },
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

    #[test]
    fn named_colors_resolve() {
        assert_eq!(Color::named("red"), Some(Color::RED));
        assert_eq!(Color::named("black"), Some(Color::BLACK));
    }

    #[test]
    fn transformed_group_holds_children() {
        let g = Shape::Group {
            transform: Affine::translate(10.0, 20.0)
                .then(Affine::rotate_deg(15.0))
                .then(Affine::scale_uniform(2.0)),
            children: vec![Shape::Circle(Circle {
                x_mm: 0.0,
                y_mm: 0.0,
                radius_mm: 5.0,
                fill: Color::BLUE,
            })],
        };
        match g {
            Shape::Group { children, .. } => assert_eq!(children.len(), 1),
            _ => panic!("expected group"),
        }
    }

    #[test]
    fn rect_requires_positive_size() {
        let ok = Rect {
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 10.0,
            height_mm: 5.0,
            fill: Color::BLACK,
        };
        assert!(ok.is_drawable());
        let bad = Rect {
            height_mm: 0.0,
            ..ok
        };
        assert!(!bad.is_drawable());
    }

    #[test]
    fn text_and_line_drawable_rules() {
        let t = Text {
            x_mm: 0.0,
            y_mm: 0.0,
            size_mm: 4.0,
            content: "a".into(),
            fill: Color::BLACK,
        };
        assert!(t.is_drawable());
        assert!(!Text {
            content: String::new(),
            ..t.clone()
        }
        .is_drawable());
        let l = Line {
            x1_mm: 0.0,
            y1_mm: 0.0,
            x2_mm: 1.0,
            y2_mm: 0.0,
            stroke: Color::BLACK,
            width_mm: 0.5,
        };
        assert!(l.is_drawable());
        assert!(!Line { x2_mm: 0.0, ..l }.is_drawable());
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

    #[test]
    fn unknown_named_color_is_none() {
        assert_eq!(Color::named("puce"), None);
        assert_eq!(Color::named(""), None);
    }
}
