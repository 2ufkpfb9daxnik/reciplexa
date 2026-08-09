//! `rpx.std.core` — typed units for length, angle, color, geometry, and ranges.

use std::fmt;

use reciplexa_scene::Color as SceneColor;

/// Length in page millimeters (PDF user space).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Length {
    pub mm: f64,
}

impl Length {
    pub const ZERO: Self = Self { mm: 0.0 };

    pub const fn mm(mm: f64) -> Self {
        Self { mm }
    }

    pub const fn as_mm(self) -> f64 {
        self.mm
    }

    pub fn is_finite(self) -> bool {
        self.mm.is_finite()
    }

    pub fn is_non_negative(self) -> bool {
        self.is_finite() && self.mm >= 0.0
    }

    pub fn is_positive(self) -> bool {
        self.is_finite() && self.mm > 0.0
    }

    pub fn saturating_add(self, other: Self) -> Self {
        Self::mm(self.mm + other.mm)
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        Self::mm((self.mm - other.mm).max(0.0))
    }

    pub fn scale(self, factor: f64) -> Self {
        Self::mm(self.mm * factor)
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}mm", self.mm)
    }
}

/// Angle in degrees (clockwise-positive matches scene rotate helpers).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Angle {
    pub degrees: f64,
}

impl Angle {
    pub const ZERO: Self = Self { degrees: 0.0 };

    pub const fn degrees(degrees: f64) -> Self {
        Self { degrees }
    }

    pub fn radians(radians: f64) -> Self {
        Self {
            degrees: radians.to_degrees(),
        }
    }

    pub const fn as_degrees(self) -> f64 {
        self.degrees
    }

    pub fn as_radians(self) -> f64 {
        self.degrees.to_radians()
    }

    pub fn is_finite(self) -> bool {
        self.degrees.is_finite()
    }

    pub fn normalized(self) -> Self {
        if !self.is_finite() {
            return Self::ZERO;
        }
        let mut d = self.degrees % 360.0;
        if d < 0.0 {
            d += 360.0;
        }
        Self::degrees(d)
    }
}

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}deg", self.degrees)
    }
}

/// 2D point in millimeter page space (origin = bottom-left).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: Length,
    pub y: Length,
}

impl Point {
    pub const ORIGIN: Self = Self {
        x: Length::ZERO,
        y: Length::ZERO,
    };

    pub const fn new(x: Length, y: Length) -> Self {
        Self { x, y }
    }

    pub const fn mm(x_mm: f64, y_mm: f64) -> Self {
        Self {
            x: Length::mm(x_mm),
            y: Length::mm(y_mm),
        }
    }

    pub fn translate(self, dx: Length, dy: Length) -> Self {
        Self {
            x: self.x.saturating_add(dx),
            y: self.y.saturating_add(dy),
        }
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Width × height in millimeters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: Length,
    pub height: Length,
}

impl Size {
    pub const ZERO: Self = Self {
        width: Length::ZERO,
        height: Length::ZERO,
    };

    pub const fn new(width: Length, height: Length) -> Self {
        Self { width, height }
    }

    pub const fn mm(width_mm: f64, height_mm: f64) -> Self {
        Self {
            width: Length::mm(width_mm),
            height: Length::mm(height_mm),
        }
    }

    pub fn is_positive(self) -> bool {
        self.width.is_positive() && self.height.is_positive()
    }

    pub fn area_mm2(self) -> f64 {
        self.width.as_mm() * self.height.as_mm()
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}×{}", self.width, self.height)
    }
}

