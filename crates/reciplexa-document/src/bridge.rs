//! Bridge between scene geometry and document model.

use reciplexa_identity::document::DocumentIdentity;
use reciplexa_identity::package::ModuleId;
use reciplexa_scene::{Color, Page, Rect, Shape, Text};
use reciplexa_source::range::TextRange;
use reciplexa_source::resource::SourceResourceId;

use crate::node::{DocumentNodeKind, NodeStore};
use crate::property::{FillColor, LayoutBox, NodeProperty, TextContent};
use crate::provenance::SourceProvenance;
use crate::snapshot::DocumentSnapshot;

/// High-level GUI edit mapped to document transactions.
#[derive(Debug, Clone, PartialEq)]
pub enum ApplyEdit {
    Move {
        node: reciplexa_identity::document::StableNodeId,
        x: f64,
        y: f64,
    },
    Resize {
        node: reciplexa_identity::document::StableNodeId,
        width: f64,
        height: f64,
    },
    SetText {
        node: reciplexa_identity::document::StableNodeId,
        text: String,
    },
}

/// Build a document snapshot from a scene page (forward direction).
pub fn document_from_scene_page(
    identity: DocumentIdentity,
    page: &Page,
    source_resource_id: SourceResourceId,
) -> DocumentSnapshot {
    let mut snap = DocumentSnapshot::new(identity);
    let root = snap.nodes.root_id().unwrap();
    let page_id = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Page)
        .unwrap();
    for shape in &page.shapes {
        ingest_shape(&mut snap, page_id, shape, source_resource_id);
    }
    snap
}

fn ingest_shape(
    snap: &mut DocumentSnapshot,
    parent: reciplexa_identity::document::StableNodeId,
    shape: &Shape,
    source_id: SourceResourceId,
) {
    match shape {
        Shape::Rect(Rect {
            x_mm,
            y_mm,
            width_mm,
            height_mm,
            fill,
        }) => {
            let id = snap
                .nodes
                .insert_child(parent, DocumentNodeKind::Rectangle)
                .unwrap();
            if let Some(n) = snap.nodes.get_mut(id) {
                n.properties = vec![
                    NodeProperty::Layout(LayoutBox::new(*x_mm, *y_mm, *width_mm, *height_mm)),
                    NodeProperty::Fill(FillColor(*fill)),
                ];
            }
            snap.provenance.insert(
                id,
                SourceProvenance {
                    source_resource_id: source_id,
                    module_id: ModuleId::new(1),
                    text_range: TextRange::EMPTY,
                    syntax_node_id: None,
                },
            );
        }
        Shape::Text(Text {
            x_mm,
            y_mm,
            size_mm,
            width_mm,
            height_mm,
            content,
            fill,
        }) => {
            let id = snap
                .nodes
                .insert_child(parent, DocumentNodeKind::Text)
                .unwrap();
            let w = width_mm.unwrap_or(*size_mm * 10.0);
            let h = height_mm.unwrap_or(*size_mm);
            if let Some(n) = snap.nodes.get_mut(id) {
                n.properties = vec![
                    NodeProperty::Layout(LayoutBox::new(*x_mm, *y_mm, w, h)),
                    NodeProperty::Fill(FillColor(*fill)),
                    NodeProperty::Text(TextContent {
                        text: content.clone(),
                    }),
                ];
            }
            snap.provenance.insert(
                id,
                SourceProvenance {
                    source_resource_id: source_id,
                    module_id: ModuleId::new(1),
                    text_range: TextRange::EMPTY,
                    syntax_node_id: None,
                },
            );
        }
        Shape::Group { children, .. } => {
            let gid = snap
                .nodes
                .insert_child(parent, DocumentNodeKind::Group)
                .unwrap();
            for child in children {
                ingest_shape(snap, gid, child, source_id);
            }
        }
        _ => {}
    }
}

/// Project document nodes back to scene shapes for rendering.
pub fn scene_shapes_from_document(store: &NodeStore) -> Vec<Shape> {
    let mut out = Vec::new();
    if let Some(root) = store.root_id() {
        collect_shapes(store, root, &mut out);
    }
    out
}

fn collect_shapes(
    store: &NodeStore,
    id: reciplexa_identity::document::StableNodeId,
    out: &mut Vec<Shape>,
) {
    let Some(node) = store.get(id) else {
        return;
    };
    match node.kind {
        DocumentNodeKind::Rectangle => {
            if let Some(layout) = node.layout() {
                let color = node
                    .properties
                    .iter()
                    .find_map(|p| match p {
                        NodeProperty::Fill(FillColor(c)) => Some(*c),
                        _ => None,
                    })
                    .unwrap_or(Color::BLACK);
                out.push(Shape::Rect(Rect {
                    x_mm: layout.x,
                    y_mm: layout.y,
                    width_mm: layout.width,
                    height_mm: layout.height,
                    fill: color,
                }));
            }
        }
        DocumentNodeKind::Text => {
            if let (Some(layout), Some(text)) = (node.layout(), node.text()) {
                let color = node
                    .properties
                    .iter()
                    .find_map(|p| match p {
                        NodeProperty::Fill(FillColor(c)) => Some(*c),
                        _ => None,
                    })
                    .unwrap_or(Color::BLACK);
                let size = layout.height / text.text.lines().count().max(1) as f64;
                out.push(Shape::Text(Text {
                    x_mm: layout.x,
                    y_mm: layout.y,
                    size_mm: size,
                    width_mm: Some(layout.width),
                    height_mm: Some(layout.height),
                    content: text.text.clone(),
                    fill: color,
                }));
            }
        }
        DocumentNodeKind::Group | DocumentNodeKind::Page | DocumentNodeKind::Document => {
            for child in &node.children.clone() {
                collect_shapes(store, *child, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::PaperSize;

    #[test]
    fn roundtrip_rect_through_document() {
        let page = Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Rect(Rect {
                x_mm: 5.0,
                y_mm: 6.0,
                width_mm: 10.0,
                height_mm: 20.0,
                fill: Color::RED,
            })],
        };
        let snap =
            document_from_scene_page(DocumentIdentity::new(1), &page, SourceResourceId::new(1));
        let shapes = scene_shapes_from_document(&snap.nodes);
        assert_eq!(shapes.len(), 1);
    }
}
