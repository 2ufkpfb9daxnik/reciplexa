//! Domain IR — document meaning before layout.

use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::Color;

/// Identifier within Domain IR (distinct from render ids).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DomainNodeId(pub u64);

impl DomainNodeId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Domain-level document after scene ingestion.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainDocument {
    pub pages: Vec<DomainPage>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DomainPage {
    pub width_mm: f64,
    pub height_mm: f64,
    pub nodes: Vec<DomainNode>,
}

/// Scene meaning retained before layout/render specialization.
#[derive(Debug, Clone, PartialEq)]
pub enum DomainNode {
    Circle {
        id: DomainNodeId,
        x_mm: f64,
        y_mm: f64,
        radius_mm: f64,
        fill: Color,
        stroke_width_mm: Option<f64>,
        alpha: f64,
        stable_node_id: Option<StableNodeId>,
        source_byte_start: Option<u32>,
        source_byte_end: Option<u32>,
    },
    Rect {
        id: DomainNodeId,
        x_mm: f64,
        y_mm: f64,
        width_mm: f64,
        height_mm: f64,
        fill: Color,
        stroke_width_mm: Option<f64>,
        alpha: f64,
        stable_node_id: Option<StableNodeId>,
        source_byte_start: Option<u32>,
        source_byte_end: Option<u32>,
    },
    Ellipse {
        id: DomainNodeId,
        x_mm: f64,
        y_mm: f64,
        rx_mm: f64,
        ry_mm: f64,
        fill: Color,
        alpha: f64,
        stable_node_id: Option<StableNodeId>,
        source_byte_start: Option<u32>,
        source_byte_end: Option<u32>,
    },
    Polygon {
        id: DomainNodeId,
        points_mm: Vec<(f64, f64)>,
        fill: Color,
        stroke_width_mm: Option<f64>,
        alpha: f64,
        stable_node_id: Option<StableNodeId>,
        source_byte_start: Option<u32>,
        source_byte_end: Option<u32>,
    },
    Text {
        id: DomainNodeId,
        x_mm: f64,
        y_mm: f64,
        size_mm: f64,
        width_mm: f64,
        height_mm: f64,
        rotation_deg: f64,
        content: String,
        fill: Color,
        alpha: f64,
        stable_node_id: Option<StableNodeId>,
        source_byte_start: Option<u32>,
        source_byte_end: Option<u32>,
    },
    Path {
        id: DomainNodeId,
        points_mm: Vec<(f64, f64)>,
        stroke: Color,
        width_mm: f64,
        closed: bool,
        alpha: f64,
        stable_node_id: Option<StableNodeId>,
        source_byte_start: Option<u32>,
        source_byte_end: Option<u32>,
    },
    Image {
        id: DomainNodeId,
        path: String,
        corners_mm: [(f64, f64); 4],
        alpha: f64,
        stable_node_id: Option<StableNodeId>,
        source_byte_start: Option<u32>,
        source_byte_end: Option<u32>,
    },
}

impl DomainNode {
    pub fn id(&self) -> DomainNodeId {
        match self {
            Self::Circle { id, .. }
            | Self::Rect { id, .. }
            | Self::Ellipse { id, .. }
            | Self::Polygon { id, .. }
            | Self::Text { id, .. }
            | Self::Path { id, .. }
            | Self::Image { id, .. } => *id,
        }
    }

    pub fn stable_node_id(&self) -> Option<StableNodeId> {
        match self {
            Self::Circle { stable_node_id, .. }
            | Self::Rect { stable_node_id, .. }
            | Self::Ellipse { stable_node_id, .. }
            | Self::Polygon { stable_node_id, .. }
            | Self::Text { stable_node_id, .. }
            | Self::Path { stable_node_id, .. }
            | Self::Image { stable_node_id, .. } => *stable_node_id,
        }
    }

