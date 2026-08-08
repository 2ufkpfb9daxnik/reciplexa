//! Lower scene documents into Render IR.

use reciplexa_scene::Document;
use reciplexa_view::{flatten_page, WorldShape};

use crate::provenance::{ProvenanceMap, RenderProvenance};
use crate::render::{RenderDocument, RenderNode, RenderNodeId, RenderPage};

/// Lower a scene [`Document`] into validated Render IR.
pub fn lower_scene_document(doc: &Document) -> (RenderDocument, ProvenanceMap) {
    let mut next_id = 1u64;
    let mut pages = Vec::new();
    let mut provenance = ProvenanceMap::default();

    for (page_idx, page) in doc.pages.iter().enumerate() {
        let Some((_, shapes)) = flatten_page(doc, page_idx) else {
            continue;
        };
        let mut nodes = Vec::new();
        for shape in shapes {
            let id = RenderNodeId::new(next_id);
            next_id += 1;
            let node = world_shape_to_render(id, &shape);
            provenance.insert(RenderProvenance {
                render_id: id,
                stable_node_id: None,
                source_byte_start: None,
                source_byte_end: None,
            });
            nodes.push(node);
        }
        pages.push(RenderPage {
            width_mm: page.paper.width_mm,
            height_mm: page.paper.height_mm,
            nodes,
        });
    }

    (
        RenderDocument { pages },
        provenance,
    )
}

fn world_shape_to_render(id: RenderNodeId, shape: &WorldShape) -> RenderNode {
    match shape {
        WorldShape::Circle(c) => RenderNode::Circle {
            id,
            x_mm: c.x_mm,
            y_mm: c.y_mm,
            radius_mm: c.radius_mm,
            fill: c.color,
            stroke_width_mm: c.stroke_width_mm,
            alpha: c.alpha,
        },
        WorldShape::Polygon(p) => RenderNode::Polygon {
            id,
            points_mm: p.points_mm.clone(),
            fill: p.color,
            stroke_width_mm: p.stroke_width_mm,
            alpha: p.alpha,
        },
        WorldShape::Text(t) => RenderNode::Text {
            id,
            x_mm: t.x_mm,
            y_mm: t.y_mm,
            size_mm: t.size_mm,
            width_mm: t.width_mm,
            height_mm: t.height_mm,
            rotation_deg: t.rotation_deg,
            content: t.content.clone(),
            fill: t.fill,
            alpha: t.alpha,
        },
        WorldShape::Path(p) => RenderNode::Path {
            id,
            points_mm: p.points_mm.clone(),
            stroke: p.stroke,
            width_mm: p.width_mm,
            closed: p.closed,
            alpha: p.alpha,
        },
        WorldShape::Image(i) => RenderNode::Image {
            id,
            path: i.path.clone(),
            corners_mm: i.corners_mm,
            alpha: i.alpha,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::{Color, Document, Page, PaperSize, Rect, Shape};

    #[test]
    fn lowers_rect_to_render_ir() {
        let mut doc = Document::default();
        doc.pages.push(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Rect(Rect {
                x_mm: 10.0,
                y_mm: 20.0,
                width_mm: 30.0,
                height_mm: 40.0,
                fill: Color::RED,
            })],
        });
        let (render, prov) = lower_scene_document(&doc);
        assert_eq!(render.pages.len(), 1);
        assert_eq!(render.pages[0].nodes.len(), 1);
        assert!(prov.get(RenderNodeId::new(1)).is_some());
    }
}