/// Axis-aligned rectangle: origin is the bottom-left corner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    pub const fn from_xywh(x_mm: f64, y_mm: f64, w_mm: f64, h_mm: f64) -> Self {
        Self {
            origin: Point::mm(x_mm, y_mm),
            size: Size::mm(w_mm, h_mm),
        }
    }

    pub fn width(self) -> Length {
        self.size.width
    }

    pub fn height(self) -> Length {
        self.size.height
    }

    pub fn max_x(self) -> Length {
        self.origin.x.saturating_add(self.size.width)
    }

    pub fn max_y(self) -> Length {
        self.origin.y.saturating_add(self.size.height)
    }

    pub fn center(self) -> Point {
        Point {
            x: Length::mm(self.origin.x.as_mm() + self.size.width.as_mm() * 0.5),
            y: Length::mm(self.origin.y.as_mm() + self.size.height.as_mm() * 0.5),
        }
    }

    pub fn contains_point(self, p: Point) -> bool {
        p.x.as_mm() >= self.origin.x.as_mm()
            && p.y.as_mm() >= self.origin.y.as_mm()
            && p.x.as_mm() <= self.max_x().as_mm()
            && p.y.as_mm() <= self.max_y().as_mm()
    }

    pub fn is_drawable(self) -> bool {
        self.origin.is_finite() && self.size.is_positive()
    }

    pub fn inflate(self, by: Length) -> Self {
        Self {
            origin: Point {
                x: Length::mm(self.origin.x.as_mm() - by.as_mm()),
                y: Length::mm(self.origin.y.as_mm() - by.as_mm()),
            },
            size: Size {
                width: self.size.width.saturating_add(by.scale(2.0)),
                height: self.size.height.saturating_add(by.scale(2.0)),
            },
        }
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rect{{origin={}, size={}}}", self.origin, self.size)
    }
}

/// sRGB color wrapper over [`reciplexa_scene::Color`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    inner: SceneColor,
}

impl Color {
    pub const BLACK: Self = Self {
        inner: SceneColor::BLACK,
    };
    pub const WHITE: Self = Self {
        inner: SceneColor::WHITE,
    };
    pub const RED: Self = Self {
        inner: SceneColor::RED,
    };
    pub const GREEN: Self = Self {
        inner: SceneColor::GREEN,
    };
    pub const BLUE: Self = Self {
        inner: SceneColor::BLUE,
    };

    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self {
            inner: SceneColor::new(r, g, b),
        }
    }

    pub fn named(name: &str) -> Option<Self> {
        SceneColor::named(name).map(|inner| Self { inner })
    }

    pub fn from_scene(c: SceneColor) -> Self {
        Self { inner: c }
    }

    pub fn to_scene(self) -> SceneColor {
        self.inner
    }

    pub fn channels(self) -> (f64, f64, f64) {
        (self.inner.r, self.inner.g, self.inner.b)
    }

    pub fn is_valid(self) -> bool {
        self.inner.is_channel_valid()
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, g, b) = self.channels();
        write!(f, "rgb({r},{g},{b})")
    }
}

impl From<SceneColor> for Color {
    fn from(value: SceneColor) -> Self {
        Self::from_scene(value)
    }
}

impl From<Color> for SceneColor {
    fn from(value: Color) -> Self {
        value.to_scene()
    }
}

/// Inclusive numeric / unit range used by layout and motion bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range<T> {
    pub start: T,
    pub end: T,
}

impl<T> Range<T> {
    pub const fn new(start: T, end: T) -> Self {
        Self { start, end }
    }

    pub fn map<U, F>(self, mut f: F) -> Range<U>
    where
        F: FnMut(T) -> U,
    {
        Range {
            start: f(self.start),
            end: f(self.end),
        }
    }
}

impl<T: PartialOrd> Range<T> {
    pub fn contains(&self, value: &T) -> bool {
        value >= &self.start && value <= &self.end
    }

    pub fn is_empty(&self) -> bool {
        self.start > self.end
    }
}

impl Range<Length> {
    pub fn span_mm(self) -> f64 {
        (self.end.as_mm() - self.start.as_mm()).max(0.0)
    }
}

