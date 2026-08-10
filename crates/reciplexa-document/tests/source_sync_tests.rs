use reciplexa_document::*;
use reciplexa_identity::document::DocumentIdentity;
use reciplexa_identity::package::ModuleId;
use reciplexa_scene::{Color, Page, PaperSize, Rect, Shape};
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;
use reciplexa_source::resource::SourceResourceId;
use reciplexa_syntax::parse_source;

fn rect_page() -> Page {
    Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 10.0,
            y_mm: 20.0,
            width_mm: 30.0,
            height_mm: 40.0,
            fill: Color::BLACK,
        })],
    }
}

fn snap_with_rect_layer(
    byte_start: usize,
    byte_end: usize,
) -> (DocumentSnapshot, reciplexa_identity::document::StableNodeId) {
    let page = rect_page();
    let layers = vec![LayerSpan {
        byte_start,
        byte_end,
        label: "rect".into(),
    }];
    let snap = document_from_scene_page_with_layers(
        DocumentIdentity::new(1),
        &page,
        &layers,
        SourceResourceId::new(1),
        &[],
    );
    let node = drawable_node_ids(&snap.nodes)[0];
    (snap, node)
}

#[test]
fn missing_layout_with_provenance_is_unknown_node() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(20));
    let root = snap.nodes.root_id().unwrap();
    let text = snap
        .nodes
        .insert_child(root, DocumentNodeKind::Text)
        .unwrap();
    snap.provenance.insert(
        text,
        SourceProvenance {
            source_resource_id: SourceResourceId::new(1),
            module_id: ModuleId::new(1),
            text_range: TextRange::EMPTY,
            syntax_node_id: None,
        },
    );
    let err = apply_provenance_edit(
        &mut snap,
        "(page a4)",
        ApplyEdit::Move {
            node: text,
            x: 1.0,
            y: 2.0,
        },
    )
    .unwrap_err();
    assert!(matches!(err, TransactionError::UnknownNode(_)));

    let err = apply_provenance_edit(
        &mut snap,
        "(page a4)",
        ApplyEdit::Resize {
            node: text,
            width: 3.0,
            height: 4.0,
        },
    )
    .unwrap_err();
    assert!(matches!(err, TransactionError::UnknownNode(_)));
}

#[test]
fn set_text_apply_fails_when_node_missing_from_store() {
    let mut snap = DocumentSnapshot::new(DocumentIdentity::new(21));
    let ghost = reciplexa_identity::document::StableNodeId::new(777);
    snap.provenance.insert(
        ghost,
        SourceProvenance {
            source_resource_id: SourceResourceId::new(1),
            module_id: ModuleId::new(1),
            text_range: TextRange::EMPTY,
            syntax_node_id: None,
        },
    );
    let err = apply_provenance_edit(
        &mut snap,
        "(page a4)",
        ApplyEdit::SetText {
            node: ghost,
            text: "x".into(),
        },
    )
    .unwrap_err();
    assert!(matches!(err, TransactionError::UnknownNode(_)));
}

#[test]
fn move_without_provenance_updates_document_only() {
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
        document_from_scene_page(DocumentIdentity::new(1), &page, SourceResourceId::new(1));
    let rect = snap
        .nodes
        .iter()
        .find(|n| matches!(n.kind, DocumentNodeKind::Rectangle))
        .unwrap()
        .id;
    let outcome = apply_provenance_edit(
        &mut snap,
        "(page a4 (rect 1 2 3 4))",
        ApplyEdit::Move {
            node: rect,
            x: 5.0,
            y: 6.0,
        },
    )
    .unwrap();
    assert!(matches!(outcome, SourceSyncOutcome::DocumentOnly(_)));
    let layout = snap.nodes.get(rect).unwrap().layout().unwrap();
    assert_eq!(layout.x, 5.0);
    assert_eq!(layout.y, 6.0);
}

#[test]
fn move_with_provenance_updates_source_literals() {
    let src = "(page a4 (rect 10 20 30 40))";
    let (mut snap, node) = snap_with_rect_layer(15, 20);
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
    match outcome {
        SourceSyncOutcome::SourceUpdated { new_source, .. } => {
            assert!(new_source.contains("50"));
            assert!(new_source.contains("60"));
            assert!(parse_source(&new_source).errors.is_empty());
        }
        other => panic!("expected SourceUpdated, got {other:?}"),
    }
    let layout = snap.nodes.get(node).unwrap().layout().unwrap();
    assert_eq!(layout.x, 50.0);
    assert_eq!(layout.y, 60.0);
}

#[test]
fn non_invertible_expression_keeps_document_only() {
    let src = "(page a4 (rect 10 20 30 40))";
    let (mut snap, node) = snap_with_rect_layer(15, 20);
    // Provenance range extends past source end → non-invertible literal rewrite.
    snap.provenance.insert(
        node,
        crate::provenance::SourceProvenance {
            source_resource_id: SourceResourceId::new(1),
            module_id: ModuleId::new(1),
            text_range: TextRange::try_new(ByteOffset::new(15), ByteOffset::new(200)).unwrap(),
            syntax_node_id: None,
        },
    );
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
    assert!(matches!(outcome, SourceSyncOutcome::DocumentOnly(_)));
    let layout = snap.nodes.get(node).unwrap().layout().unwrap();
    assert_eq!(layout.x, 50.0);
    assert_eq!(layout.y, 60.0);
}

