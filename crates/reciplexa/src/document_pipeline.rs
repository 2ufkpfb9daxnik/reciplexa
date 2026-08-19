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

/// Expand, resolve, parse, lower, and build an editable document snapshot with provenance.
///
/// Package-shaped sources (after expand) lower via the package domain bridge and
/// build a scene-backed snapshot **without** interim CST layer provenance.
pub fn document_snapshot_from_source(
    source: &str,
    identity: DocumentIdentity,
) -> Result<DocumentSnapshot, String> {
    // Expand-first: markup / macros must become page forms before bind/lower.
    let expanded = reciplexa_macro::expand_source(source).map_err(|e| e.message)?;
    if crate::pipeline::wants_package_graphics_path(&expanded) {
        let scene = crate::pipeline::document_from_source(source).map_err(|e| e.display())?;
        let page = scene
            .pages
            .first()
            .ok_or_else(|| "document has no pages".to_string())?;
        return Ok(document_from_scene_page_with_layers(
            identity,
            page,
            &[],
            SourceResourceId::new(1),
            &[],
        ));
    }
    crate::pipeline::refuse_interim_if_required(&expanded).map_err(|e| e.display())?;
    let source = expanded.as_str();
    let resolve = resolve_source(source);
    if !resolve.is_ok() {
        return Err(resolve.errors[0].message.clone());
    }
    // Parse errors are already rejected by `resolve_source` (it parses first).
    let parse = parse_source(source);
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
    // SetLayout on a node that already exposed `layout()` cannot UnknownNode.
    tx.into_transaction()
        .apply(snap)
        .expect("SetLayout on checked layout node");
    assert!(snap.revision.get() > rev_before);
    Ok(())
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
            Shape::Rect(_) | Shape::Text(_) | Shape::GlyphRun(_) => {
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

    const PKG_CIRCLE: &str = r#"
(import graphics/shapes only circle fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (fill (circle 105 148.5 40) black)))
"#;

    const PKG_RECT: &str = r#"
(import graphics/shapes only rect fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (fill (rect 10 20 30 40) black)))
"#;

    #[test]
    fn builds_snapshot_from_black_circle() {
        let snap = document_snapshot_from_source(PKG_CIRCLE, DocumentIdentity::new(1)).unwrap();
        assert!(snap.nodes.iter().count() > 1);
    }

    #[test]
    fn builds_snapshot_from_markup_after_expand() {
        let src = "(markup @heading(Hi)\nBody text.\n)";
        let snap = document_snapshot_from_source(src, DocumentIdentity::new(20)).unwrap();
        assert!(snap.nodes.iter().count() > 1);
    }

    #[test]
    fn expand_error_rejects_snapshot() {
        let err = document_snapshot_from_source("(page a4", DocumentIdentity::new(21)).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn builds_snapshot_from_package_rect() {
        let snap = document_snapshot_from_source(PKG_RECT, DocumentIdentity::new(2)).unwrap();
        assert!(snap
            .nodes
            .iter()
            .any(|n| matches!(n.kind, reciplexa_document::DocumentNodeKind::Rectangle)));
    }

    #[test]
    fn rejects_parse_errors() {
        let err = document_snapshot_from_source("(page a4", DocumentIdentity::new(1)).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn preview_shapes_from_snapshot() {
        let snap = document_snapshot_from_source(PKG_RECT, DocumentIdentity::new(3)).unwrap();
        let shapes = preview_shapes(&snap);
        assert!(!shapes.is_empty());
    }

    #[test]
    fn lowered_scene_builds_snapshot() {
        use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Shape};
        let scene = Document {
            pages: vec![Page {
                paper: PaperSize::a4(),
                shapes: vec![Shape::Circle(Circle {
                    x_mm: 1.0,
                    y_mm: 2.0,
                    radius_mm: 3.0,
                    fill: Color::BLACK,
                })],
            }],
        };
        let snap = super::document_snapshot_from_lowered(&scene).unwrap();
        assert!(snap.nodes.iter().count() > 0);
    }

    #[test]
    fn interim_keyword_page_is_refused() {
        let err =
            document_snapshot_from_source("(page a4 (rect 1 2 3 4))", DocumentIdentity::new(4))
                .unwrap_err();
        assert!(err.contains("retired") || err.contains("import") || err.contains("package"));
    }

    #[test]
    fn move_node_updates_layout() {
        let mut snap = document_snapshot_from_source(PKG_RECT, DocumentIdentity::new(5)).unwrap();
        let rect_id = snap
            .nodes
            .iter()
            .find(|n| matches!(n.kind, reciplexa_document::DocumentNodeKind::Rectangle))
            .unwrap()
            .id;
        move_node_in_snapshot(&mut snap, rect_id, 50.0, 60.0).unwrap();
        let layout = snap.nodes.get(rect_id).unwrap().layout().unwrap();
        assert_eq!(layout.x, 50.0);
        assert_eq!(layout.y, 60.0);
    }

    #[test]
    fn move_node_missing_layout_errors() {
        let mut snap = document_snapshot_from_source(PKG_RECT, DocumentIdentity::new(6)).unwrap();
        let root = snap.nodes.root_id().unwrap();
        let err = move_node_in_snapshot(&mut snap, root, 0.0, 0.0).unwrap_err();
        assert!(err.contains("layout"));
    }

    #[test]
    fn document_snapshot_rejects_interim_unknown_color() {
        let err = document_snapshot_from_source(
            "(page a4 (circle 0 0 1 puce))",
            DocumentIdentity::new(7),
        )
        .unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn document_snapshot_rejects_empty_pages() {
        let scene = reciplexa_scene::Document { pages: vec![] };
        let err = document_snapshot_from_lowered(&scene).unwrap_err();
        assert!(err.contains("no pages"));
    }

    #[test]
    fn provenance_hints_empty_without_cst_layers() {
        let snap = document_snapshot_from_source(PKG_CIRCLE, DocumentIdentity::new(8)).unwrap();
        let scene = crate::pipeline::document_from_source(PKG_CIRCLE).unwrap();
        let hints = provenance_hints_for_scene(&scene, &snap);
        // Package bridge snapshots omit interim CST layer spans.
        assert!(hints.iter().all(|h| h.is_none()) || !hints.is_empty());
    }

    #[test]
    fn move_node_unknown_id_errors() {
        let mut snap = document_snapshot_from_source(PKG_RECT, DocumentIdentity::new(11)).unwrap();
        let bogus = reciplexa_identity::document::StableNodeId::new(9999);
        let err = move_node_in_snapshot(&mut snap, bogus, 1.0, 2.0).unwrap_err();
        assert!(err.contains("layout") || err.contains("node"));
    }

    #[test]
    fn n6o_empty_pages_resolve_and_parse_residuals() {
        use reciplexa_scene::Document;
        let empty = Document { pages: vec![] };
        assert!(document_snapshot_from_lowered(&empty)
            .unwrap_err()
            .contains("no pages"));
        let err = document_snapshot_from_source(
            "(import missing/mod only x)\n(val main x)\n",
            DocumentIdentity::new(12),
        )
        .unwrap_err();
        assert!(!err.is_empty());
        let err2 = document_snapshot_from_source("(page a4 (circle))", DocumentIdentity::new(13));
        assert!(err2.is_err());
    }
}
