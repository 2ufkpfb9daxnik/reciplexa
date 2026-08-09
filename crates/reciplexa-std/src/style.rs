//! `rpx.std.style` — fill, stroke, and opacity primitives.

use std::fmt;

use crate::core::{Color, Length};

/// Solid or absent paint for filled geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill {
    None,
    Solid(Color),
}

impl Fill {
    pub const fn solid(color: Color) -> Self {
        Self::Solid(color)
    }

    pub fn color(self) -> Option<Color> {
        match self {
            Self::None => None,
            Self::Solid(c) => Some(c),
        }
    }

    pub fn is_visible(self) -> bool {
        match self {
            Self::None => false,
            Self::Solid(c) => c.is_valid(),
        }
    }
}

impl fmt::Display for Fill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "fill:none"),
            Self::Solid(c) => write!(f, "fill:{c}"),
        }
    }
}

/// Stroked outline: color + width in millimeters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    pub color: Color,
    pub width: Length,
}

impl Stroke {
    pub const fn new(color: Color, width: Length) -> Self {
        Self { color, width }
    }

    pub fn is_drawable(self) -> bool {
        self.color.is_valid() && self.width.is_positive()
    }
}

impl fmt::Display for Stroke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stroke:{} width={}", self.color, self.width)
    }
}

/// Multiplicative opacity in `0.0 ..= 1.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Opacity {
    pub alpha: f64,
}

impl Opacity {
    pub const OPAQUE: Self = Self { alpha: 1.0 };
    pub const TRANSPARENT: Self = Self { alpha: 0.0 };

    pub const fn new(alpha: f64) -> Self {
        Self { alpha }
    }

    pub fn is_valid(self) -> bool {
        (0.0..=1.0).contains(&self.alpha)
    }

    pub fn is_fully_transparent(self) -> bool {
        self.is_valid() && self.alpha == 0.0
    }

    pub fn is_fully_opaque(self) -> bool {
        self.is_valid() && self.alpha == 1.0
    }

    pub fn clamp(self) -> Self {
        Self {
            alpha: self.alpha.clamp(0.0, 1.0),
        }
    }

    pub fn multiply(self, other: Self) -> Self {
        Self {
            alpha: (self.alpha * other.alpha).clamp(0.0, 1.0),
        }
    }
}

impl Default for Opacity {
    fn default() -> Self {
        Self::OPAQUE
    }
}

impl fmt::Display for Opacity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "opacity:{}", self.alpha)
    }
}

/// Combined paint style for visual nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub fill: Fill,
    pub stroke: Option<Stroke>,
    pub opacity: Opacity,
}

impl Style {
    pub const fn filled(color: Color) -> Self {
        Self {
            fill: Fill::Solid(color),
            stroke: None,
            opacity: Opacity::OPAQUE,
        }
    }

    pub const fn stroked(stroke: Stroke) -> Self {
        Self {
            fill: Fill::None,
            stroke: Some(stroke),
            opacity: Opacity::OPAQUE,
        }
    }

    pub fn with_opacity(mut self, opacity: Opacity) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn is_drawable(self) -> bool {
        if !self.opacity.is_valid() || self.opacity.is_fully_transparent() {
            return false;
        }
        let fill_ok = self.fill.is_visible();
        let stroke_ok = self.stroke.map(|s| s.is_drawable()).unwrap_or(false);
        fill_ok || stroke_ok
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::filled(Color::BLACK)
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = format!("Style{{{}, opacity={}}}", self.fill, self.opacity.alpha);
        if let Some(stroke) = self.stroke {
            s.push(' ');
            s.push_str(&stroke.to_string());
        }
        f.write_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_variants() {
        assert!(!Fill::None.is_visible());
        assert_eq!(Fill::None.color(), None);
        assert!(Fill::solid(Color::RED).is_visible());
        assert_eq!(Fill::solid(Color::RED).color(), Some(Color::RED));
        assert!(!Fill::solid(Color::new(2.0, 0.0, 0.0)).is_visible());
        assert_eq!(Fill::None.to_string(), "fill:none");
        assert!(Fill::solid(Color::BLACK).to_string().contains("fill:"));
    }

    #[test]
    fn stroke_and_opacity() {
        let s = Stroke::new(Color::BLACK, Length::mm(1.0));
        assert!(s.is_drawable());
        assert!(!Stroke::new(Color::BLACK, Length::ZERO).is_drawable());
        assert!(!Stroke::new(Color::new(2.0, 0.0, 0.0), Length::mm(1.0)).is_drawable());
        assert!(s.to_string().contains("stroke:"));
        assert!(Opacity::OPAQUE.is_fully_opaque());
        assert!(Opacity::TRANSPARENT.is_fully_transparent());
        assert!(!Opacity::new(-0.1).is_valid());
        assert!(!Opacity::new(1.1).is_valid());
        assert_eq!(Opacity::new(2.0).clamp().alpha, 1.0);
        assert_eq!(Opacity::new(0.5).multiply(Opacity::new(0.5)).alpha, 0.25);
        assert_eq!(Opacity::default(), Opacity::OPAQUE);
        assert!(Opacity::new(0.25).to_string().contains("0.25"));
    }

    #[test]
    fn style_drawable_partitions() {
        let filled = Style::filled(Color::BLUE);
        assert!(filled.is_drawable());
        assert!(Style::default().is_drawable());
        let stroked = Style::stroked(Stroke::new(Color::BLACK, Length::mm(0.5)));
        assert!(stroked.is_drawable());
        assert!(!Style {
            fill: Fill::None,
            stroke: None,
            opacity: Opacity::OPAQUE,
        }
        .is_drawable());
        assert!(!filled.with_opacity(Opacity::TRANSPARENT).is_drawable());
        assert!(!filled.with_opacity(Opacity::new(2.0)).is_drawable());
        let both = filled.with_stroke(Stroke::new(Color::RED, Length::mm(1.0)));
        assert!(both.is_drawable());
        assert!(both.to_string().contains("Style"));
    }
}
