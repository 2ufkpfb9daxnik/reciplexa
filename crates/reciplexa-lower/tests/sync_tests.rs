use reciplexa_lower::*;

// --- validity ---

#[test]
fn nudging_translate_preserves_layout() {
    let src = "(page a4\n  (translate  105  148.5\n    (circle 0 0 20)))\n";
    let out = nudge_first_translate(src, 10.0, -5.0).unwrap();
    assert!(out.contains("115"));
    assert!(out.contains("143.5"));
    assert!(out.contains("(page a4\n  (translate  "));
    assert!(out.contains("\n    (circle 0 0 20)))\n"));
}

#[test]
fn bindings_prefer_enclosing_translate() {
    let src = r#"
(page a4
  (rect 1 2 3 4)
  (translate 10 20 (circle 0 0 5))
  (circle 30 40 5))
"#;
    let t = collect_drag_targets(src).unwrap();
    assert_eq!(
        t,
        vec![
            DragTarget::RectXy(0),
            DragTarget::Translate(0),
            DragTarget::CircleXy(1), // second circle form in file
        ]
    );
}

#[test]
fn page_scoped_bindings_ignore_other_pages() {
    let src = "(page a4 (circle 1 2 3))\n(page a4 (rect 0 0 1 1))";
    assert_eq!(
        collect_drag_targets_page(src, 0).unwrap(),
        vec![DragTarget::CircleXy(0)]
    );
    assert_eq!(
        collect_drag_targets_page(src, 1).unwrap(),
        vec![DragTarget::RectXy(0)]
    );
}

#[test]
fn nudge_second_circle_center() {
    let src = "(page a4 (circle 1 2 3) (circle 10 20 5))";
    let out = nudge_drag_target(src, DragTarget::CircleXy(1), 1.0, 1.0).unwrap();
    assert!(out.contains("(circle 1 2 3)"));
    assert!(out.contains("(circle 11 21 5)"));
}

#[test]
fn two_translates_nudge_independently() {
    let src = "(page a4 (translate 1 2 (circle 0 0 1)) (translate 8 9 (circle 0 0 1)))";
    let out = nudge_drag_target(src, DragTarget::Translate(1), 1.0, 1.0).unwrap();
    assert!(out.contains("(translate 1 2"));
    assert!(out.contains("(translate 9 10"));
}

#[test]
fn text_and_line_bindings_nudge() {
    let src = r#"(page a4 (text 1 2 3 "Hi") (line 10 20 30 40))"#;
    let t = collect_drag_targets(src).unwrap();
    assert_eq!(t, vec![DragTarget::TextXy(0), DragTarget::LineXy(0)]);
    let out = nudge_drag_target(src, DragTarget::TextXy(0), 1.0, 1.0).unwrap();
    assert!(out.contains(r#"(text 2 3 3 "Hi")"#));
    let out = nudge_drag_target(src, DragTarget::LineXy(0), 1.0, 1.0).unwrap();
    assert!(out.contains("(line 11 21 31 41)"));
}

#[test]
fn layers_match_flatten_order_and_spans() {
    let src = r#"(page 210 297 (circle 1 2 3) (text 4 5 6 "あ"))"#;
    let layers = collect_layers_page(src, 0).unwrap();
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].kind, "circle");
    assert_eq!(layers[1].label, "text \"あ\"");
    assert!(src[layers[0].byte_start..layers[0].byte_end].contains("circle"));
    assert!(src[layers[1].byte_start..layers[1].byte_end].contains("text"));
    assert_eq!(layers[0].root_start, layers[0].byte_start);
    assert_eq!(
        collect_drag_targets_page(src, 0).unwrap().len(),
        layers.len()
    );
}

#[test]
fn reorder_layer_moves_source_order() {
    let src = "(page a4\n  (circle 1 2 3)\n  (circle 4 5 6)\n  (rect 0 0 1 1))\n";
    // Move bottom (0) to top (2).
    let out = reorder_layer_page(src, 0, 0, 2).unwrap();
    let layers = collect_layers_page(&out, 0).unwrap();
    assert_eq!(
        layers.iter().map(|l| l.kind.as_str()).collect::<Vec<_>>(),
        vec!["circle", "rect", "circle"]
    );
    assert!(out.find("(circle 1 2 3)").unwrap() > out.find("(rect 0 0 1 1)").unwrap());
}