impl<T: fmt::Display> fmt::Display for Range<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} .. {}]", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_boundaries_and_ops() {
        assert!(Length::ZERO.is_non_negative());
        assert!(!Length::ZERO.is_positive());
        assert!(Length::mm(1.0).is_positive());
        assert!(!Length::mm(f64::NAN).is_finite());
        assert!(!Length::mm(-1.0).is_non_negative());
        assert_eq!(Length::mm(3.0).saturating_add(Length::mm(2.0)).as_mm(), 5.0);
        assert_eq!(Length::mm(1.0).saturating_sub(Length::mm(3.0)).as_mm(), 0.0);
        assert_eq!(Length::mm(2.0).scale(0.5).as_mm(), 1.0);
        assert_eq!(Length::mm(4.0).to_string(), "4mm");
    }

    #[test]
    fn angle_normalize_and_radians() {
        assert_eq!(Angle::degrees(0.0).as_degrees(), 0.0);
        let a = Angle::radians(std::f64::consts::PI);
        assert!((a.as_degrees() - 180.0).abs() < 1e-9);
        assert!((Angle::degrees(180.0).as_radians() - std::f64::consts::PI).abs() < 1e-9);
        assert_eq!(Angle::degrees(370.0).normalized().as_degrees(), 10.0);
        assert_eq!(Angle::degrees(-10.0).normalized().as_degrees(), 350.0);
        assert_eq!(Angle::degrees(f64::NAN).normalized(), Angle::ZERO);
        assert_eq!(Angle::degrees(45.0).to_string(), "45deg");
    }

    #[test]
    fn point_size_rect_geometry() {
        let p = Point::mm(10.0, 20.0).translate(Length::mm(5.0), Length::mm(-5.0));
        assert_eq!(p, Point::mm(15.0, 15.0));
        assert!(Point::ORIGIN.is_finite());
        let s = Size::mm(100.0, 50.0);
        assert!(s.is_positive());
        assert!(!Size::ZERO.is_positive());
        assert_eq!(s.area_mm2(), 5000.0);
        let r = Rect::from_xywh(0.0, 0.0, 10.0, 20.0);
        assert!(r.is_drawable());
        assert!(r.contains_point(Point::mm(5.0, 10.0)));
        assert!(!r.contains_point(Point::mm(-1.0, 0.0)));
        assert_eq!(r.center(), Point::mm(5.0, 10.0));
        assert_eq!(r.max_x().as_mm(), 10.0);
        assert_eq!(r.max_y().as_mm(), 20.0);
        let grown = r.inflate(Length::mm(1.0));
        assert_eq!(grown.origin, Point::mm(-1.0, -1.0));
        assert_eq!(grown.size, Size::mm(12.0, 22.0));
        assert!(!Rect::from_xywh(0.0, 0.0, 0.0, 1.0).is_drawable());
        assert!(Point::mm(1.0, 2.0).to_string().contains("1mm"));
        assert!(Size::mm(1.0, 2.0).to_string().contains('×'));
        assert!(r.to_string().contains("Rect"));
    }

    #[test]
    fn color_wraps_scene() {
        assert!(Color::BLACK.is_valid());
        assert_eq!(Color::named("red"), Some(Color::RED));
        assert_eq!(Color::named("nope"), None);
        assert!(!Color::new(2.0, 0.0, 0.0).is_valid());
        let scene = Color::BLUE.to_scene();
        assert_eq!(Color::from_scene(scene), Color::BLUE);
        assert_eq!(SceneColor::from(Color::GREEN), SceneColor::GREEN);
        assert_eq!(Color::from(SceneColor::WHITE), Color::WHITE);
        assert_eq!(Color::BLACK.channels(), (0.0, 0.0, 0.0));
        assert!(Color::RED.to_string().contains("rgb"));
    }

    #[test]
    fn range_contains_and_span() {
        let r = Range::new(Length::mm(1.0), Length::mm(5.0));
        assert!(r.contains(&Length::mm(3.0)));
        assert!(!r.contains(&Length::mm(0.0)));
        assert!(!r.is_empty());
        assert!(Range::new(5, 1).is_empty());
        assert_eq!(r.span_mm(), 4.0);
        let mapped = r.map(|l| l.as_mm() as i32);
        assert_eq!(mapped, Range::new(1, 5));
        assert!(r.to_string().contains(".."));
    }
}
