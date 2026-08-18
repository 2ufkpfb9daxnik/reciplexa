//! Render IR — validated, backend-neutral drawable nodes.

use reciplexa_scene::Color;

/// Stable identifier for a render node (maps to artifact elements).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderNodeId(pub u64);

impl RenderNodeId {
    pub const INVALID: Self = Self(0);

    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// A complete render-ready document.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderDocument {
    pub pages: Vec<RenderPage>,
}

/// One page of render nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderPage {
    pub width_mm: f64,
    pub height_mm: f64,
    pub nodes: Vec<RenderNode>,
}

/// Backend-neutral render primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderNode {
    Rect {
        id: RenderNodeId,
        x_mm: f64,
        y_mm: f64,
        width_mm: f64,
        height_mm: f64,
        fill: Color,
        stroke_width_mm: Option<f64>,
        alpha: f64,
    },
    Circle {
        id: RenderNodeId,
        x_mm: f64,
        y_mm: f64,
        radius_mm: f64,
        fill: Color,
        stroke_width_mm: Option<f64>,
        alpha: f64,
    },
    Polygon {
        id: RenderNodeId,
        points_mm: Vec<(f64, f64)>,
        fill: Color,
        stroke_width_mm: Option<f64>,
        alpha: f64,
    },
    Text {
        id: RenderNodeId,
        x_mm: f64,
        y_mm: f64,
        size_mm: f64,
        width_mm: f64,
        height_mm: f64,
        rotation_deg: f64,
        content: String,
        fill: Color,
        alpha: f64,
    },
    Path {
        id: RenderNodeId,
        points_mm: Vec<(f64, f64)>,
        stroke: Color,
        width_mm: f64,
        closed: bool,
        alpha: f64,
    },
    Image {
        id: RenderNodeId,
        path: String,
        corners_mm: [(f64, f64); 4],
        alpha: f64,
    },
    /// Cluster-preserving glyph run (IR-001). Font identity is a content digest,
    /// not a native handle.
    GlyphRun {
        id: RenderNodeId,
        font_digest: String,
        font_label: String,
        x_mm: f64,
        y_mm: f64,
        size_mm: f64,
        content: String,
        fill: Color,
        alpha: f64,
        gids: Vec<u16>,
        advances_mm: Vec<f64>,
        cluster_starts: Vec<u32>,
        cluster_ends: Vec<u32>,
    },
}

impl RenderNode {
    pub fn id(&self) -> RenderNodeId {
        match self {
            Self::Rect { id, .. }
            | Self::Circle { id, .. }
            | Self::Polygon { id, .. }
            | Self::Text { id, .. }
            | Self::Path { id, .. }
            | Self::Image { id, .. }
            | Self::GlyphRun { id, .. } => *id,
        }
    }
}