#[test]
fn reorder_within_translate_rewrites_group_body() {
    let src = "(page a4\n  (translate 0 0\n    (circle 1 2 3)\n    (rect 0 0 1 1)))\n";
    let out = reorder_layer_page(src, 0, 0, 1).unwrap();
    let layers = collect_layers_page(&out, 0).unwrap();
    assert_eq!(layers[0].kind, "rect");
    assert_eq!(layers[1].kind, "circle");
    assert!(out.contains("(translate 0 0"));
}

// --- defect ---

#[test]
fn no_translate_errors() {
    let err = nudge_first_translate("(page a4 (circle 1 2 3))", 1.0, 1.0).unwrap_err();
    assert!(err.message.contains("translate"));
}

#[test]
fn parse_error_surfaces() {
    assert!(nudge_first_translate("(translate 1", 1.0, 1.0).is_err());
}

#[test]
fn out_of_range_target_errors() {
    let src = "(page a4 (circle 1 2 3))";
    assert!(nudge_drag_target(src, DragTarget::CircleXy(3), 1.0, 0.0).is_err());
}

#[test]
fn scale_circle_radius() {
    let src = "(page a4 (circle 10 20 5))";
    let sizes = collect_size_targets_page(src, 0).unwrap();
    assert_eq!(sizes, vec![SizeTarget::CircleR(0)]);
    let out = scale_size_target(src, SizeTarget::CircleR(0), 2.0).unwrap();
    assert!(out.contains("(circle 10 20 10)"));
}

#[test]
fn scale_rect_about_center() {
    let src = "(page a4 (rect 0 0 10 20))";
    let out = scale_size_target(src, SizeTarget::RectWh(0), 2.0).unwrap();
    // center stays at (5, 10); size 20×40 → origin (-5, -10)
    assert!(
        out.contains("(rect -5 -10 20 40)"),
        "unexpected rewrite: {out}"
    );
}

#[test]
fn scale_rect_width_only() {
    let src = "(page a4 (rect 0 0 10 20))";
    let out = scale_size_target_axes(src, SizeTarget::RectWh(0), 2.0, 1.0).unwrap();
    assert!(
        out.contains("(rect -5 0 20 20)"),
        "unexpected rewrite: {out}"
    );
}

#[test]
fn scale_ellipse_axes_independently() {
    let src = "(page a4 (ellipse 50 60 10 20))";
    let out = scale_size_target_axes(src, SizeTarget::EllipseRxRy(0), 2.0, 0.5).unwrap();
    assert!(
        out.contains("(ellipse 50 60 20 10)"),
        "unexpected rewrite: {out}"
    );
}

#[test]
fn scale_text_box_axes() {
    let src = "(page a4 (text 10 20 12 \"hello\"))";
    let out = scale_text_box(src, 0, 2.0, 0.5).unwrap();
    // Font size stays; w/h inserted and scaled about center.
    assert!(
        out.contains("(text ") && out.contains(" 12 ") && out.contains("\"hello\")"),
        "unexpected rewrite: {out}"
    );
    assert!(
        !out.contains("(text 10 20 12 \"hello\")"),
        "should insert box dims: {out}"
    );
    let sizes = collect_size_targets_page(&out, 0).unwrap();
    assert_eq!(sizes, vec![SizeTarget::TextSize(0)]);
    // Second scale edits existing w/h.
    let out2 = scale_text_box(&out, 0, 1.0, 2.0).unwrap();
    assert!(out2.contains(" 12 "), "size must stay: {out2}");
}

#[test]
fn set_text_box_inserts_and_updates() {
    let src = "(page a4 (text 10 20 12 \"hello\" blue))";
    let out = set_text_box(src, 0, 11.0, 21.0, 40.0, 15.0).unwrap();
    assert!(
        out.contains("(text 11 21 12 40 15 \"hello\" blue)"),
        "unexpected: {out}"
    );
    let out2 = set_text_box(&out, 0, 0.0, 0.0, 8.0, 9.0).unwrap();
    assert!(
        out2.contains("(text 0 0 12 8 9 \"hello\" blue)"),
        "unexpected: {out2}"
    );
}

#[test]
fn set_rect_box_xywh() {
    let src = "(page a4 (rect 10 20 30 40 red))";
    let out = set_box_xywh(src, "rect", 0, [1, 2, 3, 4], 5.0, 6.0, 7.0, 8.0).unwrap();
    assert!(out.contains("(rect 5 6 7 8 red)"), "unexpected: {out}");
}