#[test]
fn parse_error_blocks_source_sync() {
    let broken_src = "(page a4 (rect 10 20 30 40)";
    let (mut snap, node) = snap_with_rect_layer(15, 20);
    let outcome = apply_provenance_edit(
        &mut snap,
        broken_src,
        ApplyEdit::Move {
            node,
            x: 50.0,
            y: 60.0,
        },
    )
    .unwrap();
    assert!(matches!(
        outcome,
        SourceSyncOutcome::Blocked(SourceSyncBlockReason::ParseError)
    ));
}

#[test]
fn resize_with_provenance_updates_width_height_literals() {
    let src = "(page a4 (rect 10 20 30 40))";
    let (mut snap, node) = snap_with_rect_layer(21, 26);
    let outcome = apply_provenance_edit(
        &mut snap,
        src,
        ApplyEdit::Resize {
            node,
            width: 99.0,
            height: 88.0,
        },
    )
    .unwrap();
    match outcome {
        SourceSyncOutcome::SourceUpdated { new_source, .. } => {
            assert!(new_source.contains("99"));
            assert!(new_source.contains("88"));
        }
        other => panic!("expected SourceUpdated, got {other:?}"),
    }
}

#[test]
fn set_text_with_provenance_escapes_quotes() {
    let text_src = r#"(page a4 (text 1 2 12 "old"))"#;
    let start = text_src.find("\"old\"").unwrap();
    let end = start + "\"old\"".len();
    let page = Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(reciplexa_scene::Text {
            x_mm: 1.0,
            y_mm: 2.0,
            size_mm: 12.0,
            width_mm: None,
            height_mm: None,
            content: "old".into(),
            fill: Color::BLACK,
        })],
    };
    let layers = vec![LayerSpan {
        byte_start: start,
        byte_end: end,
        label: "text".into(),
    }];
    let mut snap = document_from_scene_page_with_layers(
        DocumentIdentity::new(9),
        &page,
        &layers,
        SourceResourceId::new(1),
        &[],
    );
    let node = drawable_node_ids(&snap.nodes)[0];
    let outcome = apply_provenance_edit(
        &mut snap,
        text_src,
        ApplyEdit::SetText {
            node,
            text: "say \"hi\"".into(),
        },
    )
    .unwrap();
    match outcome {
        SourceSyncOutcome::SourceUpdated { new_source, .. } => {
            assert!(
                new_source.contains("say \"hi\""),
                "expected quoted content preserved, got {new_source}"
            );
            assert!(
                new_source.contains("\"\"\""),
                "expected multi-quote delimiter, got {new_source}"
            );
        }
        other => panic!("expected SourceUpdated, got {other:?}"),
    }
}

#[test]
fn unknown_node_errors_before_sync() {
    let page = rect_page();
    let mut snap =
        document_from_scene_page(DocumentIdentity::new(10), &page, SourceResourceId::new(1));
    let missing = reciplexa_identity::document::StableNodeId::new(9999);
    let err = apply_provenance_edit(
        &mut snap,
        "(page a4 (rect 1 2 3 4))",
        ApplyEdit::Move {
            node: missing,
            x: 1.0,
            y: 2.0,
        },
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("UnknownNode"));
}

#[test]
fn set_text_escapes_backslash_and_rejects_broken_span() {
    let text_src = r#"(page a4 (text 1 2 12 "old"))"#;
    let start = text_src.find("\"old\"").unwrap();
    let end = start + "\"old\"".len();
    let page = Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(reciplexa_scene::Text {
            x_mm: 1.0,
            y_mm: 2.0,
            size_mm: 12.0,
            width_mm: None,
            height_mm: None,
            content: "old".into(),
            fill: Color::BLACK,
        })],
    };
    let layers = vec![LayerSpan {
        byte_start: start,
        byte_end: end,
        label: "text".into(),
    }];
    let mut snap = document_from_scene_page_with_layers(
        DocumentIdentity::new(11),
        &page,
        &layers,
        SourceResourceId::new(1),
        &[],
    );
    let node = drawable_node_ids(&snap.nodes)[0];
    let outcome = apply_provenance_edit(
        &mut snap,
        text_src,
        ApplyEdit::SetText {
            node,
            text: "a\\b".into(),
        },
    )
    .unwrap();
    match outcome {
        SourceSyncOutcome::SourceUpdated { new_source, .. } => {
            // SYN §8.1: backslash is a normal character inside short strings.
            assert!(
                new_source.contains(r#""a\b""#),
                "got {new_source}"
            );
        }
        other => panic!("expected SourceUpdated, got {other:?}"),
    }

    if let Some(p) = snap.provenance.by_node.get_mut(&node) {
        p.text_range = reciplexa_source::range::TextRange::try_new(
            reciplexa_source::offset::ByteOffset::new(start as u32 + 1),
            reciplexa_source::offset::ByteOffset::new(end as u32),
        )
        .unwrap();
    }
    let blocked = apply_provenance_edit(
        &mut snap,
        text_src,
        ApplyEdit::SetText {
            node,
            text: "x".into(),
        },
    )
    .unwrap();
    assert!(matches!(
        blocked,
        SourceSyncOutcome::DocumentOnly(_) | SourceSyncOutcome::Blocked(_)
    ));
}
