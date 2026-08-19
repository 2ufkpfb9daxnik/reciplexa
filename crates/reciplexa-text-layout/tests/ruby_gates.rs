//! Step 8 item 1: font-backed simple ruby vs stub estimates.

use reciplexa_std::japanese::{char_em_width, Ruby, RUBY_ANNOTATION_SCALE, RUBY_HEIGHT_BUMP_EM};
use reciplexa_text_layout::{
    layout_simple_ruby, positioned_ruby_to_shapes, LayoutError, LoadedFont,
};

fn font() -> LoadedFont {
    LoadedFont::fixture()
}

#[test]
fn simple_ruby_widths_differ_from_stub() {
    let f = font();
    let ruby = Ruby::simple("あ", "かん");
    let stub = ruby.estimate_box();
    let laid = layout_simple_ruby(&f, &ruby, 20.0, 100.0, 10.0).expect("layout");
    let font_base_em: f64 = ruby
        .base
        .chars()
        .map(|c| f.hor_advance_em(c).unwrap())
        .sum();
    let stub_base: f64 = ruby.base.chars().map(char_em_width).sum();
    assert!((stub.base_width - stub_base).abs() < 1e-12);
    assert!(
        (font_base_em - stub_base).abs() > 1e-6,
        "fixture hiragana advance must differ from char_em_width"
    );
    assert!((laid.base.width_mm - font_base_em * 10.0).abs() < 1e-9);
    let font_ann_em: f64 = ruby
        .annotation
        .chars()
        .map(|c| f.hor_advance_em(c).unwrap())
        .sum();
    assert!((laid.annotation.width_mm - font_ann_em * 10.0 * RUBY_ANNOTATION_SCALE).abs() < 1e-9);
}

#[test]
fn simple_ruby_annotation_sits_above_base() {
    let f = font();
    let ruby = Ruby::simple("東", "とう");
    let origin_y = 80.0;
    let size = 12.0;
    let laid = layout_simple_ruby(&f, &ruby, 10.0, origin_y, size).expect("layout");
    assert!((laid.base.y_mm - origin_y).abs() < 1e-9);
    assert!((laid.annotation.y_mm - (origin_y + size * RUBY_HEIGHT_BUMP_EM)).abs() < 1e-9);
    assert_eq!(laid.annotation.run.size_mm, size * RUBY_ANNOTATION_SCALE);
    assert!(laid.annotation.y_mm > laid.base.y_mm);
}

#[test]
fn simple_ruby_centers_when_annotation_is_wider() {
    let f = font();
    let ruby = Ruby::simple("東京", "とうきょう");
    let laid = layout_simple_ruby(&f, &ruby, 0.0, 50.0, 10.0).expect("layout");
    assert!(laid.annotation.width_mm > laid.base.width_mm);
    assert!((laid.advance_mm - laid.annotation.width_mm).abs() < 1e-9);
    let base_inset = laid.base.x_mm;
    let ann_inset = laid.annotation.x_mm;
    assert!(base_inset > ann_inset);
    assert!((base_inset * 2.0 + laid.base.width_mm - laid.advance_mm).abs() < 1e-6);
}

#[test]
fn simple_ruby_emits_glyph_runs() {
    let f = font();
    let ruby = Ruby::simple("京", "きょう");
    let laid = layout_simple_ruby(&f, &ruby, 20.0, 40.0, 8.0).expect("layout");
    let shapes = positioned_ruby_to_shapes(&laid, reciplexa_scene::Color::BLACK);
    let n_base = ruby.base.chars().count();
    let n_ann = ruby.annotation.chars().count();
    assert_eq!(shapes.len(), n_base + n_ann);
    assert!(shapes
        .iter()
        .all(|s| matches!(s, reciplexa_scene::Shape::GlyphRun(_))));
}

#[test]
fn jukugo_ruby_is_refused() {
    let f = font();
    let ruby = Ruby::jukugo("東京", "とうきょう");
    let err = layout_simple_ruby(&f, &ruby, 0.0, 0.0, 10.0).expect_err("jukugo");
    match err {
        LayoutError::Engine { detail } => assert!(detail.contains("jukugo"), "{detail}"),
        other => panic!("expected Engine, got {other:?}"),
    }
}

#[test]
fn empty_ruby_is_refused() {
    let f = font();
    let ruby = Ruby::simple("", "かん");
    assert!(matches!(
        layout_simple_ruby(&f, &ruby, 0.0, 0.0, 10.0),
        Err(LayoutError::Engine { .. })
    ));
}