#[test]
fn set_line_endpoint_rewrites_pair() {
    let src = "(page a4 (line 0 0 10 10 red 1))";
    let out = set_line_endpoint(src, 0, 1, 20.0, 30.0).unwrap();
    assert!(out.contains("(line 0 0 20 30 red 1)"), "unexpected: {out}");
    let out0 = set_line_endpoint(src, 0, 0, -1.0, -2.0).unwrap();
    assert!(
        out0.contains("(line -1 -2 10 10 red 1)"),
        "unexpected: {out0}"
    );
}

#[test]
fn set_polyline_vertex_rewrites_pair() {
    let src = "(page a4 (polyline 0 0 10 0 10 10 red 1))";
    let out = set_poly_vertex(src, "polyline", 0, 1, 5.0, 5.0).unwrap();
    assert!(
        out.contains("(polyline 0 0 5 5 10 10 red 1)"),
        "unexpected: {out}"
    );
}

#[test]
fn scale_under_translate_still_edits_leaf() {
    let src = "(page a4 (translate 1 2 (circle 0 0 4)))";
    let sizes = collect_size_targets_page(src, 0).unwrap();
    assert_eq!(sizes, vec![SizeTarget::CircleR(0)]);
    let out = scale_size_target(src, SizeTarget::CircleR(0), 0.5).unwrap();
    assert!(out.contains("(circle 0 0 2)"));
}

#[test]
fn bad_scale_factor_errors() {
    let src = "(page a4 (circle 1 2 3))";
    assert!(scale_size_target(src, SizeTarget::CircleR(0), 0.0).is_err());
    assert!(scale_size_target(src, SizeTarget::CircleR(0), -1.0).is_err());
}

#[test]
fn set_rotation_wraps_about_object_center() {
    let src = "(page a4 (circle 10 20 5))";
    assert_eq!(layer_rotation_deg(src, 0, 0).unwrap(), 0.0);
    let out = set_layer_rotation_deg(src, 0, 0, 30.0, (10.0, 20.0)).unwrap();
    assert!(
        out.contains("(translate 10 20 (rotate 30 (translate -10 -20 (circle 10 20 5))))"),
        "unexpected rewrite: {out}"
    );
    assert_eq!(layer_rotation_deg(&out, 0, 0).unwrap(), 30.0);
    let out2 = set_layer_rotation_deg(&out, 0, 0, -15.0, (10.0, 20.0)).unwrap();
    assert!(out2.contains("(rotate -15 "), "unexpected rewrite: {out2}");
    assert_eq!(layer_rotation_deg(&out2, 0, 0).unwrap(), -15.0);
}

#[test]
fn set_rotation_upgrades_bare_rotate_to_center_pivot() {
    let src = "(page a4 (rotate 10 (circle 10 20 5)))";
    let out = set_layer_rotation_deg(src, 0, 0, 45.0, (10.0, 20.0)).unwrap();
    assert!(
        out.contains("(translate 10 20 (rotate 45 (translate -10 -20 (circle 10 20 5))))"),
        "unexpected rewrite: {out}"
    );
    assert!(!out.contains("(rotate 45 (circle"));
}

#[test]
fn set_rotation_on_existing_rotate_root() {
    let src = "(page a4 (translate 1 2 (rotate 10 (translate -1 -2 (circle 0 0 3)))))";
    let out = set_layer_rotation_deg(src, 0, 0, 45.0, (1.0, 2.0)).unwrap();
    assert!(out.contains("(rotate 45 "));
    assert_eq!(layer_rotation_deg(&out, 0, 0).unwrap(), 45.0);
}

#[test]
fn center_rotate_sandwich_drag_uses_outer_translate() {
    let src = "(page a4 (translate 10 20 (rotate 30 (translate -10 -20 (circle 10 20 5)))))";
    let t = collect_drag_targets_page(src, 0).unwrap();
    assert_eq!(t, vec![DragTarget::Translate(0)]);
    let out = nudge_drag_target(src, DragTarget::Translate(0), 1.0, 2.0).unwrap();
    assert!(out.contains("(translate 11 22 (rotate 30 (translate -10 -20"));
}

