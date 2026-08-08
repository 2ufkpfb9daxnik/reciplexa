//! Render IR structural validation.

use crate::render::{RenderDocument, RenderNode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderValidationError {
    EmptyDocument,
    InvalidPageSize { page: usize },
    InvalidNode { page: usize, reason: String },
    DuplicateNodeId(u64),
}

/// Validate render IR invariants before planning.
pub fn validate_render_document(doc: &RenderDocument) -> Result<(), RenderValidationError> {
    if doc.pages.is_empty() {
        return Err(RenderValidationError::EmptyDocument);
    }
    let mut seen = std::collections::HashSet::new();
    for (pi, page) in doc.pages.iter().enumerate() {
        if page.width_mm <= 0.0
            || page.height_mm <= 0.0
            || !page.width_mm.is_finite()
            || !page.height_mm.is_finite()
        {
            return Err(RenderValidationError::InvalidPageSize { page: pi });
        }
        for node in &page.nodes {
            let id = node.id().0;
            if !seen.insert(id) {
                return Err(RenderValidationError::DuplicateNodeId(id));
            }
            validate_node(pi, node)?;
        }
    }
    Ok(())
}

fn validate_node(page: usize, node: &RenderNode) -> Result<(), RenderValidationError> {
    let bad = |reason: &str| RenderValidationError::InvalidNode {
        page,
        reason: reason.into(),
    };
    match node {
        RenderNode::Rect {
            width_mm,
            height_mm,
            alpha,
            ..
        } => {
            if *width_mm <= 0.0 || *height_mm <= 0.0 {
                return Err(bad("rect dimensions must be positive"));
            }
            if !alpha.is_finite() || !(0.0..=1.0).contains(alpha) {
                return Err(bad("alpha out of range"));
            }
        }
        RenderNode::Circle {
            radius_mm, alpha, ..
        } => {
            if *radius_mm <= 0.0 {
                return Err(bad("circle radius must be positive"));
            }
            if !alpha.is_finite() || !(0.0..=1.0).contains(alpha) {
                return Err(bad("alpha out of range"));
            }
        }
        RenderNode::Polygon {
            points_mm, alpha, ..
        } => {
            if points_mm.len() < 3 {
                return Err(bad("polygon needs at least 3 points"));
            }
            for (x, y) in points_mm {
                if !x.is_finite() || !y.is_finite() {
                    return Err(bad("polygon coordinate not finite"));
                }
            }
            if !alpha.is_finite() || !(0.0..=1.0).contains(alpha) {
                return Err(bad("alpha out of range"));
            }
        }
        RenderNode::Text { size_mm, alpha, .. } => {
            if *size_mm <= 0.0 {
                return Err(bad("text size must be positive"));
            }
            if !alpha.is_finite() || !(0.0..=1.0).contains(alpha) {
                return Err(bad("alpha out of range"));
            }
        }
        RenderNode::Path {
            points_mm,
            width_mm,
            alpha,
            ..
        } => {
            if points_mm.len() < 2 {
                return Err(bad("path needs at least 2 points"));
            }
            if *width_mm <= 0.0 {
                return Err(bad("path width must be positive"));
            }
            if !alpha.is_finite() || !(0.0..=1.0).contains(alpha) {
                return Err(bad("alpha out of range"));
            }
        }
        RenderNode::Image {
            corners_mm, alpha, ..
        } => {
            for (x, y) in corners_mm {
                if !x.is_finite() || !y.is_finite() {
                    return Err(bad("image corner not finite"));
                }
            }
            if !alpha.is_finite() || !(0.0..=1.0).contains(alpha) {
                return Err(bad("alpha out of range"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{RenderNodeId, RenderPage};
    use reciplexa_scene::Color;

    fn base_page(nodes: Vec<RenderNode>) -> RenderDocument {
        RenderDocument {
            pages: vec![RenderPage {
                width_mm: 210.0,
                height_mm: 297.0,
                nodes,
            }],
        }
    }

    fn rect_node(id: u64, width_mm: f64, height_mm: f64, alpha: f64) -> RenderNode {
        RenderNode::Rect {
            id: RenderNodeId::new(id),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm,
            height_mm,
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha,
        }
    }

    #[test]
    fn rejects_empty_document() {
        let doc = RenderDocument { pages: vec![] };
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::EmptyDocument)
        ));
    }

    #[test]
    fn rejects_zero_page_width() {
        let doc = RenderDocument {
            pages: vec![RenderPage {
                width_mm: 0.0,
                height_mm: 297.0,
                nodes: vec![],
            }],
        };
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::InvalidPageSize { page: 0 })
        ));
    }

    #[test]
    fn rejects_negative_page_height() {
        let doc = RenderDocument {
            pages: vec![RenderPage {
                width_mm: 210.0,
                height_mm: -1.0,
                nodes: vec![],
            }],
        };
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::InvalidPageSize { page: 0 })
        ));
    }

    #[test]
    fn rejects_nan_page_size() {
        let doc = RenderDocument {
            pages: vec![RenderPage {
                width_mm: f64::NAN,
                height_mm: 297.0,
                nodes: vec![],
            }],
        };
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::InvalidPageSize { page: 0 })
        ));
    }

    #[test]
    fn rejects_duplicate_node_ids() {
        let doc = base_page(vec![
            rect_node(1, 10.0, 10.0, 1.0),
            rect_node(1, 5.0, 5.0, 1.0),
        ]);
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::DuplicateNodeId(1))
        ));
    }

    #[test]
    fn rejects_invalid_rect_dimensions() {
        let doc = base_page(vec![rect_node(1, -1.0, 1.0, 1.0)]);
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::InvalidNode { reason, .. })
            if reason == "rect dimensions must be positive"
        ));
    }

    #[test]
    fn rejects_rect_zero_height() {
        let doc = base_page(vec![rect_node(1, 1.0, 0.0, 1.0)]);
        assert!(validate_render_document(&doc).is_err());
    }

    #[test]
    fn rejects_rect_alpha_below_zero() {
        let doc = base_page(vec![rect_node(1, 1.0, 1.0, -0.1)]);
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::InvalidNode { reason, .. })
            if reason == "alpha out of range"
        ));
    }

    #[test]
    fn rejects_rect_alpha_above_one() {
        let doc = base_page(vec![rect_node(1, 1.0, 1.0, 1.1)]);
        assert!(validate_render_document(&doc).is_err());
    }

    #[test]
    fn rejects_rect_nan_alpha() {
        let doc = base_page(vec![rect_node(1, 1.0, 1.0, f64::NAN)]);
        assert!(validate_render_document(&doc).is_err());
    }

    #[test]
    fn rejects_circle_zero_radius() {
        let doc = base_page(vec![RenderNode::Circle {
            id: RenderNodeId::new(1),
            x_mm: 0.0,
            y_mm: 0.0,
            radius_mm: 0.0,
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha: 1.0,
        }]);
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::InvalidNode { reason, .. })
            if reason == "circle radius must be positive"
        ));
    }

    #[test]
    fn rejects_circle_negative_radius() {
        let doc = base_page(vec![RenderNode::Circle {
            id: RenderNodeId::new(1),
            x_mm: 0.0,
            y_mm: 0.0,
            radius_mm: -2.0,
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha: 1.0,
        }]);
        assert!(validate_render_document(&doc).is_err());
    }

    #[test]
    fn rejects_circle_alpha_out_of_range() {
        let doc = base_page(vec![RenderNode::Circle {
            id: RenderNodeId::new(1),
            x_mm: 0.0,
            y_mm: 0.0,
            radius_mm: 5.0,
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha: 2.0,
        }]);
        assert!(validate_render_document(&doc).is_err());
    }

    #[test]
    fn rejects_polygon_too_few_points() {
        let doc = base_page(vec![RenderNode::Polygon {
            id: RenderNodeId::new(1),
            points_mm: vec![(0.0, 0.0), (1.0, 0.0)],
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha: 1.0,
        }]);
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::InvalidNode { reason, .. })
            if reason == "polygon needs at least 3 points"
        ));
    }

    #[test]
    fn rejects_polygon_non_finite_coordinate() {
        let doc = base_page(vec![RenderNode::Polygon {
            id: RenderNodeId::new(1),
            points_mm: vec![(0.0, 0.0), (1.0, f64::NAN), (0.0, 1.0)],
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha: 1.0,
        }]);
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::InvalidNode { reason, .. })
            if reason == "polygon coordinate not finite"
        ));
    }

    #[test]
    fn rejects_polygon_alpha_out_of_range() {
        let doc = base_page(vec![RenderNode::Polygon {
            id: RenderNodeId::new(1),
            points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha: -0.5,
        }]);
        assert!(validate_render_document(&doc).is_err());
    }

    #[test]
    fn rejects_text_zero_size() {
        let doc = base_page(vec![RenderNode::Text {
            id: RenderNodeId::new(1),
            x_mm: 0.0,
            y_mm: 0.0,
            size_mm: 0.0,
            width_mm: 10.0,
            height_mm: 10.0,
            rotation_deg: 0.0,
            content: "x".into(),
            fill: Color::BLACK,
            alpha: 1.0,
        }]);
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::InvalidNode { reason, .. })
            if reason == "text size must be positive"
        ));
    }

    #[test]
    fn rejects_text_alpha_out_of_range() {
        let doc = base_page(vec![RenderNode::Text {
            id: RenderNodeId::new(1),
            x_mm: 0.0,
            y_mm: 0.0,
            size_mm: 12.0,
            width_mm: 10.0,
            height_mm: 10.0,
            rotation_deg: 0.0,
            content: "x".into(),
            fill: Color::BLACK,
            alpha: f64::INFINITY,
        }]);
        assert!(validate_render_document(&doc).is_err());
    }

    #[test]
    fn rejects_path_too_few_points() {
        let doc = base_page(vec![RenderNode::Path {
            id: RenderNodeId::new(1),
            points_mm: vec![(0.0, 0.0)],
            stroke: Color::BLACK,
            width_mm: 1.0,
            closed: false,
            alpha: 1.0,
        }]);
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::InvalidNode { reason, .. })
            if reason == "path needs at least 2 points"
        ));
    }

    #[test]
    fn rejects_path_zero_width() {
        let doc = base_page(vec![RenderNode::Path {
            id: RenderNodeId::new(1),
            points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
            stroke: Color::BLACK,
            width_mm: 0.0,
            closed: false,
            alpha: 1.0,
        }]);
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::InvalidNode { reason, .. })
            if reason == "path width must be positive"
        ));
    }

    #[test]
    fn rejects_path_alpha_out_of_range() {
        let doc = base_page(vec![RenderNode::Path {
            id: RenderNodeId::new(1),
            points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
            stroke: Color::BLACK,
            width_mm: 1.0,
            closed: false,
            alpha: 1.5,
        }]);
        assert!(validate_render_document(&doc).is_err());
    }

    #[test]
    fn rejects_image_non_finite_corner() {
        let doc = base_page(vec![RenderNode::Image {
            id: RenderNodeId::new(1),
            path: "a.png".into(),
            corners_mm: [(0.0, 0.0), (f64::INFINITY, 0.0), (1.0, 1.0), (0.0, 1.0)],
            alpha: 1.0,
        }]);
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::InvalidNode { reason, .. })
            if reason == "image corner not finite"
        ));
    }

    #[test]
    fn rejects_image_alpha_out_of_range() {
        let doc = base_page(vec![RenderNode::Image {
            id: RenderNodeId::new(1),
            path: "a.png".into(),
            corners_mm: [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            alpha: -1.0,
        }]);
        assert!(validate_render_document(&doc).is_err());
    }

    #[test]
    fn accepts_valid_minimal_all_kinds() {
        let doc = base_page(vec![
            rect_node(1, 1.0, 1.0, 0.0),
            RenderNode::Circle {
                id: RenderNodeId::new(2),
                x_mm: 1.0,
                y_mm: 1.0,
                radius_mm: 0.1,
                fill: Color::BLACK,
                stroke_width_mm: None,
                alpha: 1.0,
            },
            RenderNode::Polygon {
                id: RenderNodeId::new(3),
                points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
                fill: Color::BLACK,
                stroke_width_mm: None,
                alpha: 0.5,
            },
            RenderNode::Text {
                id: RenderNodeId::new(4),
                x_mm: 0.0,
                y_mm: 0.0,
                size_mm: 1.0,
                width_mm: 10.0,
                height_mm: 10.0,
                rotation_deg: 0.0,
                content: "ok".into(),
                fill: Color::BLACK,
                alpha: 1.0,
            },
            RenderNode::Path {
                id: RenderNodeId::new(5),
                points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
                stroke: Color::BLACK,
                width_mm: 0.1,
                closed: false,
                alpha: 1.0,
            },
            RenderNode::Image {
                id: RenderNodeId::new(6),
                path: "img.png".into(),
                corners_mm: [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
                alpha: 1.0,
            },
        ]);
        assert!(validate_render_document(&doc).is_ok());
    }

    #[test]
    fn accepts_alpha_boundary_values() {
        let doc = base_page(vec![
            rect_node(1, 1.0, 1.0, 0.0),
            rect_node(2, 1.0, 1.0, 1.0),
        ]);
        assert!(validate_render_document(&doc).is_ok());
    }
}
