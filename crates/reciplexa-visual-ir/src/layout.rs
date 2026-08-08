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