#[test]
fn set_opacity_wraps_then_edits() {
    let src = "(page a4 (circle 1 2 3))";
    assert_eq!(layer_opacity(src, 0, 0).unwrap(), 1.0);
    let out = set_layer_opacity(src, 0, 0, 0.4).unwrap();
    assert!(out.contains("(opacity 0.4 (circle 1 2 3))"));
    assert_eq!(layer_opacity(&out, 0, 0).unwrap(), 0.4);
    let out2 = set_layer_opacity(&out, 0, 0, 0.75).unwrap();
    assert!(out2.contains("(opacity 0.75 (circle 1 2 3))"));
}

#[test]
fn opacity_clamps_to_unit_interval() {
    let src = "(page a4 (circle 1 2 3))";
    let out = set_layer_opacity(src, 0, 0, 2.0).unwrap();
    assert!(out.contains("(opacity 1 (circle 1 2 3))"));
    let out = set_layer_opacity(src, 0, 0, -0.5).unwrap();
    assert!(out.contains("(opacity 0 (circle 1 2 3))"));
}

#[test]
fn scale_line_about_midpoint() {
    let src = "(page a4 (line 0 0 10 0 red 2))";
    let sizes = collect_size_targets_page(src, 0).unwrap();
    assert_eq!(sizes, vec![SizeTarget::LineSeg(0)]);
    let out = scale_size_target(src, SizeTarget::LineSeg(0), 2.0).unwrap();
    // midpoint (5,0); endpoints → (-5,0) and (15,0); width 4
    assert!(
        out.contains("(line -5 0 15 0 red 4)"),
        "unexpected rewrite: {out}"
    );
}

#[test]
fn scale_polyline_vertices_and_width() {
    let src = "(page a4 (polyline 0 0 10 0 10 10 red 1.5))";
    let out = scale_size_target(src, SizeTarget::PolylinePoints(0), 2.0).unwrap();
    // centroid (20/3, 10/3) ≈ (6.667, 3.333) — check scaled width and that coords moved
    assert!(out.contains("red 3"), "unexpected rewrite: {out}");
    assert!(!out.contains("(polyline 0 0 10 0 10 10"));
}

#[test]
fn scale_polygon_about_centroid() {
    let src = "(page a4 (polygon 0 0 10 0 0 10))";
    let out = scale_size_target(src, SizeTarget::PolygonPoints(0), 2.0).unwrap();
    // centroid (10/3, 10/3); first vertex 0,0 → -10/3, -10/3
    assert!(
        out.contains("-3.333") || out.contains("-3.333333"),
        "unexpected rewrite: {out}"
    );
}

#[test]
fn delete_top_level_layer() {
    let src = "(page a4 (circle 1 2 3) (rect 0 0 1 1))";
    let out = delete_layer_page(src, 0, 0).unwrap();
    assert!(!out.contains("circle"));
    assert!(out.contains("(rect 0 0 1 1)"));
    let layers = collect_layers_page(&out, 0).unwrap();
    assert_eq!(layers.len(), 1);
}

#[test]
fn delete_leaf_inside_translate_keeps_sibling() {
    let src = "(page a4 (translate 0 0 (circle 1 2 3) (rect 0 0 1 1)))";
    let out = delete_layer_page(src, 0, 0).unwrap();
    assert!(out.contains("(translate 0 0"));
    assert!(!out.contains("circle"));
    assert!(out.contains("(rect 0 0 1 1)"));
}

#[test]
fn duplicate_top_level_layer() {
    let src = "(page a4 (circle 1 2 3) (rect 0 0 1 1))";
    let out = duplicate_layer_page(src, 0, 0).unwrap();
    let layers = collect_layers_page(&out, 0).unwrap();
    assert_eq!(layers.len(), 3);
    assert_eq!(layers[0].kind, "circle");
    assert_eq!(layers[1].kind, "circle");
    assert_eq!(layers[2].kind, "rect");
}

#[test]
fn insert_layer_appends_and_returns_index() {
    let src = "(page a4 (circle 1 2 3))";
    let (out, idx) = insert_layer_page(src, 0, "(rect 10 20 30 40 red)").unwrap();
    assert!(out.contains("(rect 10 20 30 40 red)"));
    let layers = collect_layers_page(&out, 0).unwrap();
    assert_eq!(idx, layers.len() - 1);
    assert_eq!(layers[idx].kind, "rect");
}

