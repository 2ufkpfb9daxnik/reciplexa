//! Step 12 gates: Japanese Document Profile v2.

use reciplexa_text_layout::{layout_vertical_run, vert_glyph_paint_offset_em, LoadedFont};

#[test]
fn step12_slice1_reciprocal_punctuation_offset_em() {
    let (_, dy_stop) = vert_glyph_paint_offset_em('。');
    let (_, dy_comma) = vert_glyph_paint_offset_em('、');
    assert!((dy_stop - 0.25).abs() < 1e-9);
    assert!((dy_comma - 0.25).abs() < 1e-9);
    assert_eq!(vert_glyph_paint_offset_em('A'), (0.0, 0.0));
}

#[test]
fn step12_slice1_vertical_advances_unchanged_by_paint_offset() {
    let f = LoadedFont::fixture();
    let size = 10.0;
    let a = layout_vertical_run(&f, "字。", 0.0, 50.0, size).expect("layout");
    let b = layout_vertical_run(&f, "字字", 0.0, 50.0, size).expect("layout");
    assert_eq!(a.run.glyphs[0].advance_mm, b.run.glyphs[0].advance_mm);
    assert_eq!(a.run.glyphs[1].advance_mm, b.run.glyphs[1].advance_mm);
    assert!((a.height_mm - b.height_mm).abs() < 1e-9);
}
