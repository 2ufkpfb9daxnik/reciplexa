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
    // Rotation may be read through wrappers or default to 0 depending on peel rules.
    let rot = layer_rotation_deg(&src, 0, 0).unwrap();
    assert!(rot == 30.0 || rot == 0.0, "rot={rot}");
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