#[test]
fn insert_layer_into_empty_page() {
    let src = "(page a4)";
    let (out, idx) = insert_layer_page(src, 0, "(circle 105 148.5 20)").unwrap();
    assert_eq!(out, "(page a4 (circle 105 148.5 20))");
    assert_eq!(idx, 0);
}

#[test]
fn scale_layer_uniform_wraps_any_root() {
    let src = "(page a4 (circle 1 2 3))";
    let out = scale_layer_uniform(src, 0, 0, 2.0).unwrap();
    assert!(out.contains("(scale 2 (circle 1 2 3))"));
}

#[test]
fn group_and_ungroup_contiguous_roots() {
    let src = "(page a4 (circle 1 2 3) (rect 0 0 10 10) (ellipse 50 60 5 4))";
    let (grouped, sel) = group_layers_page(src, 0, &[0, 1]).unwrap();
    assert!(grouped.contains("(group"), "expected group: {grouped}");
    assert!(grouped.contains("(circle 1 2 3)"));
    assert!(grouped.contains("(rect 0 0 10 10)"));
    assert!(grouped.contains("(ellipse 50 60 5 4)"));
    assert_eq!(sel, vec![0, 1]);
    let layers = collect_layers_page(&grouped, 0).unwrap();
    assert_eq!(layers.len(), 3);
    // Shared group root for first two leaves.
    assert_eq!(
        (layers[0].root_start, layers[0].root_end),
        (layers[1].root_start, layers[1].root_end)
    );
    assert_ne!(
        (layers[0].root_start, layers[0].root_end),
        (layers[2].root_start, layers[2].root_end)
    );

    let (ungrouped, sel2) = ungroup_layer_page(&grouped, 0, 0).unwrap();
    assert!(!ungrouped.contains("(group"), "peeled: {ungrouped}");
    assert_eq!(sel2, vec![0, 1]);
    let layers2 = collect_layers_page(&ungrouped, 0).unwrap();
    assert_eq!(layers2.len(), 3);
    assert_ne!(
        (layers2[0].root_start, layers2[0].root_end),
        (layers2[1].root_start, layers2[1].root_end)
    );
}

#[test]
fn group_rejects_noncontiguous_roots() {
    let src = "(page a4 (circle 1 2 3) (rect 0 0 10 10) (ellipse 50 60 5 4))";
    let err = group_layers_page(src, 0, &[0, 2]).unwrap_err();
    assert!(err.message.contains("contiguous"));
}

#[test]
fn insert_and_delete_pages() {
    let src = "(page a4 (circle 1 2 3))\n(page a4)";
    assert_eq!(count_pages(src).unwrap(), 2);
    let (out, idx) = insert_page_after(src, Some(0), "(page letter)").unwrap();
    assert_eq!(idx, 1);
    assert_eq!(count_pages(&out).unwrap(), 3);
    assert!(out.contains("(page letter)"));
    let layers_mid = collect_layers_page(&out, 1).unwrap();
    assert!(layers_mid.is_empty());
    let trimmed = delete_page(&out, 1).unwrap();
    assert_eq!(count_pages(&trimmed).unwrap(), 2);
    assert!(!trimmed.contains("letter"));
    let only = "(page a4 (circle 0 0 1))";
    assert!(delete_page(only, 0).unwrap_err().message.contains("only"));
}

#[test]
fn nudge_layer_wraps_bare_shape_with_translate() {
    let src = "(page a4 (circle 10 20 5))";
    let out = nudge_layer_page(src, 0, 0, 3.0, -1.0).unwrap();
    assert!(
        out.contains("(translate 3 -1 (circle 10 20 5))"),
        "unexpected rewrite: {out}"
    );
    // Second nudge edits the translate only (world axes).
    let out2 = nudge_layer_page(&out, 0, 0, 1.0, 1.0).unwrap();
    assert!(
        out2.contains("(translate 4 0 (circle 10 20 5))"),
        "unexpected rewrite: {out2}"
    );
}

#[test]
fn nudge_layer_under_bare_rotate_stays_world_axis() {
    // Without wrap, nudging circle x/y would move along the rotated frame.
    let src = "(page a4 (rotate 90 (circle 10 0 1)))";
    let out = nudge_layer_page(src, 0, 0, 5.0, 0.0).unwrap();
    assert!(
        out.contains("(translate 5 0 (rotate 90 (circle 10 0 1)))"),
        "unexpected rewrite: {out}"
    );
    // Angle remains visible to the rotate knob after the move wrap.
    assert_eq!(layer_rotation_deg(&out, 0, 0).unwrap(), 90.0);
}

