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
    CubicTo { c1: Point, c2: Point, to: Point },
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
