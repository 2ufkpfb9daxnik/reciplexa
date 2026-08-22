//! Step 11 gates: typed protocol + font fallback + bidi.

use reciplexa_text_layout::{
    analyze_paragraph, build_fallback_only_font_bytes, build_liga_fixture_font_bytes,
    clusters_cover_input, infer_script, require_emit_matches_shaped_run, shape_run,
    shape_run_complex, visual_glyph_indices, Direction, FontFallbackChain, LoadedFont,
    PositionedLine, Script, ShapedGlyph, ShapingAttributes,
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
    let fallback =
        LoadedFont::from_bytes(build_fallback_only_font_bytes(), "ReciplexaFallbackOnly")
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
    let line =
        PositionedLine::from_shaped_run_with_attrs(font.id.clone(), &run, 0.0, 0.0, 10.0, &attrs);
    assert_eq!(line.run.direction, Direction::Rtl);
    assert_eq!(line.run.glyphs.first().map(|g| g.ch), Some('C'));
}

#[test]
fn step11_complex_shape_fi_ligature_cluster() {
    let font = LoadedFont::from_bytes(build_liga_fixture_font_bytes(), "ReciplexaLigaFixture")
        .expect("liga");
    let text = "fi";
    let mut attrs = ShapingAttributes::horizontal_ltr("en");
    attrs.script = Script::Latin;
    let run = shape_run_complex(&font, text, &attrs).expect("liga shape");
    assert_eq!(run.glyphs.len(), 1);
    assert_eq!(run.glyphs[0].cluster_start, 0);
    assert_eq!(run.glyphs[0].cluster_end, text.len());
    assert!(clusters_cover_input(&run));
}

#[test]
fn step11_emit_digest_mismatch_requires_relayout() {
    let layout = LoadedFont::fixture();
    let emit = LoadedFont::from_bytes(build_liga_fixture_font_bytes(), "ReciplexaLigaFixture")
        .expect("liga");
    let run = shape_run(&layout, "A").expect("shape");
    let err = require_emit_matches_shaped_run(&run, &emit).expect_err("digest mismatch");
    assert!(matches!(
        err,
        reciplexa_text_layout::LayoutError::SubstitutionRequiresRelayout { .. }
    ));
}
