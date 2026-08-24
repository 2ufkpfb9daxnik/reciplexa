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
