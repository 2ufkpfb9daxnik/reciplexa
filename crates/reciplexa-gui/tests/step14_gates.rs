//! Step 14 slice 1: hierarchical layers (authoring parent + GlyphRun children).

use reciplexa::pipeline::document_from_source;
use reciplexa_gui::canvas_sync::{nudge_authoring_layers, resolve_preview_layers};
use reciplexa_lower::{
    attach_glyph_children, collect_layers_authoring, collect_layers_vertical_demo,
    is_vertical_demo_authoring, parent_layer_index, shape_indices_for_authoring,
};
use reciplexa_view::flatten_page;

#[test]
fn step14_slice1_text_line_parent_move_keeps_cluster() {
    let src = include_str!("../../../examples/text_line.rpx");
    let doc = document_from_source(src).expect("ingest");
    let (_, shapes) = flatten_page(&doc, 0).expect("page");
    let layers = resolve_preview_layers(src, 0, &shapes);
    assert!(layers.iter().any(|l| l.kind == "text" && !l.glyph_child));
    assert!(layers.iter().any(|l| l.kind == "glyph"));
    let text_ai = layers
        .iter()
        .find(|l| l.kind == "text")
        .unwrap()
        .authoring_index;
    let framed = shape_indices_for_authoring(&layers, &[text_ai]);
    assert_eq!(framed.len(), 1, "LTR cluster is one GlyphRun");
    let out = nudge_authoring_layers(src, src, 0, &[text_ai], 4.0, 0.0).unwrap();
    assert!(
        out.contains("(text 34 260 8") || out.contains("34 260"),
        "{out}"
    );
    let cst = collect_layers_authoring(src, 0).unwrap();
    assert!(cst.iter().all(|l| !l.glyph_child));
}

#[test]
fn step14_slice1_vertical_glyphs_are_children() {
    let src = include_str!("../../../examples/pkg_vert.rpx");
    assert!(is_vertical_demo_authoring(src));
    let parents = collect_layers_vertical_demo(src, 0).unwrap();
    assert!(parents.iter().any(|l| l.kind == "vert-ruby"));
    assert_eq!(
        parents.iter().filter(|l| l.kind == "vert-sample").count(),
        4
    );
    let doc = document_from_source(src).expect("ingest");
    let (_, shapes) = flatten_page(&doc, 0).expect("page");
    let layers = attach_glyph_children(parents, &shapes);
    assert!(layers.iter().any(|l| l.glyph_child));
    let abc = layers
        .iter()
        .find(|l| l.kind == "vert-sample" && l.text_content.as_deref() == Some("ABC"))
        .expect("ABC column");
    let kids: Vec<_> = layers
        .iter()
        .filter(|l| l.glyph_child && l.authoring_index == abc.authoring_index)
        .collect();
    assert_eq!(kids.len(), 3, "ABC is three rotated glyphs");
    if let Some(si) = kids[1].shape_index {
        assert_eq!(
            parent_layer_index(&layers, si).unwrap(),
            layers
                .iter()
                .position(|l| l.kind == "vert-sample" && l.authoring_index == abc.authoring_index)
                .unwrap()
        );
    }
}

#[test]
fn step14_slice1_math_tree_stays_parents() {
    let src = include_str!("../../../examples/pkg_live_math.rpx");
    let layers = collect_layers_authoring(src, 0).unwrap();
    assert!(layers.iter().any(|l| l.kind == "math-fraction"));
    assert!(layers.iter().any(|l| l.depth > 0));
    assert!(layers.iter().all(|l| l.kind != "glyph"));
}
