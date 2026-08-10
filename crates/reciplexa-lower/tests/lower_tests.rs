use reciplexa_lower::*;
use reciplexa_pdf::document_to_pdf;
use reciplexa_scene::{Affine, Color, PaperSize, Shape};
use reciplexa_syntax::parse_source;

const BLACK_CIRCLE: &str = r#"
(// A4 black circle (M3))
(page a4
  (circle 105 148.5 40))
"#;

const TRANSFORMED: &str = r#"
(page a4
  (translate 105 148.5
(rotate 30
  (scale 1.5
    (circle 0 0 20 red)))))
"#;

// --- validity ---

#[test]
fn lowers_a4_black_circle() {
    let doc = lower_source(BLACK_CIRCLE).expect("lower");
    assert_eq!(doc.pages.len(), 1);
    let page = &doc.pages[0];
    assert_eq!(page.paper, PaperSize::a4());
    match &page.shapes[0] {
        Shape::Circle(c) => {
            assert_eq!(c.x_mm, 105.0);
            assert_eq!(c.y_mm, 148.5);
            assert_eq!(c.radius_mm, 40.0);
            assert_eq!(c.fill, Color::BLACK);
        }
        _ => panic!("expected circle"),
    }
}

#[test]
fn lowers_nested_transforms_and_named_color() {
    let doc = lower_source(TRANSFORMED).unwrap();
    let outer = &doc.pages[0].shapes[0];
    match outer {
        Shape::Group {
            transform,
            children,
        } => {
            assert_eq!(*transform, Affine::translate(105.0, 148.5));
            match &children[0] {
                Shape::Group { children, .. } => match &children[0] {
                    Shape::Group { children, .. } => match &children[0] {
                        Shape::Circle(c) => {
                            assert_eq!(c.fill, Color::RED);
                            assert_eq!(c.radius_mm, 20.0);
                        }
                        _ => panic!("expected circle"),
                    },
                    _ => panic!("expected scale group"),
                },
                _ => panic!("expected rotate group"),
            }
        }
        _ => panic!("expected translate group"),
    }
}

#[test]
fn lowers_rgb_and_nonuniform_scale() {
    let src = "(page a4 (scale 2 3 (circle 0 0 5 (rgb 0.2 0.4 0.6))))";
    let doc = lower_source(src).unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Group {
            transform,
            children,
        } => {
            assert_eq!(*transform, Affine::scale(2.0, 3.0));
            match &children[0] {
                Shape::Circle(c) => assert_eq!(c.fill, Color::new(0.2, 0.4, 0.6)),
                _ => panic!("expected circle"),
            }
        }
        _ => panic!("expected group"),
    }
}

#[test]
fn end_to_end_transformed_source_to_pdf() {
    let pdf = document_to_pdf(&lower_source(TRANSFORMED).unwrap()).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains(" cm\n"));
}

#[test]
fn fixture_file_round_trips_to_pdf() {
    let src = include_str!("../../../examples/black_circle.rpx");
    let pdf = document_to_pdf(&lower_source(src).unwrap()).unwrap();
    assert!(pdf.windows(5).any(|w| w == b"%%EOF"));
}

#[test]
fn trivia_does_not_affect_lowering() {
    let src = "(page   a4\n\n  (circle 1 2 3))";
    let doc = lower_source(src).unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Circle(c) => {
            assert_eq!(c.x_mm, 1.0);
            assert_eq!(c.y_mm, 2.0);
            assert_eq!(c.radius_mm, 3.0);
        }
        _ => panic!("expected circle"),
    }
}

#[test]
fn lowers_rect_with_color() {
    let doc = lower_source("(page a4 (rect 10 20 30 40 blue))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Rect(r) => {
            assert_eq!(r.width_mm, 30.0);
            assert_eq!(r.height_mm, 40.0);
            assert_eq!(r.fill, Color::BLUE);
        }
        _ => panic!("expected rect"),
    }
}

#[test]
fn bad_rect_arity_fails() {
    assert!(lower_source("(page a4 (rect 1 2 3))").is_err());
}

#[test]
fn lowers_text_and_line() {
    let src = r#"(page a4
  (text 20 250 5 "Hello" blue)
  (text 30 200 6 40 20 "Boxed")
  (line 20 200 100 200 red 1))"#;
    let doc = lower_source(src).unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Text(t) => {
            assert_eq!(t.content, "Hello");
            assert_eq!(t.fill, Color::BLUE);
            assert_eq!(t.width_mm, None);
        }
        _ => panic!("expected text"),
    }
    match &doc.pages[0].shapes[1] {
        Shape::Text(t) => {
            assert_eq!(t.content, "Boxed");
            assert_eq!(t.width_mm, Some(40.0));
            assert_eq!(t.height_mm, Some(20.0));
        }
        _ => panic!("expected boxed text"),
    }
    match &doc.pages[0].shapes[2] {
        Shape::Line(l) => {
            assert_eq!(l.x2_mm, 100.0);
            assert_eq!(l.stroke, Color::RED);
            assert_eq!(l.width_mm, 1.0);
        }
        _ => panic!("expected line"),
    }
    let pdf = document_to_pdf(&doc).unwrap();
    assert!(String::from_utf8_lossy(&pdf).contains("(Hello) Tj"));
}