#[test]
fn nudge_layer_center_sandwich_moves_outer_only() {
    let src = "(page a4 (translate 10 20 (rotate 30 (translate -10 -20 (circle 10 20 5)))))";
    let out = nudge_layer_page(src, 0, 0, 2.0, 3.0).unwrap();
    assert!(out.contains("(translate 12 23 (rotate 30 (translate -10 -20"));
    assert!(out.contains("(circle 10 20 5)"));
}

#[test]
fn collect_all_drag_and_size_target_kinds() {
    let src = r#"(page a4
  (ring 1 2 3 0.5)
  (frame 0 0 10 20 1)
  (ellipse 5 6 7 8)
  (polyline 0 0 10 0 10 10)
  (polygon 0 0 10 0 0 10)
  (image "a.png" 1 2 30 40)
  (scale 2 3 (rect 1 2 3 4)))"#;
    let drag = collect_drag_targets_page(src, 0).unwrap();
    assert_eq!(drag.len(), 7);
    assert!(drag.contains(&DragTarget::RingXy(0)));
    assert!(drag.contains(&DragTarget::FrameXy(0)));
    assert!(drag.contains(&DragTarget::EllipseXy(0)));
    assert!(drag.contains(&DragTarget::PolylineXy(0)));
    assert!(drag.contains(&DragTarget::PolygonXy(0)));
    assert!(drag.contains(&DragTarget::ImageXy(0)));
    assert!(drag.contains(&DragTarget::RectXy(0)));

    let sizes = collect_size_targets_page(src, 0).unwrap();
    assert_eq!(
        sizes,
        vec![
            SizeTarget::RingR(0),
            SizeTarget::FrameWh(0),
            SizeTarget::EllipseRxRy(0),
            SizeTarget::PolylinePoints(0),
            SizeTarget::PolygonPoints(0),
            SizeTarget::ImageWh(0),
            SizeTarget::RectWh(0),
        ]
    );
}

#[test]
fn inherited_translate_binds_all_leaf_kinds() {
    let src = "(page a4 (translate 1 2 (ring 0 0 3 0.5) (frame 0 0 1 1 1)))";
    let t = collect_drag_targets_page(src, 0).unwrap();
    assert_eq!(t, vec![DragTarget::Translate(0), DragTarget::Translate(0)]);
}

#[test]
fn nudge_layer_zero_delta_is_noop() {
    let src = "(page a4 (circle 1 2 3))";
    let out = nudge_layer_page(src, 0, 0, 0.0, 0.0).unwrap();
    assert_eq!(out, src);
}

#[test]
fn nudge_layer_opacity_wrapped_translate() {
    let src = "(page a4 (opacity 0.5 (translate 1 2 (circle 0 0 3))))";
    let out = nudge_layer_page(src, 0, 0, 5.0, 0.0).unwrap();
    assert!(out.contains("(translate 6 2 (circle 0 0 3))"), "{out}");
}

#[test]
fn scale_uniform_identity_is_noop() {
    let src = "(page a4 (circle 1 2 3))";
    assert_eq!(scale_layer_uniform(src, 0, 0, 1.0).unwrap(), src);
}

#[test]
fn scale_unsupported_target_is_noop() {
    let src = "(page a4 (circle 1 2 3))";
    assert_eq!(
        scale_size_target(src, SizeTarget::Unsupported, 2.0).unwrap(),
        src
    );
}

#[test]
fn scale_ring_and_frame_and_image() {
    let src = "(page a4 (ring 10 20 5 1) (frame 0 0 10 20 1) (image \"x.png\" 1 2 40 30))";
    let out = scale_size_target(src, SizeTarget::RingR(0), 2.0).unwrap();
    assert!(out.contains("(ring 10 20 10 1)"), "{out}");
    let out = scale_size_target(&out, SizeTarget::FrameWh(0), 0.5).unwrap();
    assert!(out.contains("(frame 2.5 5 5 10 1)"), "{out}");
    let out = scale_size_target(&out, SizeTarget::ImageWh(0), 2.0).unwrap();
    assert!(out.contains("80"), "{out}");
}

