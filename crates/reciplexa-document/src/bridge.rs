//! Bridge between scene geometry and document model.

use reciplexa_identity::document::DocumentIdentity;
use reciplexa_identity::package::ModuleId;
use reciplexa_scene::{Color, Page, Rect, Shape, Text};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;
use reciplexa_source::resource::SourceResourceId;

use crate::node::{DocumentNodeKind, NodeStore};
use crate::property::{FillColor, LayoutBox, NodeProperty, TextContent};
use crate::provenance::SourceProvenance;
use crate::snapshot::DocumentSnapshot;

/// Layer span from CST sync (byte range + label).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerSpan {
    pub byte_start: usize,
    pub byte_end: usize,
    pub label: String,
}

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
    document_from_scene_page_with_layers(identity, page, &[], source_resource_id, &[])
}

/// Build snapshot with CST layer provenance aligned to drawable paint order.
pub fn document_from_scene_page_with_layers(
    identity: DocumentIdentity,
    page: &Page,
    layers: &[LayerSpan],
    source_resource_id: SourceResourceId,
    syntax_ids: &[Option<reciplexa_identity::syntax::SyntaxNodeId>],
) -> DocumentSnapshot {
    let mut snap = DocumentSnapshot::new(identity);
    let root = snap.nodes.root_id().unwrap();
    let page_id = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Page)
        .unwrap();
    snap.references.link(root, page_id);
    let mut layer_idx = 0usize;
    for shape in &page.shapes {
        ingest_shape(
            &mut snap,
            page_id,
            shape,
            source_resource_id,
            layers.get(layer_idx),
            syntax_ids.get(layer_idx).and_then(|o| *o),
        );
        if matches!(shape, Shape::Rect(_) | Shape::Text(_)) {
            layer_idx += 1;
        }
    }
    snap
}

fn ingest_shape(
    snap: &mut DocumentSnapshot,
    parent: reciplexa_identity::document::StableNodeId,
    shape: &Shape,
    source_id: SourceResourceId,
    layer: Option<&LayerSpan>,
    syntax_node_id: Option<reciplexa_identity::syntax::SyntaxNodeId>,
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
            snap.nodes.get_mut(id).expect("just inserted").properties = vec![
                NodeProperty::Layout(LayoutBox::new(*x_mm, *y_mm, *width_mm, *height_mm)),
                NodeProperty::Fill(FillColor(*fill)),
            ];
            snap.provenance
                .insert(id, provenance_for_layer(source_id, layer, syntax_node_id));
            snap.references.link(parent, id);
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
            snap.nodes.get_mut(id).expect("just inserted").properties = vec![
                NodeProperty::Layout(LayoutBox::new(*x_mm, *y_mm, w, h)),
                NodeProperty::Fill(FillColor(*fill)),
                NodeProperty::Text(TextContent {
                    text: content.clone(),
                }),
            ];
            snap.provenance
                .insert(id, provenance_for_layer(source_id, layer, syntax_node_id));
            snap.references.link(parent, id);
        }
        Shape::Group { children, .. } => {
            let gid = snap
                .nodes
                .insert_child(parent, DocumentNodeKind::Group)
                .unwrap();
            for child in children {
                ingest_shape(snap, gid, child, source_id, None, None);
            }
            snap.references.link(parent, gid);
        }
        _ => {}
    }
}

fn provenance_for_layer(
    source_id: SourceResourceId,
    layer: Option<&LayerSpan>,
    syntax_node_id: Option<reciplexa_identity::syntax::SyntaxNodeId>,
) -> SourceProvenance {
    let text_range = layer
        .and_then(|l| {
            TextRange::try_new(
                ByteOffset::new(l.byte_start as u32),
                ByteOffset::new(l.byte_end as u32),
            )
            .ok()
        })
        .unwrap_or(TextRange::EMPTY);
    SourceProvenance {
        source_resource_id: source_id,
        module_id: ModuleId::new(1),
        text_range,
        syntax_node_id,
        kind: None,
    }
}

/// Collect stable node ids for drawable shapes in paint order.
pub fn drawable_node_ids(store: &NodeStore) -> Vec<reciplexa_identity::document::StableNodeId> {
    let mut out = Vec::new();
    if let Some(root) = store.root_id() {
        collect_drawable_ids(store, root, &mut out);
    }
    out
}

fn collect_drawable_ids(
    store: &NodeStore,
    id: reciplexa_identity::document::StableNodeId,
    out: &mut Vec<reciplexa_identity::document::StableNodeId>,
) {
    let Some(node) = store.get(id) else {
        return;
    };
    match node.kind {
        DocumentNodeKind::Rectangle | DocumentNodeKind::Text => out.push(id),
        DocumentNodeKind::Group | DocumentNodeKind::Page | DocumentNodeKind::Document => {
            for child in &node.children.clone() {
                collect_drawable_ids(store, *child, out);
            }
        }
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
