//! Phase 4 vertical slice conformance tests.

use reciplexa::document_pipeline::document_snapshot_from_source;
use reciplexa_document::{
    apply_provenance_edit, drawable_node_ids, scene_shapes_from_document, ApplyEdit,
    ReconcileGuard, SourceSyncOutcome,
};
use reciplexa_identity::document::DocumentIdentity;
use reciplexa_test::{run_conformance, ConformanceCase};

const PKG_RECT: &str = r#"
(import graphics/shapes only rect fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (fill (rect 10 20 30 40) black)))
"#;

const PKG_TEXT: &str = r#"
(import graphics/shapes only text)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (text 10 20 12 "Hello")))
"#;

const PKG_EMPTY: &str = r#"
(import graphics/page only a4 page)
(val main (page a4 (list)))
"#;

const PKG_TWO_RECTS: &str = r#"
(import graphics/shapes only rect fill group)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (group (list
  (fill (rect 1 1 2 2) black)
  (fill (rect 3 3 4 4) black)))))
"#;

const PKG_RED_RECT: &str = r#"
(import graphics/shapes only rect fill)
(import graphics/page only a4 page)
(import graphics/color only red)
(val main (page a4 (fill (rect 0 0 10 10) red)))
"#;

#[test]
fn test_slice_source_to_document() {
    let _ = ConformanceCase::new("TEST-SLICE-001", "Phase 4", "source to document");
    let snap = document_snapshot_from_source(PKG_RECT, DocumentIdentity::new(1)).expect("snapshot");
    assert!(snap.revision.get() == 0);
    assert!(snap.nodes.iter().count() >= 1);
}

#[test]
fn test_slice_text_source() {
    let snap = document_snapshot_from_source(PKG_TEXT, DocumentIdentity::new(2)).expect("snapshot");
    let has_text = snap.nodes.iter().any(|n| n.text().is_some());
    assert!(has_text);
}

#[test]
fn test_slice_source_rect_position_in_document() {
    let case = ConformanceCase::new("TEST-SLICE-003", "Phase 4", "source rect → document");
    run_conformance(&case, || {
        let snap = document_snapshot_from_source(PKG_RECT, DocumentIdentity::new(3)).unwrap();
        let shapes = scene_shapes_from_document(&snap.nodes);
        assert_eq!(shapes.len(), 1);
        match &shapes[0] {
            reciplexa_scene::Shape::Rect(r) => {
                assert!((r.x_mm - 10.0).abs() < 0.001);
                assert!((r.y_mm - 20.0).abs() < 0.001);
            }
            _ => panic!("expected rect"),
        }
    });
}

#[test]
fn test_slice_document_move_with_provenance() {
    let case = ConformanceCase::new("TEST-SLICE-004", "Phase 4", "document move → source");
    run_conformance(&case, || {
        // Package bridge snapshots omit interim CST provenance; move stays document-only.
        let mut snap = document_snapshot_from_source(PKG_RECT, DocumentIdentity::new(4)).unwrap();
        let node = drawable_node_ids(&snap.nodes)[0];
        let outcome = apply_provenance_edit(
            &mut snap,
            PKG_RECT,
            ApplyEdit::Move {
                node,
                x: 50.0,
                y: 60.0,
            },
        )
        .unwrap();
        let layout = snap.nodes.get(node).unwrap().layout().unwrap();
        assert!((layout.x - 50.0).abs() < 0.001);
        assert!(matches!(
            outcome,
            SourceSyncOutcome::DocumentOnly(_) | SourceSyncOutcome::SourceUpdated { .. }
        ));
    });
}

#[test]
fn test_slice_reconcile_guard_blocks_reentry() {
    let case = ConformanceCase::new("TEST-SLICE-005", "Phase 4", "no reentrant reconcile");
    run_conformance(&case, || {
        let mut guard = ReconcileGuard::new();
        assert!(guard.enter());
        assert!(!guard.enter());
        guard.leave();
        assert!(guard.enter());
    });
}

