//! Markup preview: expand → package domain bridge (N5.2d).

use reciplexa::pipeline::{document_from_source, expand, wants_package_graphics_path};
use reciplexa_scene::Shape;

fn leaf_shape_count(shapes: &[Shape]) -> usize {
    shapes
        .iter()
        .map(|s| match s {
            Shape::Group { children, .. } | Shape::Opacity { children, .. } => {
                leaf_shape_count(children)
            }
            _ => 1,
        })
        .sum()
}

fn assert_markup_preview(path: &str) {
    let src = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let doc = document_from_source(&src).unwrap_or_else(|e| {
        panic!("document_from_source failed for {path}: {}", e.display());
    });
    let expanded =
        expand(&src).unwrap_or_else(|e| panic!("expand failed for {path}: {}", e.display()));
    assert!(
        wants_package_graphics_path(&expanded),
        "{path}: markup expand should emit package graphics path\n{expanded}"
    );
    assert!(
        expanded.contains("(import graphics"),
        "{path}: expected graphics package imports"
    );
    assert!(
        expanded.contains("(val main"),
        "{path}: expected package main entry"
    );
    let page = doc
        .pages
        .first()
        .unwrap_or_else(|| panic!("{path}: no pages after package bridge"));
    assert!(
        leaf_shape_count(&page.shapes) > 0,
        "{path}: expected drawable shapes after bridge"
    );
}

#[test]
fn markup_doc_preview_layers_align() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/markup_doc.rpx");
    assert_markup_preview(path);
}

#[test]
fn markup_ja_preview_layers_align() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/markup_ja.rpx");
    assert_markup_preview(path);
}
