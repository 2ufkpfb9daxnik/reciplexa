//! Step 12 gates: Japanese Document Profile v2.

use reciplexa_text_layout::{layout_vertical_run, vert_glyph_paint_offset_em, LoadedFont};

#[test]
fn step12_slice1_reciprocal_punctuation_offset_em() {
    let (dx_stop, dy_stop) = vert_glyph_paint_offset_em('。');
    let (dx_comma, dy_comma) = vert_glyph_paint_offset_em('、');
    assert!((dx_stop - 0.5).abs() < 1e-9);
    assert!((dy_stop - 0.5).abs() < 1e-9);
    assert_eq!(dx_comma, dx_stop);
    assert_eq!(dy_comma, dy_stop);
    assert_eq!(vert_glyph_paint_offset_em('A'), (-1.0, -1.0));
}

#[test]
fn step12_slice1_prolonged_sound_offset_em() {
    let (dx, dy) = vert_glyph_paint_offset_em('ー');
    assert!((dx + 1.0).abs() < 1e-9);
    assert!((dy + 1.0).abs() < 1e-9);
}

#[test]
fn step12_slice1_prolonged_sound_rotates() {
    use reciplexa_text_layout::vert_glyph_rotation_deg;
    assert_eq!(vert_glyph_rotation_deg('ー', false), -90.0);
    assert_eq!(vert_glyph_rotation_deg('ー', true), 0.0);
    assert_eq!(vert_glyph_rotation_deg('。', false), 0.0);
    assert_eq!(vert_glyph_rotation_deg('、', false), 0.0);
    assert_eq!(vert_glyph_rotation_deg('漢', false), 0.0);
}

#[test]
fn step12_slice1_vertical_punctuation_rotates_and_offsets() {
    let f = LoadedFont::fixture();
    let size = 10.0;
    let laid = layout_vertical_run(&f, "字。", 0.0, 50.0, size).expect("layout");
    assert_eq!(laid.rotation_deg[1], 0.0);
    let kanji = layout_vertical_run(&f, "字字", 0.0, 50.0, size).expect("layout");
    let stop = &laid.run.glyphs[1];
    let second = &kanji.run.glyphs[1];
    let (dx, dy) = vert_glyph_paint_offset_em('。');
    assert!((stop.x_mm - second.x_mm - dx * size).abs() < 1e-9);
    assert!((stop.y_mm - second.y_mm - dy * size).abs() < 1e-9);
}

#[test]
fn step12_slice2_ruby_measure_is_base_width() {
    use reciplexa_std::japanese::Ruby;
    use reciplexa_text_layout::layout_simple_ruby;

    let f = LoadedFont::fixture();
    let ruby = Ruby::simple("京", "きょう");
    let laid = layout_simple_ruby(&f, &ruby, 12.0, 40.0, 10.0).expect("layout");
    assert!((laid.advance_mm - laid.base.width_mm).abs() < 1e-9);
    assert!((laid.base.x_mm - 12.0).abs() < 1e-9);
    if laid.annotation.width_mm > laid.base.width_mm {
        assert!((laid.ink_width_mm - laid.annotation.width_mm).abs() < 1e-9);
        assert!(laid.ink_width_mm > laid.advance_mm);
    }
}

#[test]
fn step12_slice2_ruby_annotation_centered_on_parent() {
    use reciplexa_std::japanese::Ruby;
    use reciplexa_text_layout::layout_simple_ruby;

    let f = LoadedFont::fixture();
    let ruby = Ruby::simple("東京", "とうきょう");
    let laid = layout_simple_ruby(&f, &ruby, 0.0, 50.0, 10.0).expect("layout");
    let base_center = laid.base.x_mm + laid.base.width_mm * 0.5;
    let ann_center = laid.annotation.x_mm + laid.annotation.width_mm * 0.5;
    assert!((base_center - ann_center).abs() < 1e-6);
}

#[test]
fn step12_slice3_jukugo_distribution_segments() {
    use reciplexa_text_layout::distribute_jukugo_annotation;

    assert_eq!(
        distribute_jukugo_annotation("東京", "とうきょう"),
        vec!["とう", "きょう"]
    );
    assert_eq!(
        distribute_jukugo_annotation("日本語", "にほんご"),
        vec!["に", "ほ", "んご"]
    );
}

#[test]
fn step12_slice3_jukugo_per_base_centers() {
    use reciplexa_std::japanese::Ruby;
    use reciplexa_text_layout::layout_ruby;

    let f = LoadedFont::fixture();
    let ruby = Ruby::jukugo("東京", "とうきょう");
    let laid = layout_ruby(&f, &ruby, 10.0, 40.0, 10.0).expect("layout");
    let east = laid.base.run.glyphs.iter().find(|g| g.ch == '東').unwrap();
    let east_c = east.x_mm + east.advance_mm * 0.5;
    let segs = reciplexa_text_layout::distribute_jukugo_annotation("東京", "とうきょう");
    let end0 = segs[0].len();
    let gs: Vec<_> = laid
        .annotation
        .run
        .glyphs
        .iter()
        .filter(|g| g.cluster_start < end0)
        .collect();
    let min_x = gs.iter().map(|g| g.x_mm).fold(f64::INFINITY, f64::min);
    let max_x = gs.iter().map(|g| g.x_mm + g.advance_mm).fold(0.0, f64::max);
    let tou_c = (min_x + max_x) * 0.5;
    assert!((east_c - tou_c).abs() < 0.6);
}

#[test]
fn step12_slice4_vertical_ruby_side_annotation() {
    use reciplexa_std::japanese::Ruby;
    use reciplexa_text_layout::layout_vertical_ruby;

    let f = LoadedFont::fixture();
    let ruby = Ruby::simple("京", "きょう");
    let laid = layout_vertical_ruby(&f, &ruby, 20.0, 80.0, 12.0).expect("layout");
    assert!((laid.advance_mm - laid.base.height_mm).abs() < 1e-9);
    let base = laid.base.run.glyphs.first().expect("base glyph");
    let ann = laid.annotation.run.glyphs.first().expect("ann glyph");
    assert!(ann.x_mm > base.x_mm + 12.0);
    let shapes = reciplexa_text_layout::positioned_vertical_ruby_to_shapes(
        &laid,
        reciplexa_scene::Color::BLACK,
    );
    assert!(shapes
        .iter()
        .any(|s| matches!(s, reciplexa_scene::Shape::GlyphRun(_))));
}
