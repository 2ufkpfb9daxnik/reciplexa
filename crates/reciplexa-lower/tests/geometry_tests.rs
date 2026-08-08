use reciplexa_lower::*;


const PAGE: &str = "(page a4 ";

fn page(body: &str) -> String {
    format!("{PAGE}{body})")
}

// --- equivalence: every DragTarget arm nudges the right slots ---

#[test]
fn collect_drag_targets_delegates_to_page_zero() {
    let src = page("(circle 1 2 3)");
    assert_eq!(
        collect_drag_targets(&src).unwrap(),
        collect_drag_targets_page(&src, 0).unwrap()
    );
}

#[test]
fn nudge_all_drag_target_variants() {
    let src = page(
        r#"(translate 1 2 (circle 0 0 1))
  (rect 10 20 30 40)
  (ellipse 5 6 7 8)
  (ring 1 2 3 0.5)
  (frame 0 0 10 20 1)
  (text 1 2 3 "Hi")
  (line 0 0 10 10)
  (polyline 0 0 10 0 10 10)
  (polygon 0 0 10 0 0 10)
  (image "x.png" 4 5 6 7)"#,
    );
    let pairs = [
        (DragTarget::Translate(0), "translate 2 3"),
        (DragTarget::RectXy(0), "rect 11 21"),
        (DragTarget::EllipseXy(0), "ellipse 6 7"),
        (DragTarget::RingXy(0), "ring 2 3"),
        (DragTarget::FrameXy(0), "frame 1 1"),
        (DragTarget::TextXy(0), r#"(text 2 3 3 "Hi")"#),
        (DragTarget::LineXy(0), "line 1 1 11 11"),
        (DragTarget::PolylineXy(0), "polyline 1 1"),
        (DragTarget::PolygonXy(0), "polygon 1 1"),
        (DragTarget::ImageXy(0), "image \"x.png\" 5 6"),
    ];
    for (target, needle) in pairs {
        let out = nudge_drag_target(&src, target, 1.0, 1.0).unwrap();
        assert!(out.contains(needle), "target {target:?}: {out}");
    }
}

// --- boundary: nudge_layer_page transform sandwiches ---

#[test]
fn nudge_layer_on_existing_translate_edits_slots() {
    let src = page("(translate 5 6 (circle 0 0 3))");
    let out = nudge_layer_page(&src, 0, 0, 2.0, -1.0).unwrap();
    assert!(out.contains("(translate 7 5 (circle 0 0 3))"), "{out}");
}

#[test]
fn nudge_layer_opacity_child_translate() {
    let src = page("(opacity 0.5 (translate 1 2 (rect 0 0 1 1)))");
    let out = nudge_layer_page(&src, 0, 0, 3.0, 4.0).unwrap();
    assert!(out.contains("(translate 4 6 (rect 0 0 1 1))"), "{out}");
}

#[test]
fn nudge_layer_opacity_without_translate_wraps() {
    let src = page("(opacity 0.5 (circle 10 20 5))");
    let out = nudge_layer_page(&src, 0, 0, 1.0, 2.0).unwrap();
    assert!(
        out.contains("(translate 1 2 (opacity 0.5 (circle 10 20 5)))"),
        "{out}"
    );
}

#[test]
fn nudge_layer_translate_wrapping_bare_rotate() {
    let src = page("(translate 0 0 (rotate 45 (circle 1 2 3)))");
    let out = nudge_layer_page(&src, 0, 0, 1.0, 0.0).unwrap();
    assert!(out.contains("(translate 1 0 (rotate 45"), "{out}");
}

// --- median: rotation read/write branches ---

#[test]
fn layer_rotation_on_bare_rotate_and_nested_translate() {
    let bare = page("(rotate 12 (circle 0 0 1))");
    assert_eq!(layer_rotation_deg(&bare, 0, 0).unwrap(), 12.0);
    let nested = page("(translate 1 2 (rotate 33 (circle 0 0 1)))");
    assert_eq!(layer_rotation_deg(&nested, 0, 0).unwrap(), 33.0);
}

#[test]
fn set_rotation_peels_translate_wrapped_bare_rotate() {
    let src = page("(translate 5 6 (rotate 10 (circle 0 0 2)))");
    let out = set_layer_rotation_deg(&src, 0, 0, 20.0, (5.0, 6.0)).unwrap();
    assert!(
        out.contains("(translate 5 6 (rotate 20 (translate -5 -6 (circle 0 0 2))))"),
        "{out}"
    );
}

#[test]
fn set_rotation_missing_rotate_body_errors() {
    let broken = page("(rotate 10)");
    let err = set_layer_rotation_deg(&broken, 0, 0, 5.0, (0.0, 0.0)).unwrap_err();
    assert!(
        err.message.contains("out of range") || err.message.contains("missing body"),
        "unexpected: {}",
        err.message
    );
}

// --- boundary: opacity ---

#[test]
fn layer_opacity_defaults_and_clamps_read() {
    let plain = page("(circle 1 2 3)");
    assert_eq!(layer_opacity(&plain, 0, 0).unwrap(), 1.0);
    let wrapped = page("(opacity 1.5 (circle 1 2 3))");
    assert_eq!(layer_opacity(&wrapped, 0, 0).unwrap(), 1.0);
}

#[test]
fn layer_opacity_missing_alpha_errors() {
    let src = page("(opacity (circle 1 2 3))");
    assert!(layer_opacity(&src, 0, 0).is_err());
    assert!(set_layer_opacity(&src, 0, 0, 0.5).is_err());
}

// --- equivalence: scale_size_target all arms ---

#[test]
fn scale_size_target_every_variant() {
    let src = page(
        r#"(circle 0 0 2)
  (ellipse 10 10 4 8)
  (text 5 5 6 20 10 "x")
  (line 0 0 10 0)
  (polyline 0 0 10 0 10 10 (rgb 1 0 0) 2)
  (polygon 0 0 10 0 0 10)
  (image "p.png" 0 0 10 20)"#,
    );
    let pairs = [
        (SizeTarget::CircleR(0), false),
        (SizeTarget::EllipseRxRy(0), false),
        (SizeTarget::TextSize(0), false),
        (SizeTarget::LineSeg(0), false),
        (SizeTarget::PolylinePoints(0), false),
        (SizeTarget::PolygonPoints(0), false),
        (SizeTarget::ImageWh(0), false),
        (SizeTarget::Unsupported, true),
    ];
    for (target, noop) in pairs {
        let out = scale_size_target(&src, target, 2.0).unwrap();
        if noop {
            assert_eq!(out, src);
        } else {
            assert_ne!(out, src, "target {target:?} should rewrite");
        }
    }
}

#[test]
fn scale_size_target_axes_line_poly_use_max_factor() {
    let src = page("(line 0 0 10 0)");
    let out = scale_size_target_axes(&src, SizeTarget::LineSeg(0), 1.0, 2.0).unwrap();
    assert!(out.contains("line -5 0 15 0"), "{out}");
    let src2 = page("(polyline 0 0 10 0)");
    let out2 = scale_size_target_axes(&src2, SizeTarget::PolylinePoints(0), 3.0, 1.0).unwrap();
    assert!(!out2.contains("(polyline 0 0 10 0)"), "{out2}");
}

#[test]
fn scale_text_box_with_existing_dims() {
    let src = page(r#"(text 10 20 8 40 20 "boxed")"#);
    let out = scale_text_box(&src, 0, 0.5, 2.0).unwrap();
    assert!(out.contains(" 8 20 40 "), "dims scaled about center: {out}");
}

#[test]
fn scale_box_axes_validation() {
    let src = page("(rect 0 0 10 10)");
    assert!(scale_box_axes(&src, "rect", 0, [1, 2, 3, 4], 0.0, 1.0).is_err());
    assert!(scale_box_axes(&src, "rect", 0, [1, 2, 3, 4], 1.0, f64::NAN).is_err());
}

#[test]
fn scale_polyline_rgb_node_width() {
    let src = page("(polyline 0 0 10 0 10 10 (rgb 0.5 0.5 0.5) 1)");
    let out = scale_size_target(&src, SizeTarget::PolylinePoints(0), 2.0).unwrap();
    assert!(out.contains("rgb"), "{out}");
    assert!(out.contains(" 2)"), "width doubled: {out}");
}

// --- boundary: setters ---

#[test]
fn set_frame_and_image_box() {
    let src = page(r#"(frame 1 2 3 4 0.5) (image "a.png" 5 6 7 8)"#);
    let f = set_box_xywh(&src, "frame", 0, [1, 2, 3, 4], 0.0, 0.0, 1.0, 2.0).unwrap();
    assert!(f.contains("(frame 0 0 1 2 0.5)"), "{f}");
    let i = set_box_xywh(&f, "image", 0, [2, 3, 4, 5], 1.0, 2.0, 3.0, 4.0).unwrap();
    assert!(i.contains(r#"(image "a.png" 1 2 3 4)"#), "{i}");
}

#[test]
fn set_polygon_vertex_and_bad_head() {
    let src = page("(polygon 0 0 10 0 0 10)");
    let out = set_poly_vertex(&src, "polygon", 0, 0, 9.0, 9.0).unwrap();
    assert!(out.contains("(polygon 9 9 10 0 0 10)"), "{out}");
    assert!(set_poly_vertex(&src, "line", 0, 0, 1.0, 1.0).is_err());
    assert!(set_poly_vertex(&src, "polygon", 0, 0, f64::NAN, 1.0).is_err());
}

#[test]
fn set_line_endpoint_finite_guard() {
    let src = page("(line 0 0 1 1)");
    assert!(set_line_endpoint(&src, 0, 0, f64::INFINITY, 0.0).is_err());
}

// --- defect: nudge/scale error paths ---

#[test]
fn nudge_missing_pair_errors() {
    let src = page("(circle x y 3)");
    assert!(nudge_drag_target(&src, DragTarget::CircleXy(0), 1.0, 0.0).is_err());
}

#[test]
fn nudge_translate_missing_numeric_slots_errors() {
    let src = page("(translate x y (circle 0 0 1))");
    assert!(nudge_layer_page(&src, 0, 0, 1.0, 0.0).is_err());
}

#[test]
fn scale_non_finite_factor_errors() {
    let src = page("(circle 1 2 3)");
    assert!(scale_size_target(&src, SizeTarget::CircleR(0), f64::NAN).is_err());
    assert!(scale_layer_uniform(&src, 0, 0, -1.0).is_err());
    assert!(scale_text_box(&src, 0, 0.0, 1.0).is_err());
}

#[test]
fn collect_size_under_nonuniform_scale_and_group() {
    let src = page("(scale 2 3 (group (circle 1 2 3) (rect 0 0 1 1)))");
    let sizes = collect_size_targets_page(&src, 0).unwrap();
    assert_eq!(sizes, vec![SizeTarget::CircleR(0), SizeTarget::RectWh(0)]);
    let drag = collect_drag_targets_page(&src, 0).unwrap();
    assert_eq!(drag.len(), 2);
}

#[test]
fn collect_size_unknown_leaf_is_unsupported() {
    let src = page("(bogus 1 2 3)");
    let sizes = collect_size_targets_page(&src, 0).unwrap();
    assert_eq!(sizes, vec![SizeTarget::Unsupported]);
    let drag = collect_drag_targets_page(&src, 0).unwrap();
    assert!(drag.is_empty());
}

#[test]
fn set_text_box_bad_dims_errors() {
    let src = page(r#"(text 1 2 3 "x")"#);
    assert!(set_text_box(&src, 0, 0.0, 0.0, 0.0, 1.0).is_err());
    assert!(set_text_box(&src, 9, 0.0, 0.0, 1.0, 1.0).is_err());
}

#[test]
fn scale_layer_uniform_out_of_range_errors() {
    let src = page("(circle 1 2 3)");
    assert!(scale_layer_uniform(&src, 0, 9, 2.0).is_err());
}

#[test]
fn nudge_layer_zero_delta_is_identity() {
    let src = page("(circle 1 2 3)");
    assert_eq!(nudge_layer_page(&src, 0, 0, 0.0, 0.0).unwrap(), src);
}

#[test]
fn nudge_layer_opacity_child_with_translate() {
    let src = page("(opacity 0.5 (translate 5 6 (circle 0 0 1)))");
    let out = nudge_layer_page(&src, 0, 0, 1.0, 2.0).unwrap();
    assert!(out.contains("(translate 6 8"), "{out}");
}

#[test]
fn nudge_first_translate_delegates() {
    let src = page("(translate 1 2 (circle 0 0 1))");
    let out = nudge_first_translate(&src, 3.0, 4.0).unwrap();
    assert!(out.contains("(translate 4 6"), "{out}");
}

#[test]
fn inherited_translate_collect_targets() {
    let src = page("(translate 1 2 (circle 0 0 3)) (rect 10 20 1 1)");
    let drag = collect_drag_targets_page(&src, 0).unwrap();
    assert_eq!(drag, vec![DragTarget::Translate(0), DragTarget::RectXy(0)]);
}

#[test]
fn set_rotation_on_center_sandwich_only_changes_degrees() {
    let src = page("(translate 10 20 (rotate 5 (translate -10 -20 (circle 0 0 2))))");
    let out = set_layer_rotation_deg(&src, 0, 0, 15.0, (10.0, 20.0)).unwrap();
    assert!(out.contains("(rotate 15"), "{out}");
    assert!(!out.contains("(rotate 5"), "{out}");
}

#[test]
fn set_layer_opacity_wraps_bare_circle() {
    let src = page("(circle 1 2 3)");
    let out = set_layer_opacity(&src, 0, 0, 0.25).unwrap();
    assert!(out.contains("(opacity 0.25 (circle 1 2 3))"), "{out}");
}

#[test]
fn layer_opacity_bad_number_errors() {
    let src = page("(opacity bad (circle 1 2 3))");
    assert!(layer_opacity(&src, 0, 0).is_err());
}

#[test]
fn scale_size_target_axes_frame_and_ring() {
    let src = page("(ring 0 0 4 0.5) (frame 0 0 10 20 1)");
    let out1 = scale_size_target_axes(&src, SizeTarget::RingR(0), 2.0, 2.0).unwrap();
    assert!(out1.contains("ring 0 0 8"), "{out1}");
    let out2 = scale_size_target_axes(&out1, SizeTarget::FrameWh(0), 0.5, 2.0).unwrap();
    assert!(!out2.contains("frame 0 0 10 20"), "{out2}");
}

#[test]
fn scale_size_target_axes_image_and_ellipse() {
    let src = page(r#"(ellipse 5 5 4 8) (image "p.png" 1 2 10 20)"#);
    let out1 = scale_size_target_axes(&src, SizeTarget::EllipseRxRy(0), 2.0, 0.5).unwrap();
    assert!(!out1.contains("ellipse 5 5 4 8"), "{out1}");
    let out2 = scale_size_target_axes(&out1, SizeTarget::ImageWh(0), 1.0, 2.0).unwrap();
    assert!(!out2.contains("image \"p.png\" 1 2 10 20"), "{out2}");
}

#[test]
fn scale_size_target_axes_bad_factors_error() {
    let src = page("(rect 0 0 10 10)");
    assert!(scale_size_target_axes(&src, SizeTarget::RectWh(0), f64::NAN, 1.0).is_err());
}

#[test]
fn set_text_box_inserts_dims_when_missing() {
    let src = page(r#"(text 10 20 8 "hello")"#);
    let out = set_text_box(&src, 0, 10.0, 20.0, 40.0, 20.0).unwrap();
    assert!(out.contains(" 8 40 20 \"hello\""), "{out}");
}

#[test]
fn scale_text_box_without_existing_dims() {
    let src = page(r#"(text 0 0 10 "abcdefghij")"#);
    let out = scale_text_box(&src, 0, 2.0, 1.0).unwrap();
    assert_ne!(out, src);
}

#[test]
fn scale_layer_uniform_noop_at_one() {
    let src = page("(circle 1 2 3)");
    assert_eq!(scale_layer_uniform(&src, 0, 0, 1.0).unwrap(), src);
}

#[test]
fn nudge_circle_xy_direct() {
    let src = page("(circle 5 6 3)");
    let out = nudge_drag_target(&src, DragTarget::CircleXy(0), 1.0, -1.0).unwrap();
    assert!(out.contains("(circle 6 5 3)"), "{out}");
}

#[test]
fn set_line_endpoint_one() {
    let src = page("(line 0 0 10 10)");
    let out = set_line_endpoint(&src, 0, 1, 5.0, 5.0).unwrap();
    assert!(out.contains("line 0 0 5 5"), "{out}");
}

#[test]
fn scale_polyline_no_trailing_width_is_ok() {
    let src = page("(polyline 0 0 10 0 10 10)");
    let out = scale_size_target(&src, SizeTarget::PolylinePoints(0), 2.0).unwrap();
    assert_ne!(out, src);
}

#[test]
fn collect_drag_on_second_page() {
    let src = "(page a4 (circle 1 2 3))\n(page a4 (rect 0 0 1 1))";
    let drag = collect_drag_targets_page(src, 1).unwrap();
    assert_eq!(drag, vec![DragTarget::RectXy(0)]);
}

#[test]
fn find_rotate_on_translate_wrapped_bare_rotate() {
    let src = page("(translate 0 0 (rotate 22 (circle 0 0 1)))");
    assert_eq!(layer_rotation_deg(&src, 0, 0).unwrap(), 22.0);
}

#[test]
fn set_rotation_peels_bare_rotate_body() {
    let src = page("(rotate 10 (circle 0 0 2))");
    let out = set_layer_rotation_deg(&src, 0, 0, 20.0, (0.0, 0.0)).unwrap();
    assert!(out.contains("(rotate 20"), "{out}");
}

#[test]
fn scale_ring_target_doubles_radius() {
    let src = page("(ring 1 2 3 0.5)");
    let out = scale_size_target(&src, SizeTarget::RingR(0), 2.0).unwrap();
    assert!(out.contains("(ring 1 2 6 0.5)"), "{out}");
}

#[test]
fn set_layer_opacity_non_finite_errors() {
    let src = page("(circle 1 2 3)");
    assert!(set_layer_opacity(&src, 0, 0, f64::NAN).is_err());
}

#[test]
fn set_layer_rotation_non_finite_errors() {
    let src = page("(circle 1 2 3)");
    assert!(set_layer_rotation_deg(&src, 0, 0, f64::INFINITY, (0.0, 0.0)).is_err());
    assert!(set_layer_rotation_deg(&src, 0, 0, 10.0, (f64::NAN, 0.0)).is_err());
}

#[test]
fn scale_layer_uniform_and_nudge_line_poly() {
    let src =
        page("(circle 0 0 5) (line 0 0 10 0) (polyline 0 0 5 5 10 0) (polygon 0 0 1 0 0 1)");
    let out = scale_layer_uniform(&src, 0, 0, 2.0).unwrap();
    assert_ne!(out, src);
    let out = nudge_drag_target(&src, DragTarget::LineXy(0), 1.0, 2.0).unwrap();
    assert!(out.contains("line"), "{out}");
    let out = nudge_drag_target(&src, DragTarget::PolylineXy(0), 1.0, 0.0).unwrap();
    assert!(out.contains("polyline"), "{out}");
    let out = nudge_drag_target(&src, DragTarget::PolygonXy(0), 0.0, 1.0).unwrap();
    assert!(out.contains("polygon"), "{out}");
}

#[test]
fn opacity_and_rotation_under_wrappers() {
    let src = page("(opacity 0.5 (translate 1 2 (rotate 30 (circle 0 0 1))))");
    assert!((layer_opacity(&src, 0, 0).unwrap() - 0.5).abs() < 1e-9);
    assert_eq!(layer_rotation_deg(&src, 0, 0).unwrap(), 30.0);
    let faded = set_layer_opacity(&src, 0, 0, 0.25).unwrap();
    assert!(
        faded.contains("0.25") || faded.contains("(opacity 0.25"),
        "{faded}"
    );
}

#[test]
fn set_poly_vertex_polyline_and_bad_index() {
    let src = page("(polyline 0 0 10 0 10 10)");
    let out = set_poly_vertex(&src, "polyline", 0, 1, 4.0, 5.0).unwrap();
    assert!(out.contains("4") && out.contains("5"), "{out}");
    assert!(set_poly_vertex(&src, "polyline", 0, 99, 1.0, 1.0).is_err());
}

#[test]
fn collect_size_targets_page_out_of_range() {
    let src = page("(circle 1 2 3)");
    // Out-of-range page should error or yield empty — must not panic.
    let _ = collect_size_targets_page(&src, 9);
    let _ = collect_drag_targets_page(&src, 9);
}

#[test]
fn scale_size_target_axes_text_and_rect() {
    let src = page(r#"(text 0 0 12 20 10 "hi") (rect 0 0 10 20)"#);
    let t = scale_size_target_axes(&src, SizeTarget::TextSize(0), 2.0, 0.5).unwrap();
    assert_ne!(t, src);
    let r = scale_size_target_axes(&src, SizeTarget::RectWh(0), 2.0, 0.5).unwrap();
    assert_ne!(r, src);
}

#[test]
fn set_box_xywh_rect() {
    let src = page("(rect 1 2 3 4)");
    let out = set_box_xywh(&src, "rect", 0, [1, 2, 3, 4], 5.0, 6.0, 7.0, 8.0).unwrap();
    assert!(out.contains("(rect 5 6 7 8)"), "{out}");
}

// --- aggressive error / branch coverage for geometry.rs ---

#[test]
fn parse_errors_surface_on_collect_and_nudge() {
    assert!(collect_drag_targets_page("(", 0).is_err());
    assert!(collect_size_targets_page("(", 0).is_err());
    assert!(nudge_layer_page("(", 0, 0, 1.0, 0.0).is_err());
    assert!(scale_text_box("(", 0, 2.0, 2.0).is_err());
}

#[test]
fn page_body_token_atoms_are_skipped() {
    let src = page("123 (circle 1 2 3)");
    assert_eq!(
        collect_drag_targets_page(&src, 0).unwrap(),
        vec![DragTarget::CircleXy(0)]
    );
    assert_eq!(
        collect_size_targets_page(&src, 0).unwrap(),
        vec![SizeTarget::CircleR(0)]
    );
}

#[test]
fn empty_and_non_ident_lists_are_ignored_by_collect() {
    let src = page("() (123) (circle 1 2 3)");
    assert_eq!(
        collect_drag_targets_page(&src, 0).unwrap(),
        vec![DragTarget::CircleXy(0)]
    );
    assert_eq!(
        collect_size_targets_page(&src, 0).unwrap(),
        vec![SizeTarget::CircleR(0)]
    );
}

#[test]
fn translate_skips_non_node_body_atoms() {
    let src = page("(translate 1 2 999 (circle 0 0 1))");
    let drag = collect_drag_targets_page(&src, 0).unwrap();
    assert_eq!(drag, vec![DragTarget::Translate(0)]);
    let sizes = collect_size_targets_page(&src, 0).unwrap();
    assert_eq!(sizes, vec![SizeTarget::CircleR(0)]);
}

#[test]
fn rotate_skips_non_node_body_atoms() {
    let src = page("(rotate 15 999 (rect 0 0 1 1))");
    let drag = collect_drag_targets_page(&src, 0).unwrap();
    assert_eq!(drag, vec![DragTarget::RectXy(0)]);
}

#[test]
fn inherited_translate_binds_every_leaf_kind() {
    let src = page(
        r#"(translate 1 2
    (circle 0 0 1)
    (rect 0 0 1 1)
    (ellipse 0 0 1 1)
    (ring 0 0 1 0.2)
    (frame 0 0 1 1 1)
    (text 0 0 3 "t")
    (line 0 0 1 1)
    (polyline 0 0 1 0 1 1)
    (polygon 0 0 1 0 0 1)
    (image "a.png" 0 0 1 1))"#,
    );
    let drag = collect_drag_targets_page(&src, 0).unwrap();
    assert_eq!(drag.len(), 10);
    assert!(drag.iter().all(|t| *t == DragTarget::Translate(0)));
}

#[test]
fn uniform_scale_transform_skip_is_two() {
    let src = page("(scale 2 (group (circle 1 2 3)))");
    let sizes = collect_size_targets_page(&src, 0).unwrap();
    assert_eq!(sizes, vec![SizeTarget::CircleR(0)]);
    assert_eq!(layer_rotation_deg(&src, 0, 0).unwrap(), 0.0);
}

#[test]
fn rotation_reads_through_scale_and_group() {
    let src = page("(scale 2 3 (group (rotate 44 (circle 0 0 1))))");
    assert_eq!(layer_rotation_deg(&src, 0, 0).unwrap(), 44.0);
}

#[test]
fn nudge_layer_digit_growth_refinds_translate() {
    // x 9 → 10 changes span length; must re-find by start.
    let src = page("(translate 9 1 (circle 0 0 1))");
    let out = nudge_layer_page(&src, 0, 0, 1.0, 0.0).unwrap();
    assert!(out.contains("(translate 10 1"), "{out}");
}

#[test]
fn nudge_layer_translate_missing_y_errors() {
    let src = page("(translate 1 y (circle 0 0 1))");
    let err = nudge_layer_page(&src, 0, 0, 1.0, 0.0).unwrap_err();
    assert!(
        err.message.contains("missing numeric y"),
        "{}",
        err.message
    );
}

#[test]
fn nudge_layer_index_and_page_errors() {
    let src = page("(circle 1 2 3)");
    assert!(nudge_layer_page(&src, 0, 3, 1.0, 0.0)
        .unwrap_err()
        .message
        .contains("out of range"));
    assert!(nudge_layer_page(&src, 9, 0, 1.0, 0.0).is_err());
}

#[test]
fn nudge_layer_opacity_center_sandwich() {
    let src =
        page("(opacity 0.4 (translate 10 20 (rotate 5 (translate -10 -20 (circle 0 0 2)))))");
    let out = nudge_layer_page(&src, 0, 0, 1.0, 2.0).unwrap();
    assert!(out.contains("(translate 11 22"), "{out}");
}

#[test]
fn wrap_span_with_translate_on_bare_leaf() {
    let src = page("(circle 3 4 5)");
    let out = nudge_layer_page(&src, 0, 0, 2.0, 3.0).unwrap();
    assert!(
        out.contains("(translate 2 3 (circle 3 4 5))"),
        "{out}"
    );
}

#[test]
fn scale_factor_and_axes_guards() {
    let src = page("(circle 0 0 2) (ring 0 0 3 0.5) (line 0 0 4 0) (polyline 0 0 2 0 2 2) (polygon 0 0 1 0 0 1)");
    assert!(scale_size_target(&src, SizeTarget::CircleR(0), 0.0)
        .unwrap_err()
        .message
        .contains("finite"));
    assert!(scale_size_target_axes(&src, SizeTarget::CircleR(0), -1.0, 2.0).is_err());
    assert!(scale_size_target_axes(&src, SizeTarget::RingR(0), 1.0, f64::NAN).is_err());
    assert!(scale_size_target_axes(&src, SizeTarget::LineSeg(0), 0.0, 1.0).is_err());
    assert!(scale_size_target_axes(&src, SizeTarget::PolylinePoints(0), f64::INFINITY, 1.0).is_err());
    assert!(scale_size_target_axes(&src, SizeTarget::PolygonPoints(0), 1.0, -0.5).is_err());
    assert!(scale_size_target_axes(&src, SizeTarget::Unsupported, 2.0, 3.0).is_ok());
}

#[test]
fn scale_size_target_axes_circle_polygon_unsupported() {
    let src = page("(circle 0 0 2) (polygon 0 0 10 0 0 10)");
    let c = scale_size_target_axes(&src, SizeTarget::CircleR(0), 1.0, 4.0).unwrap();
    assert!(c.contains("(circle 0 0 8)"), "{c}");
    let p = scale_size_target_axes(&src, SizeTarget::PolygonPoints(0), 2.0, 1.0).unwrap();
    assert_ne!(p, src);
    assert_eq!(
        scale_size_target_axes(&src, SizeTarget::Unsupported, 2.0, 2.0).unwrap(),
        src
    );
}

#[test]
fn scale_missing_slots_and_odd_coords_error() {
    let src = page("(circle 1 2)");
    assert!(scale_size_target(&src, SizeTarget::CircleR(0), 2.0)
        .unwrap_err()
        .message
        .contains("slot"));
    let line = page("(line 0 0 1)");
    assert!(scale_size_target(&line, SizeTarget::LineSeg(0), 2.0).is_err());
    assert!(scale_size_target(&page("(line)"), SizeTarget::LineSeg(0), 2.0).is_err());
}

#[test]
fn scale_line_with_and_without_width() {
    let plain = page("(line 0 0 10 0 black)");
    let out = scale_size_target(&plain, SizeTarget::LineSeg(0), 2.0).unwrap();
    assert!(out.contains("line -5"), "{out}");
    let wide = page("(line 0 0 10 0 black 2)");
    let out2 = scale_size_target(&wide, SizeTarget::LineSeg(0), 2.0).unwrap();
    assert!(out2.contains(" 4)"), "width scaled: {out2}");
}

#[test]
fn scale_polyline_named_color_width() {
    let src = page("(polyline 0 0 10 0 10 10 red 3)");
    let out = scale_size_target(&src, SizeTarget::PolylinePoints(0), 2.0).unwrap();
    assert!(out.contains(" 6)"), "width scaled: {out}");
}

#[test]
fn scale_polyline_trailing_non_color_skips_width() {
    // Trailing number after a non-color atom is not treated as stroke width.
    let src = page(r#"(polyline 0 0 10 0 10 10 "meta" 2)"#);
    let out = scale_size_target(&src, SizeTarget::PolylinePoints(0), 2.0).unwrap();
    assert!(out.contains(" 2)"), "{out}");
}

#[test]
fn set_text_box_branches_and_errors() {
    let boxed = page(r#"(text 0 0 8 40 20 "hi")"#);
    let out = set_text_box(&boxed, 0, 1.0, 2.0, 10.0, 12.0).unwrap();
    assert!(out.contains(r#"(text 1 2 8 10 12 "hi")"#), "{out}");
    assert!(set_text_box(&boxed, 0, f64::NAN, 0.0, 1.0, 1.0)
        .unwrap_err()
        .message
        .contains("finite"));
    assert!(set_text_box(&boxed, 0, 0.0, 0.0, -1.0, 1.0)
        .unwrap_err()
        .message
        .contains("w/h"));
    // Ambiguous trailing atoms: not treated as w/h box.
    let weird = page(r#"(text 1 2 3 red blue "x")"#);
    let out2 = set_text_box(&weird, 0, 1.0, 2.0, 5.0, 6.0).unwrap();
    assert!(out2.contains(" 3 5 6 "), "inserts box slots: {out2}");
}

#[test]
fn scale_text_box_missing_string_errors() {
    let src = page("(text 1 2 3)");
    assert!(scale_text_box(&src, 0, 2.0, 2.0)
        .unwrap_err()
        .message
        .contains("string"));
}

#[test]
fn insert_text_box_requires_size_slot() {
    let src = page(r#"(text 1 2 "hi")"#);
    assert!(set_text_box(&src, 0, 1.0, 2.0, 3.0, 4.0).is_err());
}

#[test]
fn set_line_endpoint_and_box_slot_errors() {
    let src = page("(line 0 0 1 1) (rect 0 0 1 1)");
    assert!(set_line_endpoint(&src, 3, 0, 1.0, 1.0).is_err());
    assert!(set_box_xywh(&src, "rect", 2, [1, 2, 3, 4], 0.0, 0.0, 1.0, 1.0).is_err());
    // First slot missing → SyncError (later slots are expect after the first succeeds).
    assert!(set_box_xywh(&src, "rect", 0, [9, 2, 3, 4], 0.0, 0.0, 1.0, 1.0).is_err());
}

#[test]
fn layer_rotation_and_opacity_index_errors() {
    let src = page("(circle 1 2 3)");
    assert!(set_layer_rotation_deg(&src, 0, 4, 10.0, (0.0, 0.0))
        .unwrap_err()
        .message
        .contains("out of range"));
    assert!(set_layer_opacity(&src, 0, 4, 0.5)
        .unwrap_err()
        .message
        .contains("out of range"));
}

#[test]
fn sandwich_with_non_numeric_degrees_rewrites() {
    // Sandwich structure matches, but degrees are not a number → fall through and wrap.
    let src = page("(translate 1 2 (rotate bad (translate -1 -2 (circle 0 0 1))))");
    assert_eq!(layer_rotation_deg(&src, 0, 0).unwrap(), 0.0);
    let out = set_layer_rotation_deg(&src, 0, 0, 15.0, (1.0, 2.0)).unwrap();
    assert!(out.contains("(rotate 15"), "{out}");
}

#[test]
fn bare_rotate_without_numeric_degrees_reads_zero() {
    let src = page("(rotate bad (circle 0 0 1))");
    assert_eq!(layer_rotation_deg(&src, 0, 0).unwrap(), 0.0);
}

#[test]
fn is_center_sandwich_rejects_incomplete_translate() {
    // Not a sandwich: translate without rotate child → wrap path.
    let src = page("(translate 1 2 (circle 0 0 1))");
    let out = set_layer_rotation_deg(&src, 0, 0, 9.0, (1.0, 2.0)).unwrap();
    assert!(
        out.contains("(translate 1 2 (rotate 9 (translate -1 -2 (translate 1 2 (circle 0 0 1)))))")
            || out.contains("(rotate 9"),
        "{out}"
    );
}

#[test]
fn first_shape_child_unknown_under_opacity_wraps() {
    // opacity child is circle (not translate) → wrap whole opacity root.
    let src = page("(opacity 0.5 (circle 1 2 3))");
    let out = nudge_layer_page(&src, 0, 0, 1.0, 1.0).unwrap();
    assert!(out.contains("(translate 1 1 (opacity 0.5"), "{out}");
}

#[test]
fn scale_layer_uniform_page_errors() {
    let src = page("(circle 1 2 3)");
    assert!(scale_layer_uniform(&src, 9, 0, 2.0).is_err());
}

#[test]
fn nudge_polyline_polygon_missing_errors() {
    let src = page("(circle 1 2 3)");
    assert!(nudge_drag_target(&src, DragTarget::PolylineXy(0), 1.0, 0.0).is_err());
    assert!(nudge_drag_target(&src, DragTarget::PolygonXy(0), 1.0, 0.0).is_err());
}

#[test]
fn set_text_box_updates_existing_dims_via_scale() {
    let src = page(r#"(text 10 20 8 40 20 "boxed")"#);
    let out = scale_size_target(&src, SizeTarget::TextSize(0), 2.0).unwrap();
    assert!(out.contains(" 8 "), "font size preserved: {out}");
    assert_ne!(out, src);
}

#[test]
fn collect_under_opacity_and_rotate() {
    let src = page("(opacity 0.2 (rotate 10 (ellipse 1 2 3 4)))");
    assert_eq!(
        collect_drag_targets_page(&src, 0).unwrap(),
        vec![DragTarget::EllipseXy(0)]
    );
    assert_eq!(
        collect_size_targets_page(&src, 0).unwrap(),
        vec![SizeTarget::EllipseRxRy(0)]
    );
}

#[test]
fn frame_and_image_under_inherited_translate() {
    let src = page(r#"(translate 0 0 (frame 1 2 3 4 1) (image "z.png" 1 2 3 4))"#);
    let drag = collect_drag_targets_page(&src, 0).unwrap();
    assert_eq!(
        drag,
        vec![DragTarget::Translate(0), DragTarget::Translate(0)]
    );
}

#[test]
fn set_layer_opacity_clamps_and_patches() {
    let src = page("(opacity 0.2 (circle 1 2 3))");
    let out = set_layer_opacity(&src, 0, 0, 2.0).unwrap();
    assert!(out.contains("(opacity 1 "), "{out}");
}

#[test]
fn scale_ellipse_second_axis_error_after_first() {
    // After scaling rx, missing ry on a truncated ellipse fails the second multiply.
    let src = page("(ellipse 0 0 4)");
    assert!(scale_size_target(&src, SizeTarget::EllipseRxRy(0), 2.0).is_err());
}

#[test]
fn multiply_optional_line_slot_absent_ok() {
    let src = page("(line 0 0 2 2)");
    let out = scale_size_target(&src, SizeTarget::LineSeg(0), 2.0).unwrap();
    assert!(out.contains("line -1"), "{out}");
}

#[test]
fn set_opacity_missing_alpha_on_bad_token() {
    let src = page("(opacity bad (circle 1 2 3))");
    assert!(layer_opacity(&src, 0, 0)
        .unwrap_err()
        .message
        .contains("missing alpha"));
    assert!(set_layer_opacity(&src, 0, 0, 0.3)
        .unwrap_err()
        .message
        .contains("missing alpha"));
}

#[test]
fn scale_second_polyline_trailing_width() {
    let src = page(
        "(polyline 0 0 1 0 1 1) (polyline 0 0 10 0 10 10 red 2)",
    );
    let out = scale_size_target(&src, SizeTarget::PolylinePoints(1), 2.0).unwrap();
    assert!(out.contains(" 4)"), "{out}");
}

#[test]
fn nudge_pair_y_not_numeric_errors() {
    let src = page("(circle 1 y 3)");
    assert!(nudge_drag_target(&src, DragTarget::CircleXy(0), 1.0, 0.0).is_err());
}

#[test]
fn rotate_with_token_before_body_still_peels() {
    let src = page("(rotate 10 999 (circle 0 0 2))");
    let out = set_layer_rotation_deg(&src, 0, 0, 20.0, (0.0, 0.0)).unwrap();
    assert!(out.contains("(rotate 20"), "{out}");
}

#[test]
fn translate_token_child_is_not_center_sandwich() {
    let src = page("(translate 1 2 999 (circle 0 0 1))");
    let out = set_layer_rotation_deg(&src, 0, 0, 8.0, (1.0, 2.0)).unwrap();
    assert!(out.contains("(rotate 8"), "{out}");
    let nudged = nudge_layer_page(&src, 0, 0, 1.0, 0.0).unwrap();
    assert!(nudged.contains("(translate 2 2"), "{nudged}");
}

#[test]
fn scale_text_box_missing_text_errors() {
    let src = page("(circle 1 2 3)");
    assert!(scale_text_box(&src, 0, 2.0, 2.0)
        .unwrap_err()
        .message
        .contains("text"));
}

#[test]
fn scale_text_box_picks_nth_text() {
    let src = page(r#"(text 0 0 8 "a") (text 1 1 8 10 10 "b")"#);
    let out = scale_text_box(&src, 1, 2.0, 2.0).unwrap();
    assert!(out.contains(r#""b""#), "{out}");
    assert_ne!(out, src);
}

#[test]
fn first_shape_child_skips_tokens_under_translate() {
    let src = page("(translate 0 0 999 (rotate 12 (circle 0 0 1)))");
    assert_eq!(layer_rotation_deg(&src, 0, 0).unwrap(), 12.0);
}

#[test]
fn scale_ellipse_axes_second_multiply() {
    let src = page("(ellipse 0 0 4 8)");
    let out = scale_size_target_axes(&src, SizeTarget::EllipseRxRy(0), 2.0, 0.5).unwrap();
    assert!(out.contains("ellipse 0 0 8 4"), "{out}");
}

#[test]
fn collect_layers_page_error_propagates_to_geometry() {
    let src = page("(circle 1 2 3)");
    assert!(layer_rotation_deg(&src, 3, 0).is_err());
    assert!(set_layer_rotation_deg(&src, 3, 0, 1.0, (0.0, 0.0)).is_err());
    assert!(layer_opacity(&src, 3, 0).is_err());
    assert!(set_layer_opacity(&src, 3, 0, 0.5).is_err());
}

#[test]
fn nudge_line_second_pair_via_drag_target() {
    let src = page("(line 9 0 10 0)");
    // Digit growth on first endpoint x, then second pair still patches.
    let out = nudge_drag_target(&src, DragTarget::LineXy(0), 1.0, 0.0).unwrap();
    assert!(out.contains("line 10 0 11 0"), "{out}");
}

#[test]
fn insert_text_box_parse_after_xy() {
    let src = page(r#"(text 9 20 8 "grow")"#);
    let out = set_text_box(&src, 0, 10.0, 20.0, 30.0, 40.0).unwrap();
    assert!(out.contains(" 8 30 40 "), "{out}");
}

#[test]
fn scale_size_target_ellipse_chain_error() {
    let src = page("(ellipse 0 0 4)");
    assert!(scale_size_target_axes(&src, SizeTarget::EllipseRxRy(0), 2.0, 2.0).is_err());
}

#[test]
fn set_text_box_parse_error() {
    assert!(set_text_box("(", 0, 1.0, 1.0, 1.0, 1.0).is_err());
}

#[test]
fn scale_box_axes_missing_first_slot() {
    let src = page("(rect)");
    assert!(scale_box_axes(&src, "rect", 0, [1, 2, 3, 4], 2.0, 2.0).is_err());
}

#[test]
fn scale_text_partial_slots_error() {
    // String exists so content resolves; non-numeric x fails the first number read.
    assert!(scale_text_box(&page(r#"(text x 2 3 "a")"#), 0, 2.0, 2.0).is_err());
}

#[test]
fn set_text_box_on_empty_text_form_errors() {
    assert!(set_text_box(&page("(text)"), 0, 1.0, 2.0, 3.0, 4.0).is_err());
}

#[test]
fn scale_ellipse_and_polyline_missing_errors() {
    assert!(scale_size_target(&page("(ellipse 0 0)"), SizeTarget::EllipseRxRy(0), 2.0).is_err());
    assert!(scale_size_target(&page("(circle 1 2 3)"), SizeTarget::PolylinePoints(0), 2.0).is_err());
    assert!(scale_size_target_axes(&page("(ellipse 0 0)"), SizeTarget::EllipseRxRy(0), 2.0, 2.0).is_err());
}

#[test]
fn helper_parse_errors_on_truncated_src() {
    assert!(scale_size_target("(", SizeTarget::CircleR(0), 2.0).is_err());
    assert!(scale_box_axes("(", "rect", 0, [1, 2, 3, 4], 2.0, 2.0).is_err());
    assert!(nudge_drag_target("(", DragTarget::LineXy(0), 1.0, 0.0).is_err());
    assert!(nudge_drag_target("(", DragTarget::PolylineXy(0), 1.0, 0.0).is_err());
    assert!(nudge_drag_target("(", DragTarget::PolygonXy(0), 1.0, 0.0).is_err());
}

#[test]
fn empty_list_descendants_skipped_while_finding_text() {
    let src = page(r#"() (text 1 2 3 "x")"#);
    let out = set_text_box(&src, 0, 2.0, 3.0, 4.0, 5.0).unwrap();
    assert!(out.contains(r#"(text 2 3 3 4 5 "x")"#), "{out}");
}

#[test]
fn collect_size_skips_tokens_under_rotate() {
    let src = page("(rotate 10 999 (circle 1 2 3))");
    assert_eq!(
        collect_size_targets_page(&src, 0).unwrap(),
        vec![SizeTarget::CircleR(0)]
    );
}

#[test]
fn nudge_poly_digit_growth_all_vertices() {
    let src = page("(polyline 9 0 10 0) (polygon 9 0 10 0 9 1)");
    let out = nudge_drag_target(&src, DragTarget::PolylineXy(0), 1.0, 0.0).unwrap();
    assert!(out.contains("polyline 10 0 11 0"), "{out}");
    let out2 = nudge_drag_target(&src, DragTarget::PolygonXy(0), 1.0, 0.0).unwrap();
    assert!(out2.contains("polygon 10 0 11 0 10 1"), "{out2}");
}

#[test]
fn scale_text_box_dims_err_via_missing_index() {
    // Missing text index fails in read_nth_text_content (called first).
    let src = page(r#"(text 1 2 3 "only")"#);
    assert!(scale_text_box(&src, 1, 2.0, 2.0)
        .unwrap_err()
        .message
        .contains("text"));
}

#[test]
fn scale_line_seg_and_set_nth_parse_errors() {
    assert!(scale_size_target("(", SizeTarget::LineSeg(0), 2.0).is_err());
    assert!(scale_size_target("(", SizeTarget::PolylinePoints(0), 2.0).is_err());
    assert!(scale_size_target("(", SizeTarget::PolygonPoints(0), 2.0).is_err());
    assert!(set_line_endpoint("(", 0, 0, 1.0, 1.0).is_err());
    assert!(set_nth_via_box_parse_error());
}

fn set_nth_via_box_parse_error() -> bool {
    set_box_xywh("(", "rect", 0, [1, 2, 3, 4], 0.0, 0.0, 1.0, 1.0).is_err()
}

#[test]
fn empty_lists_skipped_in_scale_text_and_polyline() {
    let src = page(r#"() (text 0 0 10 "abcdefghij")"#);
    let out = scale_text_box(&src, 0, 2.0, 1.0).unwrap();
    assert_ne!(out, src);
    let src2 = page("() (polyline 0 0 10 0 10 10 red 2)");
    let out2 = scale_size_target(&src2, SizeTarget::PolylinePoints(0), 2.0).unwrap();
    assert!(out2.contains(" 4)"), "{out2}");
}

#[test]
fn nudge_polyline_odd_leading_coords_nudge_complete_pairs() {
    // Odd trailing non-number stops the count; only complete pairs move.
    let src = page("(polyline 0 0 10 0 extra)");
    let out = nudge_drag_target(&src, DragTarget::PolylineXy(0), 1.0, 0.0).unwrap();
    assert!(out.contains("polyline 1 0 11 0"), "{out}");
}

#[test]
fn rotation_reads_translate_with_token_slot_three() {
    // Slot 3 is a token, not a rotate list — walk into the circle (no angle).
    let src = page("(translate 1 2 999 (circle 0 0 1))");
    assert_eq!(layer_rotation_deg(&src, 0, 0).unwrap(), 0.0);
}

#[test]
fn empty_list_skipped_when_counting_poly_coords() {
    let src = page("() (polyline 0 0 4 0)");
    let out = nudge_drag_target(&src, DragTarget::PolylineXy(0), 1.0, 0.0).unwrap();
    assert!(out.contains("polyline 1 0 5 0"), "{out}");
}

#[test]
fn find_number_pair_skips_empty_list() {
    let src = page("() (circle 1 2 3)");
    let out = nudge_drag_target(&src, DragTarget::CircleXy(0), 1.0, 1.0).unwrap();
    assert!(out.contains("(circle 2 3 3)"), "{out}");
}
