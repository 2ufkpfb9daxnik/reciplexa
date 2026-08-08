//! Phase 4 vertical slice conformance tests.

use reciplexa::document_pipeline::document_snapshot_from_source;
use reciplexa_document::{
    apply_provenance_edit, drawable_node_ids, scene_shapes_from_document, ApplyEdit,
    ReconcileGuard, SourceSyncOutcome,
};
use reciplexa_identity::document::DocumentIdentity;
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn test_slice_source_to_document() {
    let _ = ConformanceCase::new("TEST-SLICE-001", "Phase 4", "source to document");
    let snap =
        document_snapshot_from_source("(page a4 (rect 10 20 30 40))", DocumentIdentity::new(1))
            .expect("snapshot");
    assert!(snap.revision.get() == 0);
    assert!(snap.nodes.iter().count() >= 3);
}

#[test]
fn test_slice_text_source() {
    let snap = document_snapshot_from_source(
        r#"(page a4 (text 10 20 12 "Hello"))"#,
        DocumentIdentity::new(2),
    )
    .expect("snapshot");
    let has_text = snap.nodes.iter().any(|n| n.text().is_some());
    assert!(has_text);
}

#[test]
fn test_slice_source_rect_position_in_document() {
    let case = ConformanceCase::new("TEST-SLICE-003", "Phase 4", "source rect → document");
    run_conformance(&case, || {
        let snap =
            document_snapshot_from_source("(page a4 (rect 10 20 30 40))", DocumentIdentity::new(3))
                .unwrap();
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
        let src = "(page a4 (rect 10 20 30 40))";
        let mut snap = document_snapshot_from_source(src, DocumentIdentity::new(4)).unwrap();
        let node = drawable_node_ids(&snap.nodes)[0];
        let prov = snap.provenance.get(node).unwrap();
        assert!(!prov.text_range.is_empty());
        let outcome = apply_provenance_edit(
            &mut snap,
            src,
            ApplyEdit::Move {
                node,
                x: 50.0,
                y: 60.0,
            },
        )
        .unwrap();
        let layout = snap.nodes.get(node).unwrap().layout().unwrap();
        assert!((layout.x - 50.0).abs() < 0.001);
        match outcome {
            SourceSyncOutcome::SourceUpdated { new_source, .. } => {
                assert!(new_source.contains("50"));
            }
            SourceSyncOutcome::DocumentOnly(_) => {}
            SourceSyncOutcome::Blocked(_) => panic!("unexpected block"),
        }
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
        let src = r#"(page a4 (text 5 5 12 "Hi"))"#;
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
