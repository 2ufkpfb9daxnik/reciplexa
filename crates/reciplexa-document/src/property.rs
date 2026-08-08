//! Node properties for the vertical slice.

use reciplexa_scene::Color;

/// Axis-aligned layout in millimeters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl LayoutBox {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FillColor(pub Color);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextContent {
    pub text: String,
}

/// Editable properties on document nodes.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeProperty {
    Layout(LayoutBox),
    Fill(FillColor),
    Text(TextContent),
}
