//! Reference Client standard packages (`rpx.std.*`) — Phase 14.
//!
//! These types are a Strangler-friendly façade over scene / identity / motion
//! so Source, GUI, and Backend can share a small typed API before full Package
//! Manifest wiring lands.

#![forbid(unsafe_code)]

pub mod core;
pub mod document;
pub mod layout;
pub mod math;
pub mod style;
pub mod text;
pub mod visual;

pub use core::{Angle, Color, Length, Point, Range, Rect, Size};
pub use document::{Block, Figure, Flow, Heading, List, ListItem, Page, Section, Table};
pub use layout::{Align, Axis, Column, LayoutChild, Padding, Row, Stack};
pub use math::{MathAtom, MathClass};
pub use style::{Fill, Opacity, Stroke, Style};
pub use text::{Font, Paragraph, Span, Text, TextBox, TextStyle};
pub use visual::{
    rectangle_from_scene, scene_text_box, Canvas, Ellipse, Group, Image, Line, Path, Rectangle,
    Transform, VisualNode,
};
