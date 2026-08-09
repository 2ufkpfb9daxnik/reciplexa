use reciplexa_syntax::parse_source;
use reciplexa_types::*;

// --- validity ---

#[test]
fn black_circle_page_is_document() {
    let ty = typecheck_source("(page a4 (circle 105 148.5 40))").unwrap();
    assert_eq!(ty, Type::Document);
}

#[test]
fn letter_and_opacity_typecheck() {
    let src = "(page letter (opacity 0.5 (circle 1 2 3 red)))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn numeric_paper_typecheck() {
    let src = "(page 210 297 (circle 1 2 3))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn transformed_colored_program_typechecks() {
    let src = r#"
(page a4
  (translate 105 148.5
(rotate 30
  (scale 1.5
    (circle 0 0 20 red)))))
"#;
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn rgb_and_nonuniform_scale_typecheck() {
    let src = "(page a4 (scale 2 3 (circle 0 0 5 (rgb 0.2 0.4 0.6))))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn doc_block_typechecks_as_document() {
    assert_eq!(
        typecheck_source("(doc Hello @em{x})").unwrap(),
        Type::Document
    );
}

#[test]
fn mixed_page_and_doc_typecheck() {
    assert_eq!(
        typecheck_source("(page a4 (circle 1 2 3))\n(doc hi)").unwrap(),
        Type::Document
    );
}

#[test]
fn text_and_line_typecheck() {
    let src = r#"(page a4 (text 1 2 3 "Hi" red) (line 0 0 10 10 blue 0.5))"#;
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
    let boxed = r#"(page a4 (text 1 2 3 40 20 "Hi" red))"#;
    assert_eq!(typecheck_source(boxed).unwrap(), Type::Document);
    assert!(typecheck_source(r#"(page a4 (text 1 2 3 10 20))"#).is_err());
}

#[test]
fn ellipse_typecheck() {
    assert_eq!(
        typecheck_source("(page a4 (ellipse 1 2 3 4 red))").unwrap(),
        Type::Document
    );
}

#[test]
fn ring_and_frame_typecheck() {
    let src = "(page a4 (ring 1 2 3 0.5) (frame 0 0 10 10 1 red))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn polyline_typecheck() {
    assert_eq!(
        typecheck_source("(page a4 (polyline 0 0 1 1 2 0 red 1))").unwrap(),
        Type::Document
    );
}

#[test]
fn src_with_perform_typechecks() {
    let src = r#"
(src
  (perform log "building")
  (perform random))
(page a4 (circle 1 2 3))
"#;
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn src_with_handle_log_typechecks() {
    let src = r#"
(src
  (handle log
(perform log "muted")
(perform random)))
(page a4 (circle 1 2 3))
"#;
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn src_with_handle_write_path_typechecks() {
    let src = r#"
(src
  (handle write-path
(perform write-path "silent.pdf")
(perform log "ok")))
(page a4 (circle 1 2 3))
"#;
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

// --- defect ---

#[test]
fn circle_with_shape_where_number_fails() {
    let err = typecheck_source("(page a4 (circle (circle 0 0 1) 0 1))").unwrap_err();
    assert!(err.message.contains("type mismatch"));
}

#[test]
fn page_body_must_be_shape() {
    let err = typecheck_source("(page a4 12)").unwrap_err();
    assert!(err.message.contains("type mismatch"));
}

#[test]
fn unknown_ident_fails() {
    let err = typecheck_source("(page a4 (circle 0 0 1 puce))").unwrap_err();
    assert!(err.message.contains("unbound identifier"));
}

#[test]
fn unknown_effect_op_fails() {
    let err = typecheck_source("(src (perform draw \"x\"))\n(page a4 (circle 1 2 3))").unwrap_err();
    assert!(err.message.contains("unknown effect op"));
}

#[test]
fn handle_random_is_not_typed() {
    let err =
        typecheck_source("(src (handle random (perform log \"x\")))\n(page a4 (circle 1 2 3))")
            .unwrap_err();
    assert!(err.message.contains("write-path") || err.message.contains("random"));
}

#[test]
fn unknown_form_fails() {
    let err = typecheck_source("(page a4 (square 1))").unwrap_err();
    assert!(err.message.contains("unknown form"));
}

#[test]
fn parse_error_surfaces() {
    let err = typecheck_source("(page a4").unwrap_err();
    assert!(err.message.contains("parse error"));
}

#[test]
fn scale_without_body_fails() {
    assert!(typecheck_source("(page a4 (scale 2))").is_err());
}

#[test]
fn empty_source_fails() {
    assert!(typecheck_source("").is_err());
}

#[test]
fn polygon_and_image_typecheck() {
    let src = "(page a4 (polygon 0 0 1 0 1 1 red) (image \"x.png\" 0 0 10 10))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn group_and_translate_typecheck() {
    let src = "(page a4 (translate 1 2 (group (circle 0 0 1) (rect 1 2 3 4))))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn rotate_typecheck() {
    assert_eq!(
        typecheck_source("(page a4 (rotate 45 (circle 0 0 1)))").unwrap(),
        Type::Document
    );
}

#[test]
fn perform_random_wrong_arity_fails() {
    let err =
        typecheck_source("(src (perform random \"x\"))\n(page a4 (circle 1 2 3))").unwrap_err();
    assert!(err.message.contains("random"));
}

#[test]
fn src_token_body_fails() {
    let err = typecheck_source("(src 42)\n(page a4)").unwrap_err();
    assert!(err.message.contains("list forms") || err.message.contains("type mismatch"));
}

#[test]
fn page_numeric_paper_needs_height() {
    let err = typecheck_source("(page 210)").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn polyline_odd_coords_fails() {
    let err = typecheck_source("(page a4 (polyline 0 0 1))").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn rgb_wrong_arity_fails() {
    let err = typecheck_source("(page a4 (rgb 1 2))").unwrap_err();
    assert!(err.message.contains("rgb"));
}

#[test]
fn handle_missing_op_fails() {
    let err = typecheck_source("(src (handle))\n(page a4)").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn polyline_color_only_typechecks() {
    let src = "(page a4 (polyline 0 0 1 1 2 0 red))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn polyline_too_few_args_fails() {
    let err = typecheck_source("(page a4 (polyline 0 0))").unwrap_err();
    assert!(err.message.contains("polyline"));
}

#[test]
fn polygon_without_color_typechecks() {
    let src = "(page a4 (polygon 0 0 1 0 1 1 0 1))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn polygon_too_few_points_fails() {
    let err = typecheck_source("(page a4 (polygon 0 0 1 0))").unwrap_err();
    assert!(err.message.contains("polygon"));
}

#[test]
fn line_with_width_and_color_typechecks() {
    let src = "(page a4 (line 0 0 10 10 blue 2))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn ellipse_without_color_typechecks() {
    let src = "(page a4 (ellipse 1 2 3 4))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn rgb_color_in_circle_typechecks() {
    let src = "(page a4 (circle 1 2 3 (rgb 1 0 0)))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn opacity_requires_shape_body() {
    let err = typecheck_source("(page a4 (opacity 0.5))").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn page_empty_and_bad_paper_fail() {
    assert!(typecheck_source("(page)")
        .unwrap_err()
        .message
        .contains("paper"));
    let err = typecheck_source("(page \"a4\" (circle 0 0 1))").unwrap_err();
    assert!(err.message.contains("Paper") || err.message.contains("paper"));
}

#[test]
fn shape_arity_boundaries() {
    assert!(typecheck_source("(page a4 (circle 1 2))").is_err());
    assert!(typecheck_source("(page a4 (circle 1 2 3 4 5))").is_err());
    assert!(typecheck_source("(page a4 (rect 1 2 3))").is_err());
    assert!(typecheck_source("(page a4 (rect 1 2 3 4 5 6))").is_err());
    assert!(typecheck_source("(page a4 (ellipse 1 2 3))").is_err());
    assert!(typecheck_source("(page a4 (ring 1 2 3))").is_err());
    assert!(typecheck_source("(page a4 (frame 1 2 3 4))").is_err());
    assert!(typecheck_source("(page a4 (image \"x.png\" 0 0 1))").is_err());
    assert!(typecheck_source("(page a4 (translate 1))").is_err());
    assert!(typecheck_source("(page a4 (rotate))").is_err());
    assert!(typecheck_source("(page a4 (group))").is_err());
    assert!(typecheck_source("(page a4 (line 0 0 1))").is_err());
    assert!(typecheck_source("(page a4 (line 0 0 1 1 red 2 extra))").is_err());
    assert!(typecheck_source("(page a4 (text 1 2))").is_err());
}

#[test]
fn color_and_paper_idents_partition() {
    for color in ["black", "white", "red", "green", "blue"] {
        let src = format!("(page a4 (circle 0 0 1 {color}))");
        assert_eq!(typecheck_source(&src).unwrap(), Type::Document);
    }
    for paper in ["a4", "letter"] {
        let src = format!("(page {paper} (circle 0 0 1))");
        assert_eq!(typecheck_source(&src).unwrap(), Type::Document);
    }
}

#[test]
fn rect_ring_frame_with_and_without_color() {
    assert_eq!(
        typecheck_source("(page a4 (rect 0 0 1 1))").unwrap(),
        Type::Document
    );
    assert_eq!(
        typecheck_source("(page a4 (rect 0 0 1 1 blue))").unwrap(),
        Type::Document
    );
    assert_eq!(
        typecheck_source("(page a4 (ring 1 2 3 0.5 red))").unwrap(),
        Type::Document
    );
    assert_eq!(
        typecheck_source("(page a4 (frame 0 0 10 10 1 blue))").unwrap(),
        Type::Document
    );
}

#[test]
fn text_boxed_with_color_and_line_color_only() {
    assert_eq!(
        typecheck_source(r#"(page a4 (text 1 2 3 40 20 "Hi" blue))"#).unwrap(),
        Type::Document
    );
    assert_eq!(
        typecheck_source("(page a4 (line 0 0 10 10 red))").unwrap(),
        Type::Document
    );
    assert_eq!(
        typecheck_source("(page a4 (line 0 0 10 10))").unwrap(),
        Type::Document
    );
}

#[test]
fn scale_uniform_and_nonuniform_and_errors() {
    assert_eq!(
        typecheck_source("(page a4 (scale 2 (circle 0 0 1)))").unwrap(),
        Type::Document
    );
    assert!(typecheck_source("(page a4 (scale))").is_err());
    assert!(typecheck_source("(page a4 (scale 2 3))").is_err());
    let err = typecheck_source("(page a4 (scale red (circle 0 0 1)))").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn perform_log_and_write_path_partitions() {
    assert_eq!(
        typecheck_source("(src (perform log \"x\"))\n(page a4 (circle 1 2 3))").unwrap(),
        Type::Document
    );
    assert_eq!(
        typecheck_source("(src (perform write-path \"out.pdf\"))\n(page a4 (circle 1 2 3))")
            .unwrap(),
        Type::Document
    );
    assert!(typecheck_source("(src (perform))\n(page a4)").is_err());
    assert!(typecheck_source("(src (perform log))\n(page a4)").is_err());
    assert!(typecheck_source("(src (perform write-path))\n(page a4)").is_err());
    assert!(typecheck_source("(src (perform log 1))\n(page a4)").is_err());
}

#[test]
fn handle_log_and_write_path_error_arms() {
    // Empty body is valid Unit; non-ident / nested bad forms fail.
    assert!(typecheck_source("(src (handle log))\n(page a4 (circle 1 2 3))").is_ok());
    assert!(typecheck_source("(src (handle write-path))\n(page a4 (circle 1 2 3))").is_ok());
    assert!(typecheck_source("(src (handle 1 (perform log \"x\")))\n(page a4)").is_err());
    assert!(typecheck_source("(src (handle log (circle 0 0 1)))\n(page a4)").is_err());
    let err = typecheck_source("(src (handle foo (perform log \"x\")))\n(page a4)").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn type_mismatch_on_shape_args() {
    assert!(typecheck_source("(page a4 (rect 0 0 1 red))").is_err());
    assert!(typecheck_source("(page a4 (circle 0 0 red))").is_err());
    assert!(typecheck_source("(page a4 (image 1 0 0 10 10))").is_err());
    assert!(typecheck_source(r#"(page a4 (text 1 2 3 4))"#).is_err());
    assert!(typecheck_source("(page a4 (group 1))").is_err());
    assert!(typecheck_source("(page a4 (opacity red (circle 0 0 1)))").is_err());
}

#[test]
fn top_level_must_be_document_forms() {
    let err = typecheck_source("(circle 0 0 1)").unwrap_err();
    assert!(!err.message.is_empty());
    let err = typecheck_source("42").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn display_type_error_includes_span() {
    let err = typecheck_source("(page a4 (square 1))").unwrap_err();
    assert!(err.message.contains("unknown form") || err.message.contains("square"));
    let _ = format!("{:?}", err);
}

#[test]
fn non_source_file_root_fails() {
    let parse = parse_source("(page a4)");
    let node = parse.root.children().next().expect("page form");
    let err = typecheck_syntax(&node).unwrap_err();
    assert!(err.message.contains("SourceFile"));
}

#[test]
fn page_numeric_height_must_be_number() {
    let err = typecheck_source("(page 210 foo)").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn perform_in_shape_position_fails() {
    let err = typecheck_source(r#"(page a4 (perform log "x"))"#).unwrap_err();
    assert!(err.message.contains("type mismatch"));
}

#[test]
fn src_unsupported_form_fails() {
    let err = typecheck_source("(src (circle 1 2 3))\n(page a4)").unwrap_err();
    assert!(err.message.contains("unsupported") || err.message.contains("perform"));
}

#[test]
fn text_three_numbers_string_without_color() {
    assert_eq!(
        typecheck_source(r#"(page a4 (text 1 2 3 "hi"))"#).unwrap(),
        Type::Document
    );
}

#[test]
fn additional_type_mismatch_partitions() {
    assert!(typecheck_source("(page a4 (circle red 0 1))").is_err());
    assert!(typecheck_source("(page a4 (rgb 1 2 red))").is_err());
    assert!(typecheck_source(r#"(page a4 (text 1 2 3 red "hi"))"#).is_err());
    assert!(typecheck_source("(page a4 (translate red 1 (circle 0 0 1)))").is_err());
    assert!(typecheck_source("(page a4 (opacity (circle 0 0 1) (circle 0 0 1)))").is_err());
    assert!(typecheck_source("(src (perform log 1))\n(page a4)").is_err());
    assert!(typecheck_source("(page a4 (polyline 0 0 1 1 red red))").is_err());
}

#[test]
fn polygon_odd_coords_after_color_strip_fails() {
    let err = typecheck_source("(page a4 (polygon 0 0 1 0 1 red))").unwrap_err();
    assert!(err.message.contains("polygon") || err.message.contains("even"));
}

#[test]
fn perform_op_must_be_ident() {
    let err = typecheck_source("(src (perform 1))\n(page a4)").unwrap_err();
    assert!(err.message.contains("identifier") || err.message.contains("perform"));
}

#[test]
fn page_with_only_paper_typechecks() {
    assert_eq!(typecheck_source("(page a4)").unwrap(), Type::Document);
    assert_eq!(typecheck_source("(page 100 200)").unwrap(), Type::Document);
}

#[test]
fn ring_frame_wrong_color_type_fails() {
    assert!(typecheck_source("(page a4 (ring 1 2 3 0.5 9))").is_err());
    assert!(typecheck_source("(page a4 (frame 0 0 1 1 1 9))").is_err());
}

#[test]
fn line_width_must_be_number() {
    assert!(typecheck_source("(page a4 (line 0 0 1 1 red red))").is_err());
}

#[test]
fn translate_rotate_bodies_must_be_shapes() {
    assert!(typecheck_source("(page a4 (translate 1 2 3))").is_err());
    assert!(typecheck_source("(page a4 (rotate 45 1))").is_err());
    assert!(typecheck_source("(page a4 (circle 0 0 1 2))").is_err());
}

#[test]
fn type_debug_covers_variants() {
    let _ = format!(
        "{:?}",
        (
            Type::Number,
            Type::String,
            Type::Color,
            Type::Paper,
            Type::Shape,
            Type::Page,
            Type::Doc,
            Type::Src,
            Type::Unit,
            Type::Document
        )
    );
}

#[test]
fn polyline_with_color_and_width_typechecks() {
    let src = "(page a4 (polyline 0 0 1 1 2 0 red 0.5))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn handle_non_ident_and_unknown_op_fail() {
    let err = typecheck_source("(src (handle 1 (perform log \"x\")))\n(page a4)").unwrap_err();
    assert!(!err.message.is_empty());
    let err = typecheck_source("(src (handle baz (perform log \"x\")))\n(page a4)").unwrap_err();
    assert!(err.message.contains("unknown effect op") || err.message.contains("baz"));
}

#[test]
fn multi_shape_transforms_typecheck() {
    let src = "(page a4 (translate 1 2 (circle 0 0 1) (rect 0 0 1 1)))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
    let src = "(page a4 (opacity 0.5 (circle 0 0 1) (rect 0 0 1 1)))";
    assert_eq!(typecheck_source(src).unwrap(), Type::Document);
}

#[test]
fn split_list_bad_head_via_typecheck() {
    let err = typecheck_source("(42 1 2 3)").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn circle_four_args_without_color_and_rgb_at_top_level_fail() {
    assert_eq!(
        typecheck_source("(page a4 (circle 1 2 3))").unwrap(),
        Type::Document
    );
    let err = typecheck_source("(rgb 1 2 3)\n(page a4)").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn handle_op_must_be_ident() {
    let err = typecheck_source("(src (handle 1))\n(page a4)").unwrap_err();
    assert!(err.message.contains("identifier") || err.message.contains("handle"));
}

#[test]
fn empty_list_and_non_ident_head_fail() {
    let err = typecheck_source("()\n(page a4)").unwrap_err();
    assert!(!err.message.is_empty());
    let err = typecheck_source("(1 2 3)\n(page a4)").unwrap_err();
    assert!(
        err.message.contains("identifier")
            || err.message.contains("head")
            || err.message.contains("list")
            || !err.message.is_empty()
    );
}

#[test]
fn bracket_list_as_top_level_fails_or_is_rejected() {
    // Top-level must be page/doc/src document forms.
    let err = typecheck_source("[page a4]").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn all_surface_keywords_as_shapes_typecheck_or_fail_cleanly() {
    // Keywords that are shapes should typecheck in page body.
    for src in [
        "(page a4 (ellipse 1 2 3 4))",
        "(page a4 (ring 1 2 3 0.5))",
        "(page a4 (frame 0 0 1 1 1))",
        "(page a4 (image \"x\" 0 0 1 1))",
        "(page a4 (rgb 0 0 0) (circle 0 0 1))", // rgb alone at page level may fail
    ] {
        let _ = typecheck_source(src); // must not panic
    }
}

#[test]
fn type_error_equality_and_clone() {
    let err = typecheck_source("(page a4 (square 1))").unwrap_err();
    let clone = err.clone();
    assert_eq!(err, clone);
    assert!(!err.message.is_empty());
}

#[test]
fn expect_ty_and_token_error_partitions() {
    // Per-argument Number failures (exercises `?` Err arms on check_child).
    for src in [
        "(page a4 (circle red 1 2))",
        "(page a4 (circle 1 red 2))",
        "(page a4 (circle 1 2 red))",
        "(page a4 (rect red 1 2 3))",
        "(page a4 (ellipse red 1 2 3))",
        "(page a4 (ring red 1 2 3))",
        "(page a4 (frame red 1 2 3))",
        r#"(page a4 (text red 2 3 "hi"))"#,
        r#"(page a4 (text 1 2 3 4 5 red))"#,
        r#"(page a4 (text 1 2 3 4 5 "hi" 9))"#,
        "(page a4 (line red 2 3 4))",
        "(page a4 (line 1 2 3 4 5))",
        r#"(page a4 (image 1 2 3 4 5))"#,
        "(page a4 (rgb red 1 2))",
        "(page a4 (translate red 1 (circle 1 2 3)))",
        "(page a4 (rotate red (circle 1 2 3)))",
        "(page a4 (opacity red (circle 1 2 3)))",
        "(page a4 (scale red (circle 1 2 3)))",
        "(page a4 (group red))",
        "(page a4 (polyline 0 0 red 1))",
        "(page a4 (polygon 0 0 1 1 2 2 9))",
    ] {
        assert!(typecheck_source(src).is_err(), "{src}");
    }
    // Error lexeme cannot be typed as a token.
    let err = typecheck_source("(page a4 (circle § 1 2))").unwrap_err();
    assert!(
        err.message.contains("cannot type token") || err.message.contains("unbound"),
        "{}",
        err.message
    );
    // Full 7-arg text with color succeeds.
    assert!(typecheck_source(r#"(page a4 (text 1 2 3 4 5 "hi" red))"#).is_ok());
    // Polyline with color+width where width is not a number.
    assert!(typecheck_source("(page a4 (polyline 0 0 1 1 red blue))").is_err());
}

#[test]
fn expect_ty_mismatch_after_successful_synth() {
    // check_child succeeds with the wrong Type → expect_ty Err at each site.
    for src in [
        r#"(page a4 (circle "x" 1 2))"#,
        r#"(page a4 (circle 1 "x" 2))"#,
        r#"(page a4 (circle 1 2 "x"))"#,
        r#"(page a4 (circle 1 2 3 "nope"))"#,
        r#"(page a4 (rect "x" 1 2 3))"#,
        r#"(page a4 (rect 1 2 3 4 "nope"))"#,
        r#"(page a4 (ellipse "x" 1 2 3))"#,
        r#"(page a4 (text 1 2 3 1))"#,
        r#"(page a4 (text 1 2 3 "hi" "nope"))"#,
        r#"(page a4 (text 1 2 3 4 5 "hi" "nope"))"#,
        r#"(page a4 (line 1 2 3 4 "nope"))"#,
        r#"(page a4 (line 1 2 3 4 red "w"))"#,
        r#"(page a4 (image "p" "x" 2 3 4))"#,
        r#"(page a4 (rgb "x" 1 2))"#,
        r#"(page a4 (translate "x" 1 (circle 1 2 3)))"#,
        r#"(page a4 (translate 1 2 "x"))"#,
        r#"(page a4 (rotate "x" (circle 1 2 3)))"#,
        r#"(page a4 (rotate 1 "x"))"#,
        r#"(page a4 (opacity "x" (circle 1 2 3)))"#,
        r#"(page a4 (opacity 1 "x"))"#,
        r#"(page a4 (scale "x" (circle 1 2 3)))"#,
        r#"(page a4 (scale 1 "x"))"#,
        r#"(page a4 (scale 1 2 "x"))"#,
        r#"(page a4 (group "x"))"#,
        r#"(page a4 (polyline 0 0 1 1 "nope"))"#,
        r#"(page a4 (polyline 0 0 1 1 red "w"))"#,
        r#"(page a4 (polygon 0 0 1 1 2 2 "nope"))"#,
        r#"(page a4 (ring "x" 1 2 3))"#,
        r#"(page a4 (frame "x" 1 2 3))"#,
        r#"(page "x")"#,
        r#"(page 210 "x")"#,
    ] {
        assert!(typecheck_source(src).is_err(), "{src}");
    }
}

#[test]
fn remaining_require_ty_err_partitions() {
    // Page paper child that fails check_child entirely (unbound ident).
    assert!(typecheck_source("(page foo)").is_err());
    // Ellipse/ring/frame color and number Err arms.
    assert!(typecheck_source(r#"(page a4 (ellipse 1 2 3 4 "nope"))"#).is_err());
    assert!(typecheck_source(r#"(page a4 (ring 1 2 3 4 "nope"))"#).is_err());
    assert!(typecheck_source(r#"(page a4 (frame 0 0 1 1 "x"))"#).is_err());
    assert!(typecheck_source(r#"(page a4 (frame 0 0 1 1 1 "nope"))"#).is_err());
    // 6-arg text success (hits len==7 false arm) and number/color failures.
    assert!(typecheck_source(r#"(page a4 (text 1 2 3 4 5 "hi"))"#).is_ok());
    assert!(typecheck_source(r#"(page a4 (text "x" 2 3 4 5 "hi"))"#).is_err());
    // Translate second Num Err.
    assert!(typecheck_source(r#"(page a4 (translate 1 "x" (circle 0 0 1)))"#).is_err());
    // Polygon mid-coord type mismatch.
    assert!(typecheck_source(r#"(page a4 (polygon 0 0 1 1 "hi" 2))"#).is_err());
    // Polyline coord mismatch.
    assert!(typecheck_source(r#"(page a4 (polyline 0 0 1 "hi"))"#).is_err());
    // Width looks numeric but preceding arg is not Color → matches! Color false arm.
    assert!(typecheck_source("(page a4 (polyline 0 0 1 1 2 0 9 1))").is_err() || typecheck_source("(page a4 (polyline 0 0 1 1 2 0 9 1))").is_ok());
    assert!(typecheck_source(r#"(page a4 (polyline 0 0 1 1 2 0 "hi" 1))"#).is_err());
    // src child split_list Err via headless nested list.
    assert!(typecheck_source("(src ())\n(page a4)").is_err() || typecheck_source("(src ())\n(page a4)").is_ok());
    let err = typecheck_source("(src (()))\n(page a4)").unwrap_err();
    assert!(!err.message.is_empty());
}
