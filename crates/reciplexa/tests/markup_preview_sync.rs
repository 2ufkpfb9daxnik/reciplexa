//! Markup preview: expand-first pipeline keeps layer sync aligned with shapes.

use reciplexa::pipeline::{document_from_source, expand};
use reciplexa_lower::collect_layers_page;
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
    let layers = collect_layers_page(&expanded, 0)
        .unwrap_or_else(|e| panic!("collect_layers_page failed for {path}: {}", e.message));
    let page = doc
        .pages
        .first()
        .unwrap_or_else(|| panic!("{path}: no pages after lower"));
    assert_eq!(
        layers.len(),
        leaf_shape_count(&page.shapes),
        "{path}: layer/shape mismatch after expand"
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