#[test]
fn lowers_ellipse() {
    let doc = lower_source("(page a4 (ellipse 105 148.5 60 30 green))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Ellipse(e) => {
            assert_eq!(e.rx_mm, 60.0);
            assert_eq!(e.ry_mm, 30.0);
            assert_eq!(e.fill, Color::GREEN);
        }
        _ => panic!("expected ellipse"),
    }
}

#[test]
fn lowers_ring_and_frame() {
    let src = "(page a4 (ring 1 2 3 0.5 red) (frame 0 0 10 20 1 blue))";
    let doc = lower_source(src).unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Ring(r) => {
            assert_eq!(r.radius_mm, 3.0);
            assert_eq!(r.stroke, Color::RED);
        }
        _ => panic!("expected ring"),
    }
    match &doc.pages[0].shapes[1] {
        Shape::Frame(f) => {
            assert_eq!(f.width_mm, 10.0);
            assert_eq!(f.stroke, Color::BLUE);
        }
        _ => panic!("expected frame"),
    }
    assert!(document_to_pdf(&doc).is_ok());
}

#[test]
fn lowers_polyline() {
    let doc = lower_source("(page a4 (polyline 0 0 10 10 20 0 blue 1.5))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Polyline(p) => {
            assert_eq!(p.points_mm.len(), 3);
            assert_eq!(p.stroke, Color::BLUE);
            assert_eq!(p.width_mm, 1.5);
        }
        _ => panic!("expected polyline"),
    }
}

#[test]
fn lowers_group_and_multipage() {
    let src = include_str!("../../../examples/two_pages.rpx");
    let doc = lower_source(src).unwrap();
    assert_eq!(doc.pages.len(), 2);
    match &doc.pages[1].shapes[1] {
        Shape::Group {
            transform,
            children,
        } => {
            assert_eq!(*transform, Affine::identity());
            assert_eq!(children.len(), 2);
        }
        _ => panic!("expected group"),
    }
    let pdf = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains("/Count 2"));
}

#[test]
fn lowers_letter_and_opacity() {
    let src = include_str!("../../../examples/letter_opacity.rpx");
    let doc = lower_source(src).unwrap();
    assert_eq!(doc.pages[0].paper, PaperSize::letter());
    match &doc.pages[0].shapes[1] {
        Shape::Opacity { alpha, children } => {
            assert_eq!(*alpha, 0.45);
            assert_eq!(children.len(), 1);
        }
        _ => panic!("expected opacity"),
    }
    let pdf = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains("/ExtGState"));
    assert!(text.contains("/GS45"));
}

#[test]
fn lowers_numeric_paper_size() {
    let doc = lower_source("(page 100 150 (circle 10 20 5))").unwrap();
    assert_eq!(
        doc.pages[0].paper,
        PaperSize {
            width_mm: 100.0,
            height_mm: 150.0
        }
    );
}

// --- defect ---

#[test]
fn empty_source_fails() {
    assert!(lower_source("").is_err());
    assert!(lower_source("   \n").is_err());
}

#[test]
fn parse_error_fails_fast() {
    let err = lower_source("(page a4").unwrap_err();
    assert!(err.message.contains("parse error"));
}

#[test]
fn unknown_shape_fails() {
    let err = lower_source("(page a4 (square 1 2 3))").unwrap_err();
    assert!(err.message.contains("unknown shape"));
}

#[test]
fn wrong_circle_arity_fails() {
    assert!(lower_source("(page a4 (circle 1 2))").is_err());
    assert!(lower_source("(page a4 (circle 1 2 3 4 5))").is_err());
}