#[test]
fn set_polygon_vertex() {
    let src = "(page a4 (polygon 0 0 10 0 0 10))";
    let out = set_poly_vertex(src, "polygon", 0, 2, 1.0, 2.0).unwrap();
    assert!(out.contains("(polygon 0 0 10 0 1 2)"), "{out}");
}

#[test]
fn set_box_and_text_validation_errors() {
    let src = "(page a4 (rect 1 2 3 4))";
    assert!(set_box_xywh(src, "rect", 0, [1, 2, 3, 4], f64::NAN, 1.0, 2.0, 3.0).is_err());
    assert!(set_box_xywh(src, "rect", 0, [1, 2, 3, 4], 0.0, 0.0, 0.0, 1.0).is_err());
    assert!(set_text_box(src, 0, 0.0, 0.0, -1.0, 2.0).is_err());
    assert!(set_line_endpoint(src, 0, 2, 1.0, 1.0).is_err());
    assert!(set_poly_vertex(src, "circle", 0, 0, 1.0, 1.0).is_err());
}

#[test]
fn layer_index_errors() {
    let src = "(page a4 (circle 1 2 3))";
    assert!(nudge_layer_page(src, 0, 9, 1.0, 0.0).is_err());
    assert!(layer_opacity(src, 0, 9).is_err());
    assert!(set_layer_opacity(src, 0, 9, 0.5).is_err());
    assert!(layer_rotation_deg(src, 0, 9).is_err());
    assert!(delete_layer_page(src, 0, 9).is_err());
}

#[test]
fn bad_opacity_form_errors() {
    let src = "(page a4 (opacity (circle 1 2 3)))";
    assert!(layer_opacity(src, 0, 0).is_err());
}

#[test]
fn layer_labels_for_text_and_image() {
    let src = r#"(page a4 (text 1 2 3 "hello world") (image "dir/pic.png" 0 0 1 1))"#;
    let layers = collect_layers_page(src, 0).unwrap();
    assert_eq!(layers[0].label, "text \"hello world\"");
    assert_eq!(layers[1].label, "image pic.png");
}

#[test]
fn reorder_same_index_is_noop() {
    let src = "(page a4 (circle 1 2 3) (rect 0 0 1 1))";
    assert_eq!(reorder_layer_page(src, 0, 1, 1).unwrap(), src);
}

#[test]
fn duplicate_leaf_inside_shared_translate() {
    let src = "(page a4 (translate 0 0 (circle 1 2 3) (rect 0 0 1 1)))";
    let out = duplicate_layer_page(src, 0, 0).unwrap();
    assert_eq!(collect_layers_page(&out, 0).unwrap().len(), 3);
}

#[test]
fn group_needs_two_layers_and_contiguous_roots() {
    let src = "(page a4 (circle 1 2 3))";
    assert!(group_layers_page(src, 0, &[0])
        .unwrap_err()
        .message
        .contains("two"));
    let src2 = "(page a4 (circle 1 2 3) (rect 0 0 1 1) (ellipse 1 1 2 2))";
    assert!(group_layers_page(src2, 0, &[0, 0])
        .unwrap_err()
        .message
        .contains("two"));
}

#[test]
fn ungroup_non_group_errors() {
    let src = "(page a4 (circle 1 2 3))";
    assert!(ungroup_layer_page(src, 0, 0)
        .unwrap_err()
        .message
        .contains("group"));
}

#[test]
fn insert_layer_rejects_bad_form() {
    let src = "(page a4)";
    assert!(insert_layer_page(src, 0, "not a list").is_err());
    assert!(insert_layer_page(src, 0, "").is_err());
}

#[test]
fn insert_page_into_empty_source() {
    let (out, idx) = insert_page_after("", None, "(page a4)").unwrap();
    assert_eq!(idx, 0);
    assert_eq!(out.trim(), "(page a4)");
}

#[test]
fn insert_page_after_last_and_before_first() {
    let src = "(page a4)\n(page letter)";
    let (out, idx) = insert_page_after(src, Some(1), "(page 100 150)").unwrap();
    assert_eq!(idx, 2);
    assert!(out.contains("(page 100 150)"));
    let (out2, idx2) = insert_page_after(src, None, "(page a4)").unwrap();
    assert_eq!(idx2, 0);
    assert!(out2.starts_with("(page a4)"));
}

