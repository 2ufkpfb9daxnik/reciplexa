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
fn markup_expanded_paint_align_false_nudge_soft() {
    let authoring = include_str!("../../../examples/markup_ja.rpx");
    let expanded = reciplexa_macro::expand_source(authoring).expect("markup expands");
    // Expanded buffer is drawable (has page layers); authoring is markup-only.
    assert!(
        !authoring_layers_align(authoring, &expanded, 0).unwrap(),
        "synthetic markup layers must not claim authoring editability"
    );
    let err = nudge_authoring_layers(authoring, &expanded, 0, &[0], 2.0, -1.0).unwrap_err();
    assert!(
        err.message.contains("read-only") || err.message.contains("skipped"),
        "{}",
        err.message
    );
}

#[test]
fn color_byte_expand_still_editable() {
    let authoring = include_str!("../../../examples/text_line.rpx");
    let expanded = reciplexa_macro::expand_source(authoring).expect("expand");
    assert_ne!(authoring, expanded);
    assert!(authoring_layers_align(authoring, &expanded, 0).unwrap());
    let out = nudge_authoring_layers(authoring, &expanded, 0, &[0], 1.0, 0.0).unwrap();
    assert!(out.contains("(translate 1 0 (text"), "{out}");
}
