//! Document-mediated pipeline stage (Phase 4 bridge).

use reciplexa_bind::resolve_source;
use reciplexa_document::{
    document_from_scene_page, scene_shapes_from_document, DocumentSnapshot, TransactionBuilder,
};
use reciplexa_identity::document::DocumentIdentity;
use reciplexa_lower::lower_source;
use reciplexa_source::resource::SourceResourceId;

/// Parse, lower, and build an editable document snapshot.
pub fn document_snapshot_from_source(
    source: &str,
    identity: DocumentIdentity,
) -> Result<DocumentSnapshot, String> {
    let resolve = resolve_source(source);
    if !resolve.is_ok() {
        return Err(resolve.errors[0].message.clone());
    }
    let scene = lower_source(source).map_err(|e| e.message)?;
    let page = scene
        .pages
        .first()
        .ok_or_else(|| "document has no pages".to_string())?;
    Ok(document_from_scene_page(
        identity,
        page,
        SourceResourceId::new(1),
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
    let mut tx = TransactionBuilder::new();
    tx.set_layout(
        node,
        reciplexa_document::LayoutBox::new(x, y, layout.width, layout.height),
    );
    snap.nodes
        .root_id()
        .ok_or_else(|| "missing root".to_string())?;
    match tx.into_transaction().apply(snap) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{e:?}")),
    }
}

/// Rebuild drawable shapes from the current snapshot (preview path).
pub fn preview_shapes(snap: &DocumentSnapshot) -> Vec<reciplexa_scene::Shape> {
    scene_shapes_from_document(&snap.nodes)
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
}