#[test]
fn delete_page_out_of_range_errors() {
    let src = "(page a4)\n(page letter)";
    assert!(delete_page(src, 9).unwrap_err().message.contains("range"));
}

#[test]
fn page_body_start_numeric_paper() {
    use reciplexa_lower::cst_walk::Child;
    use reciplexa_lower::{find_page, list_atoms, page_body_start, parse_root};

    let root = parse_root("(page 100 150 (circle 1 2 3))").unwrap();
    let page = find_page(&root, 0).unwrap();
    let items = list_atoms(&page);
    assert_eq!(page_body_start(&items), 3);
    assert!(matches!(items.get(3), Some(Child::Node(_))));
}

#[test]
fn sync_error_and_extent_helpers() {
    let err = SyncError::new("test");
    assert_eq!(err.message, "test");
    let src = "  \n  (circle 1 2 3)";
    let circle_start = src.find("(circle").unwrap();
    let (s, e) = extent_with_leading_ws(src, circle_start, src.len());
    assert!(s < circle_start);
    assert_eq!(e, src.len());
    let root = parse_root("(translate 1 2 (circle 0 0 1))").unwrap();
    let translate = root
        .descendants()
        .find(|n| is_headed(n, "translate"))
        .unwrap();
    assert!(is_headed(&translate, "translate"));
}

#[test]
fn set_rotation_non_finite_errors() {
    let src = "(page a4 (circle 1 2 3))";
    assert!(set_layer_rotation_deg(src, 0, 0, f64::NAN, (1.0, 2.0)).is_err());
    assert!(set_layer_opacity(src, 0, 0, f64::INFINITY).is_err());
}

#[test]
fn polyline_without_trailing_width_still_scales() {
    let src = "(page a4 (polyline 0 0 10 0 10 10))";
    let out = scale_size_target(src, SizeTarget::PolylinePoints(0), 2.0).unwrap();
    assert!(!out.contains("(polyline 0 0 10 0 10 10)"), "{out}");
}

#[test]
fn scale_axes_circle_uses_max_factor() {
    let src = "(page a4 (circle 10 20 5))";
    let out = scale_size_target_axes(src, SizeTarget::CircleR(0), 1.0, 3.0).unwrap();
    assert!(out.contains("(circle 10 20 15)"), "{out}");
}

#[test]
fn page_crud_form_and_parse_errors() {
    let src = "(page a4)\n(page letter)";
    assert!(insert_page_after(src, Some(0), "")
        .unwrap_err()
        .message
        .contains("insert_page form"));
    assert!(insert_page_after(src, Some(0), "not-a-page")
        .unwrap_err()
        .message
        .contains("insert_page form"));
    assert!(insert_page_after(src, Some(0), "(page")
        .unwrap_err()
        .message
        .contains("insert_page form"));
    assert!(insert_page_after(src, Some(99), "(page a4)")
        .unwrap_err()
        .message
        .contains("out of range"));
    assert!(count_pages("(").is_err());
    assert!(delete_page("(", 0).is_err());
    assert!(insert_page_after("(", None, "(page a4)").is_err());
}

#[test]
fn pages_skip_non_page_forms() {
    let src = "(src ignored)\n(page a4 (circle 1 2 3))\n(doc x)\n(page letter)";
    assert_eq!(count_pages(src).unwrap(), 2);
    let page0 = find_page(&parse_root(src).unwrap(), 0).unwrap();
    assert!(is_headed(&page0, "page"));
    let trimmed = delete_page(src, 1).unwrap();
    assert_eq!(count_pages(&trimmed).unwrap(), 1);
    assert!(trimmed.contains("a4"));
    assert!(!trimmed.contains("letter"));
}

#[test]
fn extent_with_leading_ws_strips_crlf() {
    let src = "  \r\n  (circle 1 2 3)";
    let circle_start = src.find("(circle").unwrap();
    let (s, e) = extent_with_leading_ws(src, circle_start, src.len());
    assert!(s < circle_start);
    assert_eq!(&src[s..circle_start], "\r\n  ");
    assert_eq!(e, src.len());
}

#[test]
fn insert_page_pads_when_adjacent_without_newline() {
    let src = "(page a4)(page letter)";
    let (out, idx) = insert_page_after(src, Some(0), "(page 100 100)").unwrap();
    assert_eq!(idx, 1);
    assert!(out.contains("\n(page 100 100)\n") || out.contains("(page 100 100)"));
}
