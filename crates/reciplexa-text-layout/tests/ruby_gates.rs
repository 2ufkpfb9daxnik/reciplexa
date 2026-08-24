//! Step 8 item 1: font-backed simple ruby vs stub estimates.

use reciplexa_std::japanese::{char_em_width, Ruby, RUBY_ANNOTATION_SCALE};
use reciplexa_text_layout::ruby::RUBY_PARENT_GAP_EM;
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
    let expected_ann = origin_y + size * (1.0 + RUBY_PARENT_GAP_EM);
    assert!(
        (laid.annotation.y_mm - expected_ann).abs() < 1e-9,
        "annotation baseline must sit above the parent em-square with a gap, got {}",
        laid.annotation.y_mm
    );
    assert_eq!(laid.annotation.run.size_mm, size * RUBY_ANNOTATION_SCALE);
    let parent_top = laid.base.y_mm + size;
    assert!(
        laid.annotation.y_mm + 1e-9 >= parent_top,
        "ruby must not overlap the parent body"
    );
    assert!(laid.annotation.y_mm - parent_top >= size * 0.2);
}

#[test]
fn simple_ruby_measure_uses_base_width_when_annotation_is_wider() {
    let f = font();
    let ruby = Ruby::simple("東京", "とうきょう");
    let laid = layout_simple_ruby(&f, &ruby, 0.0, 50.0, 10.0).expect("layout");
    assert!(laid.annotation.width_mm > laid.base.width_mm);
    assert!((laid.advance_mm - laid.base.width_mm).abs() < 1e-9);
    assert!((laid.ink_width_mm - laid.annotation.width_mm).abs() < 1e-9);
    assert!((laid.base.x_mm - 0.0).abs() < 1e-9);
    let base_center = laid.base.x_mm + laid.base.width_mm * 0.5;
    let ann_center = laid.annotation.x_mm + laid.annotation.width_mm * 0.5;
    assert!((base_center - ann_center).abs() < 1e-6);
    assert!(
        laid.advance_mm + 1e-9 < laid.ink_width_mm,
        "measure must be narrower than ink when annotation overhangs"
    );
}

#[test]
fn simple_ruby_centers_annotation_on_base_when_narrower() {
    let f = font();
    let ruby = Ruby::simple("あ", "かん");
    let laid = layout_simple_ruby(&f, &ruby, 5.0, 50.0, 10.0).expect("layout");
    assert!((laid.advance_mm - laid.base.width_mm).abs() < 1e-9);
    assert!((laid.base.x_mm - 5.0).abs() < 1e-9);
    let base_center = laid.base.x_mm + laid.base.width_mm * 0.5;
    let ann_center = laid.annotation.x_mm + laid.annotation.width_mm * 0.5;
    assert!((base_center - ann_center).abs() < 1e-6);
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

fn ann_segment_center_by_bytes(
    glyphs: &[reciplexa_text_layout::PositionedGlyph],
    start: usize,
    end: usize,
) -> f64 {
    let mut min_x = f64::INFINITY;
    let mut max_x = 0.0f64;
    let mut found = false;
    for g in glyphs {
        if g.cluster_start >= start && g.cluster_start < end {
            min_x = min_x.min(g.x_mm);
            max_x = max_x.max(g.x_mm + g.advance_mm);
            found = true;
        }
    }
    assert!(found, "no glyphs in byte range [{start}, {end})");
    (min_x + max_x) * 0.5
}

#[test]
fn jukugo_ruby_distributes_per_base() {
    let f = font();
    let ruby = Ruby::jukugo("東京", "とうきょう");
    let laid = layout_simple_ruby(&f, &ruby, 0.0, 50.0, 10.0).expect("jukugo layout");
    assert!((laid.advance_mm - laid.base.width_mm).abs() < 1e-9);
    let east = laid
        .base
        .run
        .glyphs
        .iter()
        .find(|g| g.ch == '東')
        .expect("base 東");
    let kyo = laid
        .base
        .run
        .glyphs
        .iter()
        .find(|g| g.ch == '京')
        .expect("base 京");
    let east_center = east.x_mm + east.advance_mm * 0.5;
    let kyo_center = kyo.x_mm + kyo.advance_mm * 0.5;
    let segs = reciplexa_text_layout::distribute_jukugo_annotation("東京", "とうきょう");
    let b0 = 0;
    let b1 = segs[0].len();
    let b2 = b1 + segs[1].len();
    let tou_center = ann_segment_center_by_bytes(&laid.annotation.run.glyphs, b0, b1);
    let kyou_center = ann_segment_center_by_bytes(&laid.annotation.run.glyphs, b1, b2);
    assert!(
        (tou_center - east_center).abs() < 0.5,
        "とう should sit above 東: tou={tou_center} east={east_center}"
    );
    assert!(
        (kyou_center - kyo_center).abs() < 0.5,
        "きょう should center on 京: ann={kyou_center} base={kyo_center}"
    );
    assert!(
        tou_center < kyou_center,
        "とう segment should be left of きょう segment"
    );
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