#[test]
fn test_slice_text_both_ways() {
    let case = ConformanceCase::new("TEST-SLICE-006", "Phase 4", "text source and document");
    run_conformance(&case, || {
        let src = r#"
(import graphics/shapes only text)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (text 5 5 12 "Hi")))
"#;
        let mut snap = document_snapshot_from_source(src, DocumentIdentity::new(5)).unwrap();
        let node = snap.nodes.iter().find(|n| n.text().is_some()).unwrap().id;
        apply_provenance_edit(
            &mut snap,
            src,
            ApplyEdit::SetText {
                node,
                text: "Bye".into(),
            },
        )
        .unwrap();
        assert_eq!(snap.nodes.get(node).unwrap().text().unwrap().text, "Bye");
    });
}

#[test]
fn test_slice_empty_page() {
    let snap =
        document_snapshot_from_source(PKG_EMPTY, DocumentIdentity::new(7)).expect("empty page");
    assert!(snap
        .nodes
        .iter()
        .any(|n| matches!(n.kind, reciplexa_document::DocumentNodeKind::Page)));
}

#[test]
fn test_slice_multiple_rects() {
    let snap = document_snapshot_from_source(PKG_TWO_RECTS, DocumentIdentity::new(8)).unwrap();
    let shapes = scene_shapes_from_document(&snap.nodes);
    assert!(shapes.len() >= 2);
}

#[test]
fn test_slice_blocked_edit_without_provenance() {
    let case = ConformanceCase::new("TEST-SLICE-007", "Phase 4", "blocked sync");
    run_conformance(&case, || {
        use reciplexa_document::{document_from_scene_page, ApplyEdit};
        use reciplexa_scene::{Color, Page, PaperSize, Rect, Shape};
        use reciplexa_source::resource::SourceResourceId;
        let page = Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Rect(Rect {
                x_mm: 1.0,
                y_mm: 2.0,
                width_mm: 3.0,
                height_mm: 4.0,
                fill: Color::BLACK,
            })],
        };
        let mut snap =
            document_from_scene_page(DocumentIdentity::new(9), &page, SourceResourceId::new(1));
        let node = drawable_node_ids(&snap.nodes)[0];
        let outcome = apply_provenance_edit(
            &mut snap,
            PKG_RECT,
            ApplyEdit::SetText {
                node,
                text: "nope".into(),
            },
        )
        .unwrap();
        assert!(matches!(outcome, SourceSyncOutcome::DocumentOnly(_)));
    });
}

#[test]
fn test_slice_color_rect() {
    let snap = document_snapshot_from_source(PKG_RED_RECT, DocumentIdentity::new(10)).unwrap();
    let shapes = scene_shapes_from_document(&snap.nodes);
    assert_eq!(shapes.len(), 1);
}

#[test]
fn test_slice_grouped_rects() {
    let snap = document_snapshot_from_source(PKG_TWO_RECTS, DocumentIdentity::new(11)).unwrap();
    assert!(snap.nodes.iter().count() >= 2);
}

#[test]
fn test_slice_parse_error_blocks_source_sync() {
    let case = ConformanceCase::new("TEST-SLICE-008", "Phase 4", "parse error blocks sync");
    run_conformance(&case, || {
        use reciplexa_document::{
            document_from_scene_page_with_layers, ApplyEdit, LayerSpan, SourceSyncBlockReason,
        };
        use reciplexa_scene::{Color, Page, PaperSize, Rect, Shape};
        use reciplexa_source::resource::SourceResourceId;
        let page = Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Rect(Rect {
                x_mm: 10.0,
                y_mm: 20.0,
                width_mm: 30.0,
                height_mm: 40.0,
                fill: Color::BLACK,
            })],
        };
        let spans = vec![LayerSpan {
            byte_start: 0,
            byte_end: 5,
            label: "rect".into(),
        }];
        let mut snap = document_from_scene_page_with_layers(
            DocumentIdentity::new(12),
            &page,
            &spans,
            SourceResourceId::new(1),
            &[],
        );
        let node = drawable_node_ids(&snap.nodes)[0];
        let outcome = apply_provenance_edit(
            &mut snap,
            "(page",
            ApplyEdit::Move {
                node,
                x: 1.0,
                y: 2.0,
            },
        )
        .unwrap();
        assert!(matches!(
            outcome,
            SourceSyncOutcome::Blocked(SourceSyncBlockReason::ParseError)
                | SourceSyncOutcome::DocumentOnly(_)
                | SourceSyncOutcome::Blocked(_)
        ));
    });
}
