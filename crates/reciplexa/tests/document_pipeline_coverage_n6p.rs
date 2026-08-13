//! Document pipeline residual tips: provenance None, empty pages, move errors.

use reciplexa::document_pipeline::{
    document_snapshot_from_lowered, document_snapshot_from_source, move_node_in_snapshot,
    preview_shapes, provenance_hints_for_scene,
};
use reciplexa_identity::document::{DocumentIdentity, StableNodeId};
use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Rect, Shape};

#[test]
fn document_pipeline_n6p_residuals() {
    let empty = Document { pages: vec![] };
    assert!(document_snapshot_from_lowered(&empty).is_err());

    let _ = document_snapshot_from_source(
        "(import nope/missing only x)\n(val main x)\n",
        DocumentIdentity::new(20),
    );
    let _ = document_snapshot_from_source("(page a4 (", DocumentIdentity::new(21));

    let src = "(page a4 (rect 1 2 3 4) (circle 5 6 7) (group (rect 8 9 1 1)))";
    let mut snap = document_snapshot_from_source(src, DocumentIdentity::new(22)).unwrap();
    let scene = reciplexa_lower::lower_source(src).unwrap();
    let _ = preview_shapes(&snap);
    let _ = provenance_hints_for_scene(&scene, &snap);

    let ids = reciplexa_document::drawable_node_ids(&snap.nodes);
    if let Some(&id) = ids.first() {
        let _ = move_node_in_snapshot(&mut snap, id, 10.0, 20.0);
    }
    let _ = move_node_in_snapshot(&mut snap, StableNodeId::new(99999), 0.0, 0.0);

    let mixed = Document {
        pages: vec![Page {
            paper: PaperSize::a4(),
            shapes: vec![
                Shape::Circle(Circle {
                    x_mm: 1.0,
                    y_mm: 2.0,
                    radius_mm: 3.0,
                    fill: Color::BLACK,
                }),
                Shape::Group {
                    transform: reciplexa_scene::Affine::identity(),
                    children: vec![Shape::Rect(Rect {
                        x_mm: 0.0,
                        y_mm: 0.0,
                        width_mm: 1.0,
                        height_mm: 1.0,
                        fill: Color::BLACK,
                    })],
                },
            ],
        }],
    };
    let snap2 =
        document_snapshot_from_source("(page a4 (rect 1 2 3 4))", DocumentIdentity::new(23))
            .unwrap();
    let _ = provenance_hints_for_scene(&mixed, &snap2);
    let _ = document_snapshot_from_lowered(&scene);
}
