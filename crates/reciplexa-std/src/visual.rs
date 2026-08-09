//! `rpx.std.visual` — canvas, groups, and basic shapes with scene bridge.

use std::fmt;

use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::{
    Affine, Circle as SceneCircle, Ellipse as SceneEllipse, Frame, Image as SceneImage,
    Line as SceneLine, PaperSize, Polygon, Polyline, Rect as SceneRect, Ring, Shape, Text as SceneText,
};

use crate::core::{Angle, Color, Length, Point, Rect, Size};
use crate::style::{Fill, Opacity, Stroke, Style};

/// 2D transform: translate / rotate / scale composed into a scene [`Affine`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translate: Point,
    pub rotate: Angle,
    pub scale_x: f64,
    pub scale_y: f64,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translate: Point::ORIGIN,
        rotate: Angle::ZERO,
        scale_x: 1.0,
        scale_y: 1.0,
    };

    pub const fn translate(x: Length, y: Length) -> Self {
        Self {
            translate: Point::new(x, y),
            rotate: Angle::ZERO,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    pub fn to_affine(self) -> Affine {
        Affine::translate(self.translate.x.as_mm(), self.translate.y.as_mm())
            .then(Affine::rotate_deg(self.rotate.as_degrees()))
            .then(Affine::scale(self.scale_x, self.scale_y))
    }

    pub fn is_identity(self) -> bool {
        self == Self::IDENTITY
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "T(t={}, r={}, sx={}, sy={})",
            self.translate, self.rotate, self.scale_x, self.scale_y
        )
    }
}

/// Drawing surface with paper size and child nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    pub id: StableNodeId,
    pub paper: PaperSize,
    pub children: Vec<VisualNode>,
}

impl Canvas {
    pub fn a4(id: StableNodeId) -> Self {
        Self {
            id,
            paper: PaperSize::a4(),
            children: Vec::new(),
        }
    }

    pub fn push(&mut self, node: VisualNode) {
        self.children.push(node);
    }

    pub fn to_scene_shapes(&self) -> Vec<Shape> {
        self.children.iter().filter_map(|c| c.to_scene_shape()).collect()
    }
}

/// Editable visual tree node.
#[derive(Debug, Clone, PartialEq)]
pub enum VisualNode {
    Group(Group),
    Rectangle(Rectangle),
    Ellipse(Ellipse),
    Line(Line),
    Path(Path),
    Image(Image),
}

impl VisualNode {
    pub fn id(&self) -> StableNodeId {
        match self {
            Self::Group(n) => n.id,
            Self::Rectangle(n) => n.id,
            Self::Ellipse(n) => n.id,
            Self::Line(n) => n.id,
            Self::Path(n) => n.id,
            Self::Image(n) => n.id,
        }
    }

