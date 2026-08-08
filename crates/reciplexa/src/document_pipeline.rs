//! Document-mediated pipeline stage (Phase 4 bridge).

use reciplexa_bind::resolve_source;
use reciplexa_document::{
    document_from_scene_page_with_layers, drawable_node_ids, scene_shapes_from_document,
    DocumentSnapshot, LayerSpan, TransactionBuilder,
};
use reciplexa_identity::document::DocumentIdentity;
use reciplexa_lower::{collect_layers_page, lower_source};
use reciplexa_source::resource::SourceResourceId;
use reciplexa_syntax::{build_identity_map, parse_source};

/// Build snapshot from lowered scene only (no source provenance).
pub fn document_snapshot_from_lowered(
    scene: &reciplexa_scene::Document,
) -> Result<DocumentSnapshot, String> {
    let page = scene
        .pages
        .first()
        .ok_or_else(|| "document has no pages".to_string())?;
    Ok(document_from_scene_page_with_layers(
        DocumentIdentity::new(1),
        page,
        &[],
        SourceResourceId::new(1),
        &[],
    ))
}

/// Parse, lower, and build an editable document snapshot with provenance.
pub fn document_snapshot_from_source(
    source: &str,
    identity: DocumentIdentity,
) -> Result<DocumentSnapshot, String> {
    let resolve = resolve_source(source);
    if !resolve.is_ok() {
        return Err(resolve.errors[0].message.clone());
    }
    let parse = parse_source(source);
    if parse.has_errors() {
        return Err(parse.errors[0].message.clone());
    }
    let id_map = build_identity_map(&parse.root);
    let scene = lower_source(source).map_err(|e| e.message)?;
    let page = scene
        .pages
        .first()
        .ok_or_else(|| "document has no pages".to_string())?;
    let layers = collect_layers_page(source, 0).map_err(|e| e.message)?;
    let layer_spans: Vec<LayerSpan> = layers
        .iter()
        .map(|l| LayerSpan {
            byte_start: l.byte_start,
            byte_end: l.byte_end,
            label: l.label.clone(),
        })
        .collect();
    let syntax_ids: Vec<_> = layer_spans
        .iter()
        .map(|l| id_map.get_byte_offsets(l.byte_start as u32, l.byte_end as u32))
        .collect();
    Ok(document_from_scene_page_with_layers(
        identity,
        page,
        &layer_spans,
        SourceResourceId::new(1),
        &syntax_ids,
    ))
}

/// Apply a layout move via document transaction and project back to scene shapes.
pub fn move_node_in_snapshot(
    snap: &mut DocumentSnapshot,
    node: reciplexa_identity::document::StableNodeId,
    x: f64,
    y: f64,
) -> Result<(), String> {
    let layout = snap
        .nodes
        .get(node)
        .and_then(|n| n.layout())
        .ok_or_else(|| "node has no layout".to_string())?;
    let rev_before = snap.revision.get();
    let mut tx = TransactionBuilder::new();
    tx.set_layout(
        node,
        reciplexa_document::LayoutBox::new(x, y, layout.width, layout.height),
    );
    snap.nodes
        .root_id()
        .ok_or_else(|| "missing root".to_string())?;
    match tx.into_transaction().apply(snap) {
        Ok(_) => {
            assert!(snap.revision.get() > rev_before);
            Ok(())
        }
        Err(e) => Err(format!("{e:?}")),
    }
}

/// Rebuild drawable shapes from the current snapshot (preview path).
pub fn preview_shapes(snap: &DocumentSnapshot) -> Vec<reciplexa_scene::Shape> {
    scene_shapes_from_document(&snap.nodes)
}

/// Build paint-order provenance hints aligned with flatten leaf order.
pub fn provenance_hints_for_scene(
    scene: &reciplexa_scene::Document,
    snap: &DocumentSnapshot,
) -> Vec<Option<reciplexa_visual_ir::NodeSourceHint>> {
    let mut drawables = drawable_node_ids(&snap.nodes).into_iter();
    let mut out = Vec::new();
    for page in &scene.pages {
        collect_shape_hints(&page.shapes, snap, &mut drawables, &mut out);
    }
    out
}

fn collect_shape_hints(
    shapes: &[reciplexa_scene::Shape],
    snap: &DocumentSnapshot,
    drawables: &mut impl Iterator<Item = reciplexa_identity::document::StableNodeId>,
    out: &mut Vec<Option<reciplexa_visual_ir::NodeSourceHint>>,
) {
    use reciplexa_scene::Shape;
    for shape in shapes {
        match shape {
            Shape::Rect(_) | Shape::Text(_) => {
                out.push(drawables.next().and_then(|id| hint_for_node(snap, id)));
            }
            Shape::Opacity { children, .. } | Shape::Group { children, .. } => {
                collect_shape_hints(children, snap, drawables, out);
            }
            Shape::Circle(_)
            | Shape::Ellipse(_)
            | Shape::Ring(_)
            | Shape::Frame(_)
            | Shape::Line(_)
            | Shape::Polyline(_)
            | Shape::Polygon(_)
            | Shape::Image(_) => {
                out.push(None);
            }
        }
    }
}

fn hint_for_node(
    snap: &DocumentSnapshot,
    id: reciplexa_identity::document::StableNodeId,
) -> Option<reciplexa_visual_ir::NodeSourceHint> {
    let prov = snap.provenance.get(id)?;
    Some(reciplexa_visual_ir::NodeSourceHint {
        stable_node_id: id,
        source_byte_start: prov.text_range.start().get(),
        source_byte_end: prov.text_range.end().get(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_snapshot_from_black_circle() {
        let src = "(page a4 (circle 105 148.5 40))";
        let snap = document_snapshot_from_source(src, DocumentIdentity::new(1)).unwrap();
        assert!(snap.nodes.iter().count() > 1);
    }

    #[test]
    fn populates_provenance_for_rect() {
        let src = "(page a4 (rect 10 20 30 40))";
        let snap = document_snapshot_from_source(src, DocumentIdentity::new(2)).unwrap();
        let rect = snap
            .nodes
            .iter()
            .find(|n| matches!(n.kind, reciplexa_document::DocumentNodeKind::Rectangle))
            .unwrap();
        let prov = snap.provenance.get(rect.id).unwrap();
        assert!(!prov.text_range.is_empty());
    }
}