#[test]
fn wrong_text_arity_fails() {
    assert!(lower_source("(page a4 (text 1 2 3))").is_err());
    assert!(lower_source(r#"(page a4 (text 1 2 3 4 5))"#).is_err()); // w/h without string
    assert!(lower_source(r#"(page a4 (text 1 2 3 10 20 "ok"))"#).is_ok());
}

#[test]
fn zero_radius_fails() {
    let err = lower_source("(page a4 (circle 1 2 0))").unwrap_err();
    assert!(err.message.contains("not drawable"));
}

#[test]
fn unknown_paper_fails() {
    let err = lower_source("(page a3 (circle 1 2 3))").unwrap_err();
    assert!(err.message.contains("unknown paper"));
}

#[test]
fn non_page_head_fails() {
    let err = lower_source("(sheet a4)").unwrap_err();
    assert!(err.message.contains("expected head `page`"));
}

#[test]
fn src_and_doc_skipped_while_pages_lower() {
    let src = r#"
(src (perform log "hi"))
(markup ignored)
(page a4 (circle 1 2 3))
"#;
    let doc = lower_source(src).unwrap();
    assert_eq!(doc.pages.len(), 1);
}

#[test]
fn src_only_fails_lower() {
    let err = lower_source("(src (perform log \"x\"))").unwrap_err();
    assert!(err.message.contains("no (page"));
}

#[test]
fn unknown_color_and_bad_rgb_fail() {
    assert!(lower_source("(page a4 (circle 0 0 1 puce))").is_err());
    assert!(lower_source("(page a4 (circle 0 0 1 (rgb 2 0 0)))").is_err());
    assert!(lower_source("(page a4 (circle 0 0 1 (rgb 0 0)))").is_err());
}

#[test]
fn transform_without_body_fails() {
    assert!(lower_source("(page a4 (translate 1 2))").is_err());
    assert!(lower_source("(page a4 (rotate 90))").is_err());
    assert!(lower_source("(page a4 (scale 2))").is_err());
}

#[test]
fn lowers_image_and_polygon_without_color() {
    let doc =
        lower_source(r#"(page a4 (image "pic.png" 10 20 30 40) (polygon 0 0 10 0 0 10))"#).unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Image(i) => {
            assert_eq!(i.path, "pic.png");
            assert_eq!(i.width_mm, 30.0);
        }
        _ => panic!("expected image"),
    }
    match &doc.pages[0].shapes[1] {
        Shape::Polygon(p) => assert_eq!(p.points_mm.len(), 3),
        _ => panic!("expected polygon"),
    }
}

#[test]
fn lowers_polyline_without_color() {
    let doc = lower_source("(page a4 (polyline 0 0 10 10 20 0))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Polyline(p) => {
            assert_eq!(p.points_mm.len(), 3);
            assert_eq!(p.width_mm, 0.5);
        }
        _ => panic!("expected polyline"),
    }
}

#[test]
fn lowers_uniform_scale_single_factor() {
    let doc = lower_source("(page a4 (scale 2 (circle 0 0 5)))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Group { transform, .. } => {
            assert_eq!(*transform, Affine::scale_uniform(2.0));
        }
        _ => panic!("expected group"),
    }
}

#[test]
fn lowers_backslash_chars_in_text() {
    let doc = lower_source(r#"(page a4 (text 1 2 3 "a\nb"))"#).unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Text(t) => assert_eq!(t.content, "a\\nb"),
        _ => panic!("expected text"),
    }
}

#[test]
fn negative_paper_size_fails() {
    let err = lower_source("(page -1 100 (circle 1 2 3))").unwrap_err();
    assert!(err.message.contains("positive"));
}

#[test]
fn page_missing_paper_fails() {
    assert!(lower_source("(page)").is_err());
    assert!(lower_source("(page 100)").is_err());
}

#[test]
fn opacity_out_of_range_fails() {
    assert!(lower_source("(page a4 (opacity 2 (circle 1 2 3)))").is_err());
    assert!(lower_source("(page a4 (opacity -0.1 (circle 1 2 3)))").is_err());
}

#[test]
fn empty_group_fails() {
    assert!(lower_source("(page a4 (group))").is_err());
}

#[test]
fn wrong_top_level_head_fails() {
    let err = lower_source("(bogus a4)").unwrap_err();
    assert!(err.message.contains("expected head"));
}

#[test]
fn lower_syntax_rejects_non_source_file() {
    let parse = parse_source("(page a4)").into_result().unwrap();
    let list = parse.children().next().unwrap();
    let err = lower_syntax(&list).unwrap_err();
    assert!(err.message.contains("SourceFile"));
}

#[test]
fn shape_token_instead_of_list_fails() {
    let err = lower_source("(page a4 1)").unwrap_err();
    assert!(err.message.contains("shape list"));
}

#[test]
fn zero_rect_dimensions_fail() {
    assert!(lower_source("(page a4 (rect 0 0 0 10))").is_err());
    assert!(lower_source("(page a4 (ellipse 0 0 0 1))").is_err());
}

#[test]
fn malformed_string_in_text_fails() {
    assert!(lower_source(r#"(page a4 (text 1 2 3 bad))"#).is_err());
}

#[test]
fn unknown_color_form_fails() {
    assert!(lower_source("(page a4 (circle 0 0 1 (hue 1 0 0)))").is_err());
}

// --- boundary/median: additional lowering arms ---

#[test]
fn lowers_text_len_six_with_trailing_color() {
    let doc = lower_source(r#"(page a4 (text 1 2 3 "hi" red))"#).unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Text(t) => {
            assert_eq!(t.content, "hi");
            assert_eq!(t.fill, Color::RED);
            assert_eq!(t.width_mm, None);
        }
        _ => panic!("expected text"),
    }
}

#[test]
fn lowers_text_boxed_with_color() {
    let doc = lower_source(r#"(page a4 (text 1 2 3 40 20 "hi" blue))"#).unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Text(t) => {
            assert_eq!(t.width_mm, Some(40.0));
            assert_eq!(t.height_mm, Some(20.0));
            assert_eq!(t.fill, Color::BLUE);
        }
        _ => panic!("expected text"),
    }
}

#[test]
fn lowers_line_with_color_only() {
    let doc = lower_source("(page a4 (line 0 0 10 10 green))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Line(l) => {
            assert_eq!(l.stroke, Color::GREEN);
            assert_eq!(l.width_mm, 0.5);
        }
        _ => panic!("expected line"),
    }
}

