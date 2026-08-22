//! Step 11 gates: typed protocol + font fallback + bidi.

use reciplexa_text_layout::{
    analyze_paragraph, build_fallback_only_font_bytes, clusters_cover_input, infer_script,
    shape_run, visual_glyph_indices, Direction, FontFallbackChain, LoadedFont, PositionedLine,
    Script, ShapedGlyph, ShapingAttributes,
};

#[test]
fn step11_infer_script_covers_ja_and_latin() {
    assert_eq!(infer_script("hello"), Script::Latin);
    assert_eq!(infer_script("あいう"), Script::Hiragana);
    assert_eq!(infer_script("漢字"), Script::Han);
    assert_eq!(infer_script("カタ"), Script::Katakana);
}

#[test]
fn step11_fallback_preserves_clusters_and_order() {
    let primary = LoadedFont::fixture();
    let fallback = LoadedFont::from_bytes(
        build_fallback_only_font_bytes(),
        "ReciplexaFallbackOnly",
    )
    .expect("parse fallback font");
    let chain = FontFallbackChain::new(primary).with_fallback(fallback);
    let text = "日☺本";
    let run = chain
        .shape(text, &ShapingAttributes::horizontal_ltr("ja"))
        .expect("shape with fallback");
    assert!(clusters_cover_input(&run));
    assert_eq!(run.text, text);
    assert_eq!(run.glyphs.len(), 3);
    assert_eq!(
        run.glyphs.iter().map(|g| g.ch).collect::<Vec<_>>(),
        vec!['日', '☺', '本']
    );
}

fn glyphs_for_mixed_ja_rtl(text: &str, font_id: reciplexa_text_layout::FontId) -> Vec<ShapedGlyph> {
    let mut glyphs = Vec::new();
    let mut byte = 0usize;
    for ch in text.chars() {
        let len = ch.len_utf8();
        glyphs.push(ShapedGlyph {
            font_id: font_id.clone(),
            gid: 1,
            ch,
            cluster_start: byte,
            cluster_end: byte + len,
            advance_em: 1.0,
            italic_correction_em: 0.0,
        });
        byte += len;
    }
    glyphs
}

#[test]
fn step11_bidi_mixed_ja_rtl_visual_order() {
    let text = "日اب本";
    let segments = analyze_paragraph(text, Direction::Ltr);
    assert!(segments.iter().any(|s| s.direction == Direction::Rtl));
    let font = LoadedFont::fixture();
    let glyphs = glyphs_for_mixed_ja_rtl(text, font.id);
    let order = visual_glyph_indices(text, &glyphs, Direction::Ltr);
    let chars: String = order.iter().map(|&i| glyphs[i].ch).collect();
    assert_eq!(chars, "日با本");
}

#[test]
fn step11_positioned_line_propagates_rtl_direction() {
    let font = LoadedFont::fixture();
    let run = shape_run(&font, "ABC").expect("shape");
    let attrs = ShapingAttributes::horizontal_rtl("ar");
    let line = PositionedLine::from_shaped_run_with_attrs(
        font.id.clone(),
        &run,
        0.0,
        0.0,
        10.0,
        &attrs,
    );
    assert_eq!(line.run.direction, Direction::Rtl);
    assert_eq!(line.run.glyphs.first().map(|g| g.ch), Some('C'));
}
