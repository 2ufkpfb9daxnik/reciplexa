//! `rpx.std.vector` — path segments, stroke details, gradient stubs.

use std::fmt;

use reciplexa_identity::document::StableNodeId;

use crate::core::{Color, Length, Point};
use crate::style::Stroke;

/// Path construction command.
#[derive(Debug, Clone, PartialEq)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    QuadTo { control: Point, to: Point },
    CubicTo {
        c1: Point,
        c2: Point,
        to: Point,
    },
    Close,
}

impl PathCommand {
    pub fn is_close(&self) -> bool {
        matches!(self, Self::Close)
    }
}

/// Editable vector path with StableNodeId.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorPath {
    pub id: StableNodeId,
    pub commands: Vec<PathCommand>,
}

impl VectorPath {
    pub fn new(id: StableNodeId) -> Self {
        Self {
            id,
            commands: Vec::new(),
        }
    }

    pub fn move_to(mut self, p: Point) -> Self {
        self.commands.push(PathCommand::MoveTo(p));
        self
    }

    pub fn line_to(mut self, p: Point) -> Self {
        self.commands.push(PathCommand::LineTo(p));
        self
    }

    pub fn quad_to(mut self, control: Point, to: Point) -> Self {
        self.commands.push(PathCommand::QuadTo { control, to });
        self
    }

    pub fn cubic_to(mut self, c1: Point, c2: Point, to: Point) -> Self {
        self.commands.push(PathCommand::CubicTo { c1, c2, to });
        self
    }

    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::Close);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn endpoint_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|c| !matches!(c, PathCommand::Close))
            .count()
    }
}

/// Stroke join style stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeJoin {
    Miter,
    Round,
    Bevel,
}

/// Stroke cap style stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeCap {
    Butt,
    Round,
    Square,
}

/// Extended stroke with join/cap/dash.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorStroke {
    pub base: Stroke,
    pub join: StrokeJoin,
    pub cap: StrokeCap,
    pub dash: Vec<Length>,
}

impl VectorStroke {
    pub fn solid(base: Stroke) -> Self {
        Self {
            base,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            dash: Vec::new(),
        }
    }

    pub fn with_dash(mut self, dash: Vec<Length>) -> Self {
        self.dash = dash;
        self
    }

    pub fn is_dashed(&self) -> bool {
        !self.dash.is_empty()
    }
}

/// Gradient stop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    pub offset: f64,
    pub color: Color,
}

impl GradientStop {
    pub fn new(offset: f64, color: Color) -> Option<Self> {
        if !(0.0..=1.0).contains(&offset) || !color.is_valid() {
            return None;
        }
        Some(Self { offset, color })
    }
}

/// Gradient stub (linear or radial).
#[derive(Debug, Clone, PartialEq)]
pub enum Gradient {
    Linear {
        id: StableNodeId,
        start: Point,
        end: Point,
        stops: Vec<GradientStop>,
    },
    Radial {
        id: StableNodeId,
        center: Point,
        radius: Length,
        stops: Vec<GradientStop>,
    },
}

impl Gradient {
    pub fn id(&self) -> StableNodeId {
        match self {
            Self::Linear { id, .. } | Self::Radial { id, .. } => *id,
        }
    }

    pub fn stop_count(&self) -> usize {
        match self {
            Self::Linear { stops, .. } | Self::Radial { stops, .. } => stops.len(),
        }
    }

    pub fn is_usable(&self) -> bool {
        self.stop_count() >= 2
    }
}

/// Boolean path operation stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    Union,
    Intersection,
    Difference,
    Xor,
}

impl BooleanOp {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Union => "union",
            Self::Intersection => "intersection",
            Self::Difference => "difference",
            Self::Xor => "xor",
        }
    }
}

impl fmt::Display for BooleanOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Declared boolean combine of two path ids (evaluation deferred).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BooleanPathOp {
    pub id: StableNodeId,
    pub op: BooleanOp,
    pub a: StableNodeId,
    pub b: StableNodeId,
}

impl BooleanPathOp {
    pub fn new(id: StableNodeId, op: BooleanOp, a: StableNodeId, b: StableNodeId) -> Self {
        Self { id, op, a, b }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Stroke;

    fn id(n: u64) -> StableNodeId {
        StableNodeId::new(n)
    }

    #[test]
    fn path_commands_and_stroke() {
        assert!(PathCommand::Close.is_close());
        assert!(!PathCommand::MoveTo(Point::ORIGIN).is_close());
        let path = VectorPath::new(id(1))
            .move_to(Point::mm(0.0, 0.0))
            .line_to(Point::mm(1.0, 0.0))
            .quad_to(Point::mm(1.0, 1.0), Point::mm(0.0, 1.0))
            .cubic_to(
                Point::mm(0.0, 0.5),
                Point::mm(-0.5, 0.5),
                Point::mm(0.0, 0.0),
            )
            .close();
        assert!(!path.is_empty());
        assert_eq!(path.endpoint_count(), 4);
        assert!(VectorPath::new(id(1)).is_empty());
        let stroke = VectorStroke::solid(Stroke::new(Color::BLACK, Length::mm(1.0)))
            .with_dash(vec![Length::mm(2.0), Length::mm(1.0)]);
        assert!(stroke.is_dashed());
        assert!(!VectorStroke::solid(Stroke::new(Color::BLACK, Length::mm(1.0))).is_dashed());
        let _ = StrokeJoin::Round;
        let _ = StrokeCap::Square;
        let _ = StrokeJoin::Bevel;
        let _ = StrokeCap::Round;
    }

    #[test]
    fn gradient_and_boolean() {
        assert!(GradientStop::new(-0.1, Color::BLACK).is_none());
        assert!(GradientStop::new(1.1, Color::BLACK).is_none());
        assert!(GradientStop::new(0.5, Color::new(2.0, 0.0, 0.0)).is_none());
        let s0 = GradientStop::new(0.0, Color::BLACK).unwrap();
        let s1 = GradientStop::new(1.0, Color::WHITE).unwrap();
        let lin = Gradient::Linear {
            id: id(1),
            start: Point::ORIGIN,
            end: Point::mm(10.0, 0.0),
            stops: vec![s0, s1],
        };
        assert!(lin.is_usable());
        assert_eq!(lin.id(), id(1));
        let rad = Gradient::Radial {
            id: id(2),
            center: Point::ORIGIN,
            radius: Length::mm(5.0),
            stops: vec![s0],
        };
        assert!(!rad.is_usable());
        assert_eq!(rad.stop_count(), 1);
        assert_eq!(BooleanOp::Union.as_str(), "union");
        assert_eq!(BooleanOp::Intersection.to_string(), "intersection");
        assert_eq!(BooleanOp::Difference.as_str(), "difference");
        assert_eq!(BooleanOp::Xor.as_str(), "xor");
        let op = BooleanPathOp::new(id(3), BooleanOp::Xor, id(1), id(2));
        assert_eq!(op.op, BooleanOp::Xor);
    }
}