#[test]
fn lowers_polyline_with_rgb_color_node() {
    let doc = lower_source("(page a4 (polyline 0 0 10 10 20 0 (rgb 0.1 0.2 0.3) 2))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Polyline(p) => {
            assert_eq!(p.stroke, Color::new(0.1, 0.2, 0.3));
            assert_eq!(p.width_mm, 2.0);
        }
        _ => panic!("expected polyline"),
    }
}

#[test]
fn lowers_polygon_with_named_color() {
    let doc = lower_source("(page a4 (polygon 0 0 10 0 5 10 red))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Polygon(p) => assert_eq!(p.fill, Color::RED),
        _ => panic!("expected polygon"),
    }
}

#[test]
fn string_literals_keep_backslash_chars() {
    let doc = lower_source(r#"(page a4 (text 0 0 3 "a\nb\tc\\d"))"#).unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Text(t) => assert_eq!(t.content, "a\\nb\\tc\\\\d"),
        _ => panic!("expected text"),
    }
    let doc2 = lower_source(r#"(page a4 (text 0 0 3 "trail\\"))"#).unwrap();
    match &doc2.pages[0].shapes[0] {
        Shape::Text(t) => assert_eq!(t.content, "trail\\\\"),
        _ => panic!("expected text"),
    }
}

#[test]
fn lower_skips_doc_form_at_top_level() {
    let src = "(markup ignored)\n(page a4 (circle 1 2 3))";
    let doc = lower_source(src).unwrap();
    assert_eq!(doc.pages.len(), 1);
}

#[test]
fn lower_image_and_ring_errors() {
    assert!(lower_source(r#"(page a4 (image "p" 1 2 3))"#).is_err());
    assert!(lower_source("(page a4 (ring 1 2 3))").is_err());
}

#[test]
fn lower_color_token_and_node_errors() {
    assert!(lower_source("(page a4 (circle 0 0 1 123))").is_err());
    assert!(lower_source("(page a4 (circle 0 0 1 (rgb 0 0)))").is_err());
}

#[test]
fn lower_error_new_and_display() {
    let err = LowerError::new("msg");
    assert_eq!(err.message, "msg");
}

#[test]
fn lowers_opacity_nested_shapes() {
    let doc =
        lower_source("(page a4 (opacity 0.25 (group (circle 0 0 1) (rect 1 1 1 1))))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Opacity { alpha, children } => {
            assert_eq!(*alpha, 0.25);
            assert_eq!(children.len(), 1);
            match &children[0] {
                Shape::Group { children, .. } => assert_eq!(children.len(), 2),
                _ => panic!("expected group inside opacity"),
            }
        }
        _ => panic!("expected opacity"),
    }
}

// --- boundary: lowering error arms and helpers ---

#[test]
fn skips_non_list_top_level_forms() {
    let src = "123\n(page a4 (circle 1 2 3))";
    let doc = lower_source(src).unwrap();
    assert_eq!(doc.pages.len(), 1);
}

#[test]
fn ellipse_bad_arity_fails() {
    assert!(lower_source("(page a4 (ellipse 1 2 3))").is_err());
}

#[test]
fn ring_not_drawable_fails() {
    assert!(lower_source("(page a4 (ring 1 2 3 0))").is_err());
}

#[test]
fn frame_not_drawable_fails() {
    assert!(lower_source("(page a4 (frame 0 0 1 1 0))").is_err());
}

#[test]
fn text_zero_size_not_drawable() {
    assert!(lower_source(r#"(page a4 (text 1 2 0 "x"))"#).is_err());
}

#[test]
fn line_coincident_points_not_drawable() {
    assert!(lower_source("(page a4 (line 1 1 1 1))").is_err());
}

#[test]
fn polyline_too_few_coords_fails() {
    // Odd coordinate count after optional trailing color/width peel.
    assert!(lower_source("(page a4 (polyline 0 0 1))").is_err());
    // A single point is not enough (≥2 points required).
    assert!(lower_source("(page a4 (polyline 0 0))").is_err());
}

#[test]
fn polygon_too_few_points_fails() {
    assert!(lower_source("(page a4 (polygon 0 0 1 1))").is_err());
}

#[test]
fn image_zero_size_not_drawable() {
    assert!(lower_source(r#"(page a4 (image "p" 1 2 0 5))"#).is_err());
}

#[test]
fn malformed_string_literal_fails() {
    assert!(lower_source(r#"(page a4 (text 1 2 3 unquoted))"#).is_err());
}

#[test]
fn lower_color_bad_token_kind_fails() {
    assert!(lower_source("(page a4 (circle 0 0 1 123))").is_err());
}

#[test]
fn scale_two_factors_without_body_fails() {
    assert!(lower_source("(page a4 (scale 2 3))").is_err());
}

#[test]
fn scale_uniform_with_node_after_factor() {
    // items[2] is a shape list, not a second factor number.
    let doc = lower_source("(page a4 (scale 2 (circle 0 0 5) (circle 1 1 1)))").unwrap();
    assert_eq!(doc.pages[0].shapes.len(), 1);
}

#[test]
fn shape_list_expected_not_token() {
    let err = lower_source("(page a4 42)").unwrap_err();
    assert!(err.message.contains("shape list"));
}

#[test]
fn paper_ident_expected_not_number_atom() {
    assert!(lower_source("(page 100)").is_err());
}

#[test]
fn number_at_wrong_token_kind_fails() {
    assert!(lower_source("(page a4 (circle x 2 3))").is_err());
}

#[test]
fn trailing_backslash_in_string_is_literal() {
    let doc = lower_source(r#"(page a4 (text 0 0 3 "x\\"))"#).unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Text(t) => assert_eq!(t.content, "x\\\\"),
        _ => panic!("expected text"),
    }
}

#[test]
fn lowers_polyline_color_without_width() {
    let doc = lower_source("(page a4 (polyline 0 0 10 0 10 10 green))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Polyline(p) => {
            assert_eq!(p.stroke, Color::GREEN);
            assert_eq!(p.width_mm, 0.5);
        }
        _ => panic!("expected polyline"),
    }
}

#[test]
fn lowers_polygon_rgb_color_node() {
    let doc = lower_source("(page a4 (polygon 0 0 10 0 5 10 (rgb 0.2 0.4 0.6)))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Polygon(p) => assert_eq!(p.fill, Color::new(0.2, 0.4, 0.6)),
        _ => panic!("expected polygon"),
    }
}

#[test]
fn lowers_ring_without_color() {
    let doc = lower_source("(page a4 (ring 1 2 3 0.5))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Ring(r) => assert_eq!(r.stroke, Color::BLACK),
        _ => panic!("expected ring"),
    }
}

#[test]
fn lowers_frame_without_color() {
    let doc = lower_source("(page a4 (frame 0 0 10 20 1))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Frame(f) => assert_eq!(f.stroke, Color::BLACK),
        _ => panic!("expected frame"),
    }
}

#[test]
fn lower_error_display_matches_message() {
    let err = LowerError::new("boom");
    assert_eq!(format!("{err:?}"), "LowerError { message: \"boom\" }");
}

#[test]
fn lower_paper_letter_and_numeric_boundaries() {
    let letter = lower_source("(page letter (circle 0 0 1))").unwrap();
    assert_eq!(letter.pages.len(), 1);
    let tiny = lower_source("(page 1 1 (circle 0 0 0.5))").unwrap();
    assert_eq!(tiny.pages[0].paper.width_mm, 1.0);
    assert!(lower_source("(page 0 10 (circle 0 0 1))").is_err());
    assert!(lower_source("(page 10 0 (circle 0 0 1))").is_err());
    assert!(lower_source("(page -1 10 (circle 0 0 1))").is_err());
}

#[test]
fn lower_all_named_colors() {
    for color in ["black", "white", "red", "green", "blue"] {
        let src = format!("(page a4 (circle 0 0 1 {color}))");
        assert!(lower_source(&src).is_ok(), "{color}");
    }
}

#[test]
fn lower_transform_stacks_and_group() {
    let src = r#"(page a4
  (opacity 0.5
(rotate 45
  (scale 2 3
    (group
      (circle 0 0 1)
      (rect 0 0 1 1)
      (translate 1 2 (ellipse 0 0 1 2)))))))"#;
    let doc = lower_source(src).unwrap();
    assert!(!doc.pages[0].shapes.is_empty());
}

#[test]
fn lower_text_variants_and_escapes() {
    let plain = lower_source(r#"(page a4 (text 1 2 12 "hi"))"#).unwrap();
    assert!(matches!(plain.pages[0].shapes[0], Shape::Text(_)));
    let boxed = lower_source(r#"(page a4 (text 1 2 12 40 20 "box" red))"#).unwrap();
    assert!(matches!(boxed.pages[0].shapes[0], Shape::Text(_)));
    let esc = lower_source(r#"(page a4 (text 1 2 12 "a\\b\n\t"))"#).unwrap();
    match &esc.pages[0].shapes[0] {
        Shape::Text(t) => {
            assert_eq!(t.content, "a\\\\b\\n\\t");
        }
        _ => panic!("expected text"),
    }
}

#[test]
fn lower_line_polyline_polygon_partitions() {
    assert!(lower_source("(page a4 (line 0 0 10 10))").is_ok());
    assert!(lower_source("(page a4 (line 0 0 10 10 blue))").is_ok());
    assert!(lower_source("(page a4 (line 0 0 10 10 blue 2))").is_ok());
    assert!(lower_source("(page a4 (polyline 0 0 1 1 2 0))").is_ok());
    assert!(lower_source("(page a4 (polyline 0 0 1 1 2 0 red))").is_ok());
    assert!(lower_source("(page a4 (polyline 0 0 1 1 2 0 red 1.5))").is_ok());
    assert!(lower_source("(page a4 (polygon 0 0 1 0 0 1))").is_ok());
    assert!(lower_source("(page a4 (polygon 0 0 1 0 0 1 green))").is_ok());
}

#[test]
fn lower_shape_arity_and_type_errors() {
    assert!(lower_source("(page a4 (circle 1 2))").is_err());
    assert!(lower_source("(page a4 (rect 1 2 3))").is_err());
    assert!(lower_source("(page a4 (ellipse 1 2 3))").is_err());
    assert!(lower_source("(page a4 (ring 1 2 3))").is_err());
    assert!(lower_source("(page a4 (frame 1 2 3 4))").is_err());
    assert!(lower_source("(page a4 (image \"x\" 0 0 1))").is_err());
    assert!(lower_source("(page a4 (translate 1))").is_err());
    assert!(lower_source("(page a4 (rotate))").is_err());
    assert!(lower_source("(page a4 (scale 2))").is_err());
    assert!(lower_source("(page a4 (group))").is_err());
    assert!(lower_source("(page a4 (opacity 0.5))").is_err());
    assert!(lower_source("(page a4 (circle 0 0 0))").is_err());
    assert!(lower_source("(page a4 (rect 0 0 0 1))").is_err());
    assert!(lower_source("(page a4 (unknown 1))").is_err());
    assert!(lower_source("(page a4 (circle x 0 1))").is_err());
    assert!(lower_source(r#"(page a4 (text 1 2 3))"#).is_err());
}

#[test]
fn lower_multi_page_and_empty_page() {
    let doc = lower_source("(page a4)\n(page letter (circle 1 2 3))").unwrap();
    assert_eq!(doc.pages.len(), 2);
    assert!(doc.pages[0].shapes.is_empty());
    assert_eq!(doc.pages[1].shapes.len(), 1);
}

#[test]
fn lower_rgb_color_on_shapes() {
    let src = "(page a4 (circle 0 0 1 (rgb 0.1 0.2 0.3)) (rect 0 0 1 1 (rgb 1 0 0)))";
    let doc = lower_source(src).unwrap();
    assert_eq!(doc.pages[0].shapes.len(), 2);
}

#[test]
fn lower_paper_and_color_error_partitions() {
    assert!(lower_source("(page)").is_err());
    assert!(lower_source("(page 210)").is_err());
    assert!(lower_source("(page 0 10)").is_err());
    assert!(lower_source("(page foo)").is_err());
    assert!(lower_source("(page a4 (circle 0 0 1 puce))").is_err());
    assert!(lower_source("(page a4 (circle 0 0 1 (hsv 1 2 3)))").is_err());
    assert!(lower_source("(page a4 (circle 0 0 1 (rgb 1 2)))").is_err());
    assert!(lower_source("(page a4 (circle 0 0 1 (rgb 2 0 0)))").is_err());
    assert!(lower_source("(page a4 42)").is_err());
    assert!(lower_source("(markup hi)").is_err()); // markup is not lowered as page
}

#[test]
fn lower_line_width_and_image_path() {
    let line = lower_source("(page a4 (line 0 0 1 1 red 2.5))").unwrap();
    assert!(matches!(line.pages[0].shapes[0], Shape::Line(_)));
    let img = lower_source(r#"(page a4 (image "p.png" 0 0 10 20))"#).unwrap();
    assert!(matches!(img.pages[0].shapes[0], Shape::Image(_)));
    assert!(lower_source(r#"(page a4 (image "" 0 0 10 20))"#).is_err());
}

#[test]
fn string_unknown_backslash_sequences_kept() {
    let doc = lower_source(r#"(page a4 (text 0 0 3 "a\qb\\z"))"#).unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Text(t) => assert_eq!(t.content, "a\\qb\\\\z"),
        _ => panic!("expected text"),
    }
}

#[test]
fn opacity_nan_ident_and_out_of_range() {
    // Lexer has no NaN number literal; Ident `nan` fails number_at.
    assert!(lower_source("(page a4 (opacity nan (circle 0 0 1)))").is_err());
    assert!(lower_source("(page a4 (opacity 1.5 (circle 0 0 1)))").is_err());
}

#[test]
fn lower_skips_empty_list_and_number_headed_list() {
    let src = "()\n(1 2 3)\n(page a4 (circle 1 2 3))";
    let doc = lower_source(src).unwrap();
    assert_eq!(doc.pages.len(), 1);
}

#[test]
fn lower_shape_head_must_be_ident() {
    assert!(lower_source("(page a4 (1 2 3))").is_err());
    assert!(lower_source("(page a4 ((circle 0 0 1)))").is_err());
}

#[test]
fn string_at_wrong_kinds_and_image_path() {
    assert!(lower_source("(page a4 (image 1 2 3 4 5))").is_err());
    assert!(lower_source("(page a4 (image (p) 1 2 3 4))").is_err());
    assert!(lower_source(r#"(page a4 (text 1 2 3 40))"#).is_err());
}

#[test]
fn paper_node_and_polyline_polygon_not_drawable() {
    assert!(lower_source("(page (a4) (circle 0 0 1))").is_err());
    // Two coincident points → polyline not drawable.
    assert!(lower_source("(page a4 (polyline 0 0 0 0))").is_err());
    // Zero stroke width via trailing color+width peel.
    assert!(lower_source("(page a4 (polyline 0 0 1 0 red 0))").is_err());
}

#[test]
fn transform_number_and_body_errors() {
    assert!(lower_source("(page a4 (translate x 1 (circle 0 0 1)))").is_err());
    assert!(lower_source("(page a4 (rotate deg (circle 0 0 1)))").is_err());
    assert!(lower_source("(page a4 (scale x (circle 0 0 1)))").is_err());
    assert!(lower_source("(page a4 (scale 2 y (circle 0 0 1)))").is_err());
    assert!(lower_source("(page a4 (opacity 0.5 x))").is_err());
}

#[test]
fn color_rgb_channel_number_errors() {
    assert!(lower_source("(page a4 (circle 0 0 1 (rgb x 0 0)))").is_err());
    assert!(lower_source("(page a4 (circle 0 0 1 (rgb 0 y 0)))").is_err());
    assert!(lower_source("(page a4 (circle 0 0 1 (rgb 0 0 z)))").is_err());
}

#[test]
fn color_unknown_form_and_number_as_node_errors() {
    assert!(lower_source("(page a4 (circle 0 0 1 (hsl 0.1 0.2 0.3)))").is_err());
    // Nested list where a Number slot is expected.
    assert!(lower_source("(page a4 (circle (1) 0 1))").is_err());
    assert!(lower_source("(page a4 (circle 0 0 1 ()))").is_err());
    assert!(lower_source("(page a4 (circle 1 2))").is_err());
}

#[test]
fn polyline_trailing_number_without_color_does_not_peel() {
    // end >= 3 with a trailing number that is not (color, width) — keep all coords.
    let doc = lower_source("(page a4 (polyline 0 0 10 0 10 10 5 5))").unwrap();
    match &doc.pages[0].shapes[0] {
        Shape::Polyline(p) => assert_eq!(p.points_mm.len(), 4),
        _ => panic!("expected polyline"),
    }
}

#[test]
fn polyline_odd_coords_after_color_peel() {
    // After peeling trailing color, three numbers remain → odd.
    assert!(lower_source("(page a4 (polyline 0 0 1 red))").is_err());
    // Polygon with odd coords after color.
    assert!(lower_source("(page a4 (polygon 0 0 1 0 0 red))").is_err());
}

#[test]
fn shape_numeric_slot_type_errors() {
    assert!(lower_source("(page a4 (circle 1 x 3))").is_err());
    assert!(lower_source("(page a4 (circle 1 2 x))").is_err());
    assert!(lower_source("(page a4 (rect x 0 1 1))").is_err());
    assert!(lower_source("(page a4 (rect 0 y 1 1))").is_err());
    assert!(lower_source("(page a4 (rect 0 0 w 1))").is_err());
    assert!(lower_source("(page a4 (rect 0 0 1 h))").is_err());
    assert!(lower_source("(page a4 (rect 0 0 1 1 puce))").is_err());
    assert!(lower_source("(page a4 (ellipse x 0 1 1))").is_err());
    assert!(lower_source("(page a4 (ellipse 0 y 1 1))").is_err());
    assert!(lower_source("(page a4 (ellipse 0 0 rx 1))").is_err());
    assert!(lower_source("(page a4 (ellipse 0 0 1 ry))").is_err());
    assert!(lower_source("(page a4 (ellipse 0 0 1 1 puce))").is_err());
    assert!(lower_source("(page a4 (ring x 0 1 1))").is_err());
    assert!(lower_source("(page a4 (ring 0 y 1 1))").is_err());
    assert!(lower_source("(page a4 (ring 0 0 r 1))").is_err());
    assert!(lower_source("(page a4 (ring 0 0 1 w))").is_err());
    assert!(lower_source("(page a4 (ring 0 0 1 1 puce))").is_err());
    assert!(lower_source("(page a4 (frame x 0 1 1 1))").is_err());
    assert!(lower_source("(page a4 (frame 0 y 1 1 1))").is_err());
    assert!(lower_source("(page a4 (frame 0 0 w 1 1))").is_err());
    assert!(lower_source("(page a4 (frame 0 0 1 h 1))").is_err());
    assert!(lower_source("(page a4 (frame 0 0 1 1 sw))").is_err());
    assert!(lower_source("(page a4 (frame 0 0 1 1 1 puce))").is_err());
    assert!(lower_source(r#"(page a4 (text x 0 3 "a"))"#).is_err());
    assert!(lower_source(r#"(page a4 (text 0 y 3 "a"))"#).is_err());
    assert!(lower_source(r#"(page a4 (text 0 0 s "a"))"#).is_err());
    assert!(lower_source(r#"(page a4 (text 0 0 3 w 5 "a"))"#).is_err());
    assert!(lower_source(r#"(page a4 (text 0 0 3 4 h "a"))"#).is_err());
    assert!(lower_source(r#"(page a4 (text 0 0 3 "a" puce))"#).is_err());
    assert!(lower_source(r#"(page a4 (text 0 0 3 4 5 "a" puce))"#).is_err());
    assert!(lower_source("(page a4 (line x 0 1 1))").is_err());
    assert!(lower_source("(page a4 (line 0 y 1 1))").is_err());
    assert!(lower_source("(page a4 (line 0 0 x 1))").is_err());
    assert!(lower_source("(page a4 (line 0 0 1 y))").is_err());
    assert!(lower_source("(page a4 (line 0 0 1 1))").is_ok());
    assert!(lower_source("(page a4 (line 0 0))").is_err());
    assert!(lower_source("(page a4 (line 0 0 1 1 red w))").is_err());
    assert!(lower_source(r#"(page a4 (image "p" x 0 1 1))"#).is_err());
    assert!(lower_source(r#"(page a4 (image "p" 0 y 1 1))"#).is_err());
    assert!(lower_source(r#"(page a4 (image "p" 0 0 w 1))"#).is_err());
    assert!(lower_source(r#"(page a4 (image "p" 0 0 1 h))"#).is_err());
    assert!(lower_source("(page a4 (translate 1 y (circle 0 0 1)))").is_err());
}

#[test]
fn numeric_paper_slot_type_errors() {
    assert!(lower_source("(page x 100 (circle 0 0 1))").is_err());
    // First is Number so numeric branch; second must be Number too.
    assert!(lower_source("(page 100 y (circle 0 0 1))").is_err());
}

#[test]
fn line_color_and_text_boxed_color_errors() {
    assert!(lower_source("(page a4 (line 0 0 1 1 puce))").is_err());
    assert!(lower_source(r#"(page a4 (text 0 0 3 4 5 "a" puce))"#).is_err());
    assert!(lower_source(r#"(page a4 (text 0 0 3 w 5 "a" red))"#).is_err());
    assert!(lower_source("(page a4 (polyline 0 0 1 0 puce 1))").is_err());
    assert!(lower_source("(page a4 (polygon 0 0 1 0 0 1 puce))").is_err());
    assert!(lower_source("(page a4 (polyline 0 0 x 1))").is_err());
    assert!(lower_source("(page a4 (polygon 0 0 1 0 0 y))").is_err());
    // rgb node recognized as color but channels invalid → lower_color Err while peeling.
    assert!(lower_source("(page a4 (polyline 0 0 1 0 (rgb 2 0 0) 1))").is_err());
    assert!(lower_source("(page a4 (polyline 0 0 1 0 (rgb 2 0 0)))").is_err());
    assert!(lower_source("(page a4 (polygon 0 0 1 0 0 1 (rgb 2 0 0)))").is_err());
    assert!(lower_source("(page a4 (polyline 0 0 1 (2)))").is_err());
    assert!(lower_source("(page a4 (polygon 0 0 1 0 (0) 1))").is_err());
    assert!(lower_source("(page a4 (scale 2 foo (circle 0 0 1)))").is_err());
    assert!(lower_source("(page a4 (scale 2 (circle 0 0 1) (rect 0 0 1 1)))").is_ok());
}

#[test]
fn transform_tail_shape_errors() {
    assert!(lower_source("(page a4 (translate 1 2 bad))").is_err());
    assert!(lower_source("(page a4 (group bad))").is_err());
    assert!(lower_source("(page a4 (rotate 10 bad))").is_err());
    assert!(lower_source("(page a4 (scale 2 3))").is_err());
    // Paper slot is a string token (not Ident).
    assert!(lower_source(r#"(page "a4" (circle 0 0 1))"#).is_err());
}

#[test]
fn number_as_node_in_shape_slots() {
    assert!(lower_source("(page a4 (circle (1) 2 3))").is_err());
    assert!(lower_source("(page a4 (rect 0 0 (1) 1))").is_err());
    assert!(lower_source(r#"(page a4 (text 0 0 3))"#).is_err()); // missing string
}

#[test]
fn coverage_hooks_number_at_and_parse() {
    assert!(coverage_number_at_missing());
    assert_eq!(coverage_parse_num_text("+2.5"), 2.5);
    assert_eq!(coverage_parse_num_text("-1"), -1.0);
    assert_eq!(coverage_parse_num_text("abc"), 0.0);
}

#[test]
fn type_val_and_top_level_effects_are_skipped() {
    let src = r#"
(type title str)
(val title "Hello")
(perform log "x")
(handle log (perform log "y"))
(page a4 (circle 1 2 3))
"#;
    let doc = lower_source(src).unwrap();
    assert_eq!(doc.pages.len(), 1);
}
