//! Minimal layout IR — currently an identity pass over Domain IR.

use crate::domain::{DomainDocument, DomainNode};

/// Layout document after the minimal layout pass.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutDocument {
    pub pages: Vec<LayoutPage>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutPage {
    pub width_mm: f64,
    pub height_mm: f64,
    pub nodes: Vec<DomainNode>,
}

/// Identity layout: Domain geometry is already in page millimeters.
pub fn layout_identity(domain: &DomainDocument) -> LayoutDocument {
    LayoutDocument {
        pages: domain
            .pages
            .iter()
            .map(|p| LayoutPage {
                width_mm: p.width_mm,
                height_mm: p.height_mm,
                nodes: p.nodes.clone(),
            })
            .collect(),
    }
}

/// Convenience: pages of a layout document.
pub fn layout_pages(layout: &LayoutDocument) -> &[LayoutPage] {
    &layout.pages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DomainNode, DomainNodeId, DomainPage};
    use reciplexa_scene::Color;

    #[test]
    fn identity_layout_preserves_nodes() {
        let domain = DomainDocument {
            pages: vec![DomainPage {
                width_mm: 210.0,
                height_mm: 297.0,
                nodes: vec![DomainNode::Circle {
                    id: DomainNodeId::new(1),
                    x_mm: 1.0,
                    y_mm: 2.0,
                    radius_mm: 3.0,
                    fill: Color::BLACK,
                    stroke_width_mm: None,
                    alpha: 1.0,
                    stable_node_id: None,
                    source_byte_start: None,
                    source_byte_end: None,
                }],
            }],
        };
        let layout = layout_identity(&domain);
        assert_eq!(layout.pages[0].nodes.len(), 1);
    }

    #[test]
    fn empty_domain_produces_empty_layout() {
        let domain = DomainDocument { pages: vec![] };
        let layout = layout_identity(&domain);
        assert!(layout.pages.is_empty());
        assert!(layout_pages(&layout).is_empty());
    }
}
