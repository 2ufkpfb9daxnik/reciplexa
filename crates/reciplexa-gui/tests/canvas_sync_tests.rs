use reciplexa_gui::canvas_sync::{authoring_layers_align, nudge_authoring_layers};

#[test]
fn nudge_text_on_authoring_page_updates_xy_via_translate() {
    let src = r#"(page a4 (text 30 260 8 "Reciplexa" black))"#;
    let out = nudge_authoring_layers(src, src, 0, &[0], 5.0, -3.0).unwrap();
    assert!(
        out.contains(r#"(translate 5 -3 (text 30 260 8 "Reciplexa" black))"#),
        "{out}"
    );
}

#[test]
fn japanese_page_text_nudge_keeps_sibling() {
    // Interim CST golden for CJK nudge; package path lives in examples/japanese_page.rpx.
    let src = r#"(page a4
  (text 25 270 8 "レポート草稿" black)
  (text 25 250 4 "本文。ページ上の text として置く現行の書き方です。" black)
  (rotate 18 (circle 140 140 42 blue))
  (opacity 0.35
    (rect 40 60 90 50 red)))"#;
    assert!(
        authoring_layers_align(src, src, 0).unwrap(),
        "interim CJK page authoring layers should self-align"
    );
    let out = nudge_authoring_layers(src, src, 0, &[0], 3.0, -2.0).unwrap();
    assert!(
        out.contains(r#"(translate 3 -2 (text 25 270 8 "レポート草稿" black))"#),
        "first CJK text should nudge: {out}"
    );
    assert!(
        out.contains(
            r#"(text 25 250 4 "本文。ページ上の text として置く現行の書き方です。" black)"#
        ),
        "sibling japanese body text must stay put: {out}"
    );
}

#[test]
fn markup_expanded_paint_align_false_nudge_soft() {
    let authoring = include_str!("../../../examples/markup_ja.rpx");
    let expanded = reciplexa_macro::expand_source(authoring).expect("markup expands");
    // Expanded buffer is package-shaped; authoring is markup-only — not CST-editable.
    assert!(
        !authoring_layers_align(authoring, &expanded, 0).unwrap(),
        "package-emitted markup layers must not claim authoring editability"
    );
    let err = nudge_authoring_layers(authoring, &expanded, 0, &[0], 2.0, -1.0).unwrap_err();
    assert!(
        err.message.contains("read-only")
            || err.message.contains("skipped")
            || err.message.contains("package"),
        "{}",
        err.message
    );
}

#[test]
fn color_byte_expand_still_editable() {
    let authoring = r#"(// Latin text, stroked line, rgb / color-byte fills.)
(page a4
  (text 30 260 8 "Reciplexa" black)
  (line 30 250 180 250 (color-byte 200 40 40) 1)
  (translate 105 120
    (circle 0 0 25 (color-byte 30 90 180))))"#;
    let expanded = reciplexa_macro::expand_source(authoring).expect("expand");
    assert_ne!(authoring, expanded);
    assert!(authoring_layers_align(authoring, &expanded, 0).unwrap());
    let out = nudge_authoring_layers(authoring, &expanded, 0, &[0], 1.0, 0.0).unwrap();
    assert!(out.contains("(translate 1 0 (text"), "{out}");
}

#[test]
fn authoring_parse_error_with_divergent_expanded_is_soft_false() {
    let authoring = "(page";
    let expanded = "(page a4 (text 1 2 3 \"x\" black))";
    assert!(!authoring_layers_align(authoring, expanded, 0).unwrap());
    let err = nudge_authoring_layers(authoring, expanded, 0, &[0], 1.0, 0.0).unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn package_black_circle_nudge_updates_authoring() {
    let authoring = include_str!("../../../examples/pkg_black_circle.rpx");
    let expanded = reciplexa_macro::expand_source(authoring).expect("expand");
    assert!(
        authoring_layers_align(authoring, &expanded, 0).unwrap(),
        "package twin should align for CST edits"
    );
    let out = nudge_authoring_layers(authoring, &expanded, 0, &[0], 2.0, -1.0).unwrap();
    assert!(
        out.contains("(circle 107 147.5 40)"),
        "expected package circle xy rewrite: {out}"
    );
}

#[test]
fn black_circle_gui_golden_is_package_nudgeable() {
    let authoring = include_str!("../../../examples/black_circle.rpx");
    assert!(reciplexa_lower::is_package_shaped_authoring(authoring));
    let out = nudge_authoring_layers(authoring, authoring, 0, &[0], 2.0, -1.0).unwrap();
    assert!(
        out.contains("(circle 107 147.5 40)"),
        "GUI golden should nudge package circle: {out}"
    );
}

#[test]
fn interim_black_circle_nudge_still_works() {
    let authoring = include_str!("../../../examples/interim_black_circle.rpx");
    let out = nudge_authoring_layers(authoring, authoring, 0, &[0], 2.0, -1.0).unwrap();
    assert!(
        out.contains("(translate 2 -1 (circle 105 148.5 40))")
            || out.contains("(circle 107 147.5 40)"),
        "{out}"
    );
}

#[test]
fn markup_ja_example_still_soft_refuses_authoring_edits() {
    let authoring = include_str!("../../../examples/markup_ja.rpx");
    let expanded = reciplexa_macro::expand_source(authoring).expect("markup expands");
    assert!(
        !reciplexa_lower::is_package_shaped_authoring(authoring),
        "markup authoring must not look package-shaped"
    );
    assert!(
        !authoring_layers_align(authoring, &expanded, 0).unwrap(),
        "markup_ja must not claim authoring editability"
    );
    let err = nudge_authoring_layers(authoring, &expanded, 0, &[0], 2.0, -1.0).unwrap_err();
    assert!(
        err.message.contains("read-only")
            || err.message.contains("skipped")
            || err.message.contains("markup"),
        "{}",
        err.message
    );
}

#[test]
fn multipage_package_nudge_second_page() {
    let authoring = r#"(import graphics/shapes only circle fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main
  (list
    (page a4 (fill (circle 10 20 5) black))
    (page a4 (fill (circle 30 40 6) black))))
"#;
    let out = nudge_authoring_layers(authoring, authoring, 1, &[0], 1.0, 1.0).unwrap();
    assert!(out.contains("(circle 10 20 5)"), "{out}");
    assert!(out.contains("(circle 31 41 6)"), "{out}");
}