    pub fn to_scene_shape(&self) -> Option<Shape> {
        match self {
            Self::Group(n) => n.to_scene_shape(),
            Self::Rectangle(n) => n.to_scene_shape(),
            Self::Ellipse(n) => n.to_scene_shape(),
            Self::Line(n) => n.to_scene_shape(),
            Self::Path(n) => n.to_scene_shape(),
            Self::Image(n) => n.to_scene_shape(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub id: StableNodeId,
    pub transform: Transform,
    pub opacity: Opacity,
    pub children: Vec<VisualNode>,
}

impl Group {
    pub fn new(id: StableNodeId) -> Self {
        Self {
            id,
            transform: Transform::IDENTITY,
            opacity: Opacity::OPAQUE,
            children: Vec::new(),
        }
    }

    pub fn to_scene_shape(&self) -> Option<Shape> {
        if !self.opacity.is_valid() || self.opacity.is_fully_transparent() {
            return None;
        }
        let children: Vec<Shape> = self
            .children
            .iter()
            .filter_map(|c| c.to_scene_shape())
            .collect();
        let group = Shape::Group {
            transform: self.transform.to_affine(),
            children,
        };
        if self.opacity.is_fully_opaque() {
            Some(group)
        } else {
            Some(Shape::Opacity {
                alpha: self.opacity.alpha,
                children: vec![group],
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rectangle {
    pub id: StableNodeId,
    pub frame: Rect,
    pub style: Style,
}

impl Rectangle {
    pub fn new(id: StableNodeId, frame: Rect, style: Style) -> Self {
        Self { id, frame, style }
    }

    pub fn to_scene_shape(&self) -> Option<Shape> {
        if !self.frame.is_drawable() || !self.style.is_drawable() {
            return None;
        }
        match (&self.style.fill, self.style.stroke) {
            (Fill::Solid(c), None) => Some(Shape::Rect(SceneRect {
                x_mm: self.frame.origin.x.as_mm(),
                y_mm: self.frame.origin.y.as_mm(),
                width_mm: self.frame.width().as_mm(),
                height_mm: self.frame.height().as_mm(),
                fill: c.to_scene(),
            })),
            (Fill::None, Some(stroke)) if stroke.is_drawable() => Some(Shape::Frame(Frame {
                x_mm: self.frame.origin.x.as_mm(),
                y_mm: self.frame.origin.y.as_mm(),
                width_mm: self.frame.width().as_mm(),
                height_mm: self.frame.height().as_mm(),
                stroke_width_mm: stroke.width.as_mm(),
                stroke: stroke.color.to_scene(),
            })),
            (Fill::Solid(c), Some(stroke)) if stroke.is_drawable() => {
                // Prefer filled rect; stroke is retained in std Style for GUI/codec.
                let _ = stroke;
                Some(Shape::Rect(SceneRect {
                    x_mm: self.frame.origin.x.as_mm(),
                    y_mm: self.frame.origin.y.as_mm(),
                    width_mm: self.frame.width().as_mm(),
                    height_mm: self.frame.height().as_mm(),
                    fill: c.to_scene(),
                }))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ellipse {
    pub id: StableNodeId,
    pub center: Point,
    pub radii: Size,
    pub style: Style,
}

impl Ellipse {
    pub fn new(id: StableNodeId, center: Point, radii: Size, style: Style) -> Self {
        Self {
            id,
            center,
            radii,
            style,
        }
    }

    pub fn is_circle(&self) -> bool {
        (self.radii.width.as_mm() - self.radii.height.as_mm()).abs() < f64::EPSILON
    }

    pub fn to_scene_shape(&self) -> Option<Shape> {
        if !self.radii.is_positive() || !self.style.is_drawable() {
            return None;
        }
        match (&self.style.fill, self.style.stroke) {
            (Fill::Solid(c), _) if self.is_circle() => Some(Shape::Circle(SceneCircle {
                x_mm: self.center.x.as_mm(),
                y_mm: self.center.y.as_mm(),
                radius_mm: self.radii.width.as_mm(),
                fill: c.to_scene(),
            })),
            (Fill::Solid(c), _) => Some(Shape::Ellipse(SceneEllipse {
                x_mm: self.center.x.as_mm(),
                y_mm: self.center.y.as_mm(),
                rx_mm: self.radii.width.as_mm(),
                ry_mm: self.radii.height.as_mm(),
                fill: c.to_scene(),
            })),
            (Fill::None, Some(stroke)) if stroke.is_drawable() && self.is_circle() => {
                Some(Shape::Ring(Ring {
                    x_mm: self.center.x.as_mm(),
                    y_mm: self.center.y.as_mm(),
                    radius_mm: self.radii.width.as_mm(),
                    width_mm: stroke.width.as_mm(),
                    stroke: stroke.color.to_scene(),
                }))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub id: StableNodeId,
    pub start: Point,
    pub end: Point,
    pub stroke: Stroke,
}

impl Line {
    pub fn new(id: StableNodeId, start: Point, end: Point, stroke: Stroke) -> Self {
        Self {
            id,
            start,
            end,
            stroke,
        }
    }

    pub fn to_scene_shape(&self) -> Option<Shape> {
        if !self.stroke.is_drawable() {
            return None;
        }
        if self.start == self.end {
            return None;
        }
        Some(Shape::Line(SceneLine {
            x1_mm: self.start.x.as_mm(),
            y1_mm: self.start.y.as_mm(),
            x2_mm: self.end.x.as_mm(),
            y2_mm: self.end.y.as_mm(),
            stroke: self.stroke.color.to_scene(),
            width_mm: self.stroke.width.as_mm(),
        }))
    }
}

/// Lightweight path: open polyline or closed polygon.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub id: StableNodeId,
    pub points: Vec<Point>,
    pub closed: bool,
    pub style: Style,
}

impl Path {
    pub fn open(id: StableNodeId, points: Vec<Point>, stroke: Stroke) -> Self {
        Self {
            id,
            points,
            closed: false,
            style: Style::stroked(stroke),
        }
    }

    pub fn closed(id: StableNodeId, points: Vec<Point>, fill: Color) -> Self {
        Self {
            id,
            points,
            closed: true,
            style: Style::filled(fill),
        }
    }

    pub fn to_scene_shape(&self) -> Option<Shape> {
        if self.closed {
            if self.points.len() < 3 {
                return None;
            }
            match self.style.fill {
                Fill::Solid(c) if c.is_valid() => Some(Shape::Polygon(Polygon {
                    points_mm: self
                        .points
                        .iter()
                        .map(|p| (p.x.as_mm(), p.y.as_mm()))
                        .collect(),
                    fill: c.to_scene(),
                })),
                _ => None,
            }
        } else {
            let stroke = self.style.stroke?;
            if self.points.len() < 2 || !stroke.is_drawable() {
                return None;
            }
            Some(Shape::Polyline(Polyline {
                points_mm: self
                    .points
                    .iter()
                    .map(|p| (p.x.as_mm(), p.y.as_mm()))
                    .collect(),
                stroke: stroke.color.to_scene(),
                width_mm: stroke.width.as_mm(),
            }))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub id: StableNodeId,
    pub path: String,
    pub frame: Rect,
}

impl Image {
    pub fn new(id: StableNodeId, path: impl Into<String>, frame: Rect) -> Self {
        Self {
            id,
            path: path.into(),
            frame,
        }
    }

    pub fn to_scene_shape(&self) -> Option<Shape> {
        if self.path.is_empty() || !self.frame.is_drawable() {
            return None;
        }
        Some(Shape::Image(SceneImage {
            path: self.path.clone(),
            x_mm: self.frame.origin.x.as_mm(),
            y_mm: self.frame.origin.y.as_mm(),
            width_mm: self.frame.width().as_mm(),
            height_mm: self.frame.height().as_mm(),
        }))
    }
}

/// Bridge helper: rebuild a filled rectangle from a scene rect (lossy for IDs).
pub fn rectangle_from_scene(id: StableNodeId, rect: &SceneRect) -> Rectangle {
    Rectangle::new(
        id,
        Rect::from_xywh(rect.x_mm, rect.y_mm, rect.width_mm, rect.height_mm),
        Style::filled(Color::from_scene(rect.fill)),
    )
}

/// Placeholder text drawable mapped onto scene [`SceneText`].
pub fn scene_text_box(
    x_mm: f64,
    y_mm: f64,
    size_mm: f64,
    content: impl Into<String>,
    fill: Color,
) -> SceneText {
    SceneText {
        x_mm,
        y_mm,
        size_mm,
        width_mm: None,
        height_mm: None,
        content: content.into(),
        fill: fill.to_scene(),
    }
}
