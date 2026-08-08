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
        if page.width_mm <= 0.0 || page.height_mm <= 0.0 || !page.width_mm.is_finite() || !page.height_mm.is_finite() {
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
        RenderNode::Rect { width_mm, height_mm, alpha, .. } => {
            if *width_mm <= 0.0 || *height_mm <= 0.0 {
                return Err(bad("rect dimensions must be positive"));
            }
            if !alpha.is_finite() || !(0.0..=1.0).contains(alpha) {
                return Err(bad("alpha out of range"));
            }
        }
        RenderNode::Circle { radius_mm, alpha, .. } => {
            if *radius_mm <= 0.0 {
                return Err(bad("circle radius must be positive"));
            }
            if !alpha.is_finite() || !(0.0..=1.0).contains(alpha) {
                return Err(bad("alpha out of range"));
            }
        }
        RenderNode::Polygon { points_mm, alpha, .. } => {
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
        RenderNode::Path { points_mm, width_mm, alpha, .. } => {
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
        RenderNode::Image { corners_mm, alpha, .. } => {
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

    #[test]
    fn rejects_empty_document() {
        let doc = RenderDocument { pages: vec![] };
        assert!(matches!(
            validate_render_document(&doc),
            Err(RenderValidationError::EmptyDocument)
        ));
    }

    #[test]
    fn rejects_invalid_rect() {
        let doc = RenderDocument {
            pages: vec![RenderPage {
                width_mm: 210.0,
                height_mm: 297.0,
                nodes: vec![RenderNode::Rect {
                    id: RenderNodeId::new(1),
                    x_mm: 0.0,
                    y_mm: 0.0,
                    width_mm: -1.0,
                    height_mm: 1.0,
                    fill: Color::BLACK,
                    stroke_width_mm: None,
                    alpha: 1.0,
                }],
            }],
        };
        assert!(validate_render_document(&doc).is_err());
    }
}
