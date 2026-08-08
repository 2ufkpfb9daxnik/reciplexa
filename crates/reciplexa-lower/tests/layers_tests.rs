use reciplexa_lower::*;


const PAGE: &str = "(page a4 ";

fn page(body: &str) -> String {
    format!("{PAGE}{body})")
}

// --- equivalence: labels and collection ---

#[test]
fn layer_labels_fallback_without_string() {
    let src = page("(text 1 2 3) (image)");
    let layers = collect_layers_page(&src, 0).unwrap();
    assert_eq!(layers[0].label, "text");
    assert_eq!(layers[1].label, "image");
}

#[test]
fn collect_layers_through_transforms_and_unknown() {
    let src = page(
        "(scale 2 (rotate 90 (group (circle 1 2 3) (rect 0 0 1 1)))) (scale 1 2 (ellipse 1 1 2 2)) (unknown 0)",
    );
    let layers = collect_layers_page(&src, 0).unwrap();
    assert_eq!(layers.len(), 3);
    assert_eq!(layers[0].kind, "circle");
    assert_eq!(layers[1].kind, "rect");
    assert_eq!(layers[2].kind, "ellipse");
    assert_eq!(layers[0].root_start, layers[1].root_start);
}

// --- boundary: reorder / rewrite ---

#[test]
fn reorder_same_root_siblings() {
    let src = page("(group (circle 1 2 3) (rect 0 0 1 1))");
    let out = reorder_layer_page(&src, 0, 0, 1).unwrap();
    let layers = collect_layers_page(&out, 0).unwrap();
    assert_eq!(layers[0].kind, "rect");
    assert_eq!(layers[1].kind, "circle");
}

#[test]
fn reorder_distinct_roots_noop_when_same_index() {
    let src = page("(circle 1 2 3) (rect 0 0 1 1)");
    assert_eq!(reorder_layer_page(&src, 0, 0, 0).unwrap(), src);
}

// --- median: delete / duplicate padding ---

#[test]
fn delete_sole_root_removes_whole_form() {
    let src = "(page a4\n  (circle 1 2 3)\n  (rect 0 0 1 1)\n)";
    let out = delete_layer_page(src, 0, 0).unwrap();
    assert!(!out.contains("circle"));
    assert!(out.contains("rect"));
}

#[test]
fn duplicate_inline_without_leading_newline() {
    let src = page("(circle 1 2 3)(rect 0 0 1 1)");
    let out = duplicate_layer_page(&src, 0, 0).unwrap();
    assert_eq!(collect_layers_page(&out, 0).unwrap().len(), 3);
    assert!(out.contains("(circle 1 2 3) (circle 1 2 3)"));
}

#[test]
fn insert_on_single_line_page_uses_space_pad() {
    let src = "(page a4 (circle 1 2 3))";
    let (out, idx) = insert_layer_page(src, 0, "(rect 0 0 1 1)").unwrap();
    assert!(out.contains("(circle 1 2 3) (rect 0 0 1 1)"));
    assert_eq!(idx, 1);
}

// --- boundary: group / ungroup ---

#[test]
fn group_dedupes_same_index() {
    let src = page("(circle 1 2 3) (rect 0 0 10 10)");
    let err = group_layers_page(&src, 0, &[0, 0]).unwrap_err();
    assert!(err.message.contains("two"));
}

#[test]
fn group_same_root_twice_errors() {
    let src = page("(translate 0 0 (circle 1 2 3) (rect 0 0 1 1))");
    let err = group_layers_page(&src, 0, &[0, 1]).unwrap_err();
    assert!(err.message.contains("two page-level"));
}

#[test]
fn ungroup_empty_group_errors() {
    let src = page("(group)");
    let err = ungroup_layer_page(&src, 0, 0).unwrap_err();
    assert!(
        err.message.contains("no children")
            || err.message.contains("group")
            || err.message.contains("out of range")
    );
}

#[test]
fn ungroup_returns_selection_spanning_children() {
    let src = page("(group (circle 1 2 3) (rect 0 0 1 1))");
    let (out, sel) = ungroup_layer_page(&src, 0, 0).unwrap();
    assert!(!out.contains("(group"));
    assert_eq!(sel, vec![0, 1]);
}

// --- defect: validation ---

#[test]
fn insert_layer_page_index_out_of_range() {
    let src = page("(circle 1 2 3)");
    assert!(collect_layers_page(&src, 9).is_err());
}

#[test]
fn delete_layer_shared_span_cuts_leaf_only() {
    let src = page("(translate 0 0 (circle 1 2 3) (rect 0 0 1 1))");
    let out = delete_layer_page(&src, 0, 1).unwrap();
    assert!(out.contains("circle"));
    assert!(!out.contains("rect"));
}

#[test]
fn duplicate_shared_root_copies_leaf_span() {
    let src = page("(translate 0 0 (circle 1 2 3) (rect 0 0 1 1))");
    let out = duplicate_layer_page(&src, 0, 1).unwrap();
    assert_eq!(collect_layers_page(&out, 0).unwrap().len(), 3);
}