    pub fn source_bytes(&self) -> (Option<u32>, Option<u32>) {
        match self {
            Self::Circle {
                source_byte_start,
                source_byte_end,
                ..
            }
            | Self::Rect {
                source_byte_start,
                source_byte_end,
                ..
            }
            | Self::Ellipse {
                source_byte_start,
                source_byte_end,
                ..
            }
            | Self::Polygon {
                source_byte_start,
                source_byte_end,
                ..
            }
            | Self::Text {
                source_byte_start,
                source_byte_end,
                ..
            }
            | Self::Path {
                source_byte_start,
                source_byte_end,
                ..
            }
            | Self::Image {
                source_byte_start,
                source_byte_end,
                ..
            } => (*source_byte_start, *source_byte_end),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_identity::document::StableNodeId;

    fn base_fields() -> (Option<StableNodeId>, Option<u32>, Option<u32>) {
        (Some(StableNodeId::new(7)), Some(10), Some(20))
    }

    #[test]
    fn circle_id_stable_and_source_bytes() {
        let (sid, start, end) = base_fields();
        let node = DomainNode::Circle {
            id: DomainNodeId::new(1),
            x_mm: 0.0,
            y_mm: 0.0,
            radius_mm: 1.0,
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha: 1.0,
            stable_node_id: sid,
            source_byte_start: start,
            source_byte_end: end,
        };
        assert_eq!(node.id(), DomainNodeId::new(1));
        assert_eq!(node.stable_node_id(), sid);
        assert_eq!(node.source_bytes(), (start, end));
    }

    #[test]
    fn rect_id_stable_and_source_bytes() {
        let (sid, start, end) = base_fields();
        let node = DomainNode::Rect {
            id: DomainNodeId::new(2),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 1.0,
            height_mm: 1.0,
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha: 1.0,
            stable_node_id: sid,
            source_byte_start: start,
            source_byte_end: end,
        };
        assert_eq!(node.id(), DomainNodeId::new(2));
        assert_eq!(node.stable_node_id(), sid);
        assert_eq!(node.source_bytes(), (start, end));
    }

    #[test]
    fn ellipse_id_stable_and_source_bytes() {
        let (sid, start, end) = base_fields();
        let node = DomainNode::Ellipse {
            id: DomainNodeId::new(3),
            x_mm: 0.0,
            y_mm: 0.0,
            rx_mm: 1.0,
            ry_mm: 2.0,
            fill: Color::BLACK,
            alpha: 1.0,
            stable_node_id: sid,
            source_byte_start: start,
            source_byte_end: end,
        };
        assert_eq!(node.id(), DomainNodeId::new(3));
        assert_eq!(node.stable_node_id(), sid);
        assert_eq!(node.source_bytes(), (start, end));
    }

    #[test]
    fn polygon_id_stable_and_source_bytes() {
        let (sid, start, end) = base_fields();
        let node = DomainNode::Polygon {
            id: DomainNodeId::new(4),
            points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha: 1.0,
            stable_node_id: sid,
            source_byte_start: start,
            source_byte_end: end,
        };
        assert_eq!(node.id(), DomainNodeId::new(4));
        assert_eq!(node.stable_node_id(), sid);
        assert_eq!(node.source_bytes(), (start, end));
    }

    #[test]
    fn text_id_stable_and_source_bytes() {
        let (sid, start, end) = base_fields();
        let node = DomainNode::Text {
            id: DomainNodeId::new(5),
            x_mm: 0.0,
            y_mm: 0.0,
            size_mm: 12.0,
            width_mm: 10.0,
            height_mm: 10.0,
            rotation_deg: 0.0,
            content: "x".into(),
            fill: Color::BLACK,
            alpha: 1.0,
            stable_node_id: sid,
            source_byte_start: start,
            source_byte_end: end,
        };
        assert_eq!(node.id(), DomainNodeId::new(5));
        assert_eq!(node.stable_node_id(), sid);
        assert_eq!(node.source_bytes(), (start, end));
    }

    #[test]
    fn path_id_stable_and_source_bytes() {
        let (sid, start, end) = base_fields();
        let node = DomainNode::Path {
            id: DomainNodeId::new(6),
            points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
            stroke: Color::BLACK,
            width_mm: 1.0,
            closed: false,
            alpha: 1.0,
            stable_node_id: sid,
            source_byte_start: start,
            source_byte_end: end,
        };
        assert_eq!(node.id(), DomainNodeId::new(6));
        assert_eq!(node.stable_node_id(), sid);
        assert_eq!(node.source_bytes(), (start, end));
    }

    #[test]
    fn image_id_stable_and_source_bytes() {
        let (sid, start, end) = base_fields();
        let node = DomainNode::Image {
            id: DomainNodeId::new(7),
            path: "a.png".into(),
            corners_mm: [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            alpha: 1.0,
            stable_node_id: sid,
            source_byte_start: start,
            source_byte_end: end,
        };
        assert_eq!(node.id(), DomainNodeId::new(7));
        assert_eq!(node.stable_node_id(), sid);
        assert_eq!(node.source_bytes(), (start, end));
    }

    #[test]
    fn none_provenance_fields() {
        let node = DomainNode::Rect {
            id: DomainNodeId::new(8),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 1.0,
            height_mm: 1.0,
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha: 1.0,
            stable_node_id: None,
            source_byte_start: None,
            source_byte_end: None,
        };
        assert_eq!(node.stable_node_id(), None);
        assert_eq!(node.source_bytes(), (None, None));
    }
}
