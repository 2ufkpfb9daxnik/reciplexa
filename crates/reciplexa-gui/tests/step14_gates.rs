//! Step 14 gates: slice 1 hierarchical layers; slice 2 pagebreak / 版面; slice 3 flow blocks.

use reciplexa::pipeline::document_from_source;
use reciplexa_gui::canvas_sync::{nudge_authoring_layers, resolve_preview_layers};
use reciplexa_lower::{
    attach_glyph_children, collect_layers_authoring, collect_layers_document,
    collect_layers_vertical_demo, is_document_page_authoring, is_vertical_demo_authoring,
    parent_layer_index, shape_indices_for_authoring,
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

#[test]
fn step14_slice1_vertical_nudge_is_visible_and_ingestible() {
    use reciplexa::pipeline::expand;
    use reciplexa_gui::canvas_sync::authoring_layers_align;
    use reciplexa_lower::{
        collect_layer_props, set_layer_prop, vertical_demo_nudge_span, PropEditContext, PropValue,
    };

    let src = include_str!("../../../examples/pkg_vert.rpx");
    let expanded = expand(src).expect("expand");
    assert!(authoring_layers_align(src, &expanded, 0).unwrap());
    let layers = collect_layers_vertical_demo(src, 0).unwrap();
    let heading = layers.iter().position(|l| l.kind == "doc-heading").unwrap();
    let sample = layers
        .iter()
        .position(|l| l.kind == "vert-sample" && l.text_content.as_deref() == Some("縦書き"))
        .unwrap();
    let out = nudge_authoring_layers(src, &expanded, 0, &[heading, sample], 3.0, -1.0).unwrap();
    assert!(out.contains("(nudge (list"), "{out}");
    let samples_at = out.find("(samples samples)").unwrap();
    let nudge_at = out.find("(nudge (list").unwrap();
    assert!(nudge_at > samples_at);
    assert!(vertical_demo_nudge_span(&out).is_some());
    document_from_source(&out).expect("ingest after nudge");

    let ctx = PropEditContext {
        aabb_mm: (10.0, 20.0, 18.0, 80.0),
        paper_w_mm: 210.0,
        paper_h_mm: 297.0,
    };
    let props = collect_layer_props(src, 0, sample, &ctx).unwrap();
    assert!(props.iter().any(|p| p.id == "layout.x"));
    let via_props =
        set_layer_prop(src, 0, sample, "layout.x", &PropValue::Number(15.0), &ctx).unwrap();
    assert!(via_props.contains("(nudge (list"), "{via_props}");
}

#[test]
fn step14_slice1_canvas_union_grabs_text_box() {
    use reciplexa_gui::canvas_sync::{hit_test_authoring_parent, point_in_aabb, union_bounds_mm};
    use reciplexa_view::hit_test_shapes;

    let src = include_str!("../../../examples/pkg_vert.rpx");
    let doc = document_from_source(src).expect("ingest");
    let (_, shapes) = flatten_page(&doc, 0).expect("page");
    let layers = resolve_preview_layers(src, 0, &shapes);
    let abc = layers
        .iter()
        .find(|l| l.kind == "vert-sample" && l.text_content.as_deref() == Some("ABC"))
        .expect("ABC");
    let idxs = shape_indices_for_authoring(&layers, &[abc.authoring_index]);
    let b = union_bounds_mm(&shapes, &idxs).expect("union");
    let parent_row = layers
        .iter()
        .position(|l| !l.glyph_child && l.authoring_index == abc.authoring_index)
        .unwrap();
    let mut at = ((b.0 + b.2) * 0.5, (b.1 + b.3) * 0.5);
    for i in 0..=8 {
        let t = f64::from(i) / 8.0;
        let x = (b.0 + b.2) * 0.5;
        let y = b.1 + (b.3 - b.1) * t;
        if point_in_aabb(x, y, b) && hit_test_shapes(&shapes, x, y).is_none() {
            at = (x, y);
            break;
        }
    }
    assert_eq!(
        hit_test_authoring_parent(&layers, &shapes, at.0, at.1),
        Some(parent_row)
    );
}

#[test]
fn step14_slice2_pagebreak_starts_second_package_page() {
    let src = include_str!("../../../examples/pkg_pagebreak.rpx");
    let doc = document_from_source(src).expect("ingest");
    assert_eq!(
        doc.pages.len(),
        2,
        "explicit pagebreak must emit two scene pages"
    );
    let page_text = |i: usize| {
        doc.pages[i]
            .shapes
            .iter()
            .filter_map(reciplexa_scene::Shape::text_content)
            .collect::<String>()
    };
    let t0 = page_text(0);
    let t1 = page_text(1);
    assert!(t0.contains("First") || t0.contains("page"), "page0={t0}");
    assert!(t1.contains("Second") || t1.contains("hanmen"), "page1={t1}");
    assert!(flatten_page(&doc, 0).is_some());
    assert!(flatten_page(&doc, 1).is_some());
    assert!(is_document_page_authoring(src));
    let layers = collect_layers_document(src, 1).expect("scene page 1 uses CST page 0");
    assert!(layers.iter().any(|l| l.kind == "doc-heading"));
    assert!(layers.iter().any(|l| l.kind == "doc-paragraph"));
}

#[test]
fn step14_slice2_hanmen_shared_by_preview_and_export() {
    let src = include_str!("../../../examples/pkg_pagebreak.rpx");
    let doc = document_from_source(src).expect("ingest");
    let left = 48.0;
    let top = 36.0;
    for page in &doc.pages {
        for shape in &page.shapes {
            let x = shape.text_x_mm().expect("x");
            let y = shape.text_y_mm().expect("y");
            assert!(
                x + 1e-6 >= left,
                "shape x={x} must sit in hanmen left={left}"
            );
            assert!(
                y <= page.paper.height_mm - top + 1e-6,
                "shape y={y} must sit on/below top margin"
            );
        }
    }
    let export_x = doc.pages[0].shapes[0].text_x_mm().expect("export x");
    let (_, shapes) = flatten_page(&doc, 0).expect("flatten");
    let preview_x = match &shapes[0] {
        reciplexa_view::WorldShape::Text(t) => t.x_mm,
        other => panic!("expected text, got {other:?}"),
    };
    assert!(
        (export_x - preview_x).abs() < 1e-9,
        "preview x={preview_x} export x={export_x}"
    );
}

#[test]
fn step14_slice3_flow_blocks_preview_and_export() {
    reciplexa::run_on_host_stack("step14-slice3", || {
        let src = include_str!("../../../examples/pkg_flow_blocks.rpx");
        let doc = document_from_source(src).expect("ingest");
        assert_eq!(doc.pages.len(), 1);
        let joined = doc.pages[0]
            .shapes
            .iter()
            .filter_map(reciplexa_scene::Shape::text_content)
            .collect::<String>();
        assert!(
            joined.contains("Flow") || joined.contains("blocks"),
            "{joined}"
        );
        assert!(
            joined.contains("First") && joined.contains("Second"),
            "{joined}"
        );
        assert!(
            joined.contains("Step") && joined.contains("one"),
            "{joined}"
        );
        assert!(joined.contains("Note:"), "{joined}");
        assert!(joined.contains("Diagram"), "{joined}");
        assert!(joined.contains("Caption:"), "{joined}");
        assert!(
            joined.contains("Name") && joined.contains("Qty"),
            "{joined}"
        );
        assert!(joined.contains('A') && joined.contains('B'), "{joined}");
        assert!(
            doc.pages[0].shapes.iter().any(|s| matches!(
                s,
                reciplexa_scene::Shape::Frame(_) | reciplexa_scene::Shape::Rect(_)
            )),
            "figure placeholder box"
        );
        assert!(
            doc.pages[0]
                .shapes
                .iter()
                .any(|s| matches!(s, reciplexa_scene::Shape::Line(_))),
            "table grid rules"
        );
        let figure_bottom = doc.pages[0]
            .shapes
            .iter()
            .find_map(|s| match s {
                reciplexa_scene::Shape::Rect(r) => Some(r.y_mm),
                _ => None,
            })
            .expect("figure fill");
        let caption_y = doc.pages[0]
            .shapes
            .iter()
            .filter_map(|s| {
                let y = s.text_y_mm()?;
                (y < figure_bottom - 1e-6).then_some(y)
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .expect("text below figure box");
        assert!(
            caption_y + 3.5 <= figure_bottom + 1e-6,
            "caption y={caption_y} must sit below figure box bottom {figure_bottom}"
        );
        let left = 24.0;
        let top = 28.0;
        for shape in &doc.pages[0].shapes {
            if let Some(x) = shape.text_x_mm() {
                assert!(x + 1e-6 >= left, "x={x}");
            }
            if let Some(y) = shape.text_y_mm() {
                assert!(
                    y <= doc.pages[0].paper.height_mm - top + 1e-6,
                    "y={y} must sit on/below top margin"
                );
            }
        }
        let export_x = doc.pages[0]
            .shapes
            .iter()
            .find_map(reciplexa_scene::Shape::text_x_mm)
            .expect("export x");
        let (_, shapes) = flatten_page(&doc, 0).expect("flatten");
        let preview_x = shapes
            .iter()
            .find_map(|s| match s {
                reciplexa_view::WorldShape::Text(t) => Some(t.x_mm),
                _ => None,
            })
            .expect("preview text");
        assert!(
            (export_x - preview_x).abs() < 1e-9,
            "preview x={preview_x} export x={export_x}"
        );
        assert!(is_document_page_authoring(src));
    })
    .expect("join");
}
