//! Step 11 gates: typed protocol + font fallback + bidi + host product wiring.

use reciplexa_scene::{Color, Shape, Text};
use reciplexa_std::japanese::ParagraphSceneLayout;
use reciplexa_text_layout::{
    analyze_paragraph, build_fallback_only_font_bytes, build_liga_fixture_font_bytes,
    clusters_cover_input, glyph_shapes_export_coords, infer_script,
    layout_wrapped_paragraph_product, layout_wrapped_paragraph_product_lines,
    positioned_line_glyph_coords, positioned_line_to_glyph_shapes, productize_shape_text_emit,
    require_emit_matches_shaped_run, shape_run, shape_run_complex, visual_glyph_indices, Direction,
    FontFallbackChain, LoadedFont, PositionedLine, Script, ShapedGlyph, ShapingAttributes,
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

#[test]
fn step11_productize_visual_gid_order_uses_bidi_helper() {
    let font = LoadedFont::fixture();
    let text = "ABC";
    let shaped = productize_shape_text_emit(
        Shape::Text(Text {
            x_mm: 0.0,
            y_mm: 0.0,
            size_mm: 10.0,
            width_mm: None,
            height_mm: None,
            content: text.into(),
            fill: Color::BLACK,
        }),
        &font,
        &font,
    )
    .expect("productize");
    let g = match shaped {
        Shape::GlyphRun(g) => g,
        other => panic!("expected GlyphRun, got {other:?}"),
    };
    let run =
        shape_run_complex(&font, text, &ShapingAttributes::horizontal_ltr("en")).expect("shape");
    let order = visual_glyph_indices(text, &run.glyphs, Direction::Ltr);
    let expected_gids: Vec<u16> = order.iter().map(|&i| run.glyphs[i].gid).collect();
    assert_eq!(g.gids, expected_gids);
    assert_eq!(g.gids.len(), g.advances_mm.len());
    assert_eq!(g.font_digest, font.id.digest);
}

#[test]
fn step11_from_segment_bidi_visual_x_order() {
    let text = "日اب本";
    let font = LoadedFont::fixture();
    let glyphs = glyphs_for_mixed_ja_rtl(text, font.id.clone());
    let seg = reciplexa_text_layout::LineSegment {
        text: text.into(),
        start_byte: 0,
        end_byte: text.len(),
        natural_width_em: glyphs.len() as f64,
        hang_em: 0.0,
        trim_em: 0.0,
        reason: reciplexa_text_layout::BreakReason::End,
        glyphs,
    };
    let line = PositionedLine::from_segment(font.id.clone(), &seg, 0.0, 0.0, 10.0, "ja");
    let visual_chars: String = line.run.glyphs.iter().map(|g| g.ch).collect();
    assert_eq!(visual_chars, "日با本");
}

#[test]
fn step11_productize_emit_digest_mismatch_fails() {
    let layout = LoadedFont::fixture();
    let emit = LoadedFont::from_bytes(build_liga_fixture_font_bytes(), "ReciplexaLigaFixture")
        .expect("liga");
    let err = productize_shape_text_emit(
        Shape::Text(Text {
            x_mm: 0.0,
            y_mm: 0.0,
            size_mm: 10.0,
            width_mm: None,
            height_mm: None,
            content: "A".into(),
            fill: Color::BLACK,
        }),
        &layout,
        &emit,
    )
    .expect_err("digest mismatch");
    assert!(matches!(
        err,
        reciplexa_text_layout::LayoutError::SubstitutionRequiresRelayout { .. }
    ));
}

#[test]
fn step11_ja_paragraph_preview_export_glyph_coords_match() {
    let font = LoadedFont::fixture();
    let layout = ParagraphSceneLayout {
        base_x_mm: 12.0,
        start_y_mm: 200.0,
        pitch_mm: 6.0,
        size_mm: 4.0,
        fill: Color::BLACK,
    };
    let text = "あいうえおかきくけこ";
    let lines =
        layout_wrapped_paragraph_product_lines(&font, text, 8.0, 2.0, &layout).expect("lines");
    assert!(lines.len() > 1);
    let shapes = layout_wrapped_paragraph_product(&font, text, 8.0, 2.0, &layout).expect("shapes");
    let from_lines: Vec<_> = lines
        .iter()
        .flat_map(positioned_line_glyph_coords)
        .collect();
    let from_shapes = glyph_shapes_export_coords(&shapes);
    assert_eq!(from_lines.len(), from_shapes.len());
    for (line, shape) in from_lines.iter().zip(from_shapes.iter()) {
        assert_eq!(line.0, shape.0);
        assert_eq!(line.1, shape.1);
        assert_eq!(line.2, shape.2);
        assert!((line.3 - shape.3).abs() < 1e-9);
        assert_eq!(line.4, shape.4);
    }
}

#[test]
fn step11_fallback_per_glyph_digest_reaches_export() {
    let primary = LoadedFont::fixture();
    let fallback =
        LoadedFont::from_bytes(build_fallback_only_font_bytes(), "ReciplexaFallbackOnly")
            .expect("parse fallback font");
    let chain = FontFallbackChain::new(primary.clone()).with_fallback(fallback);
    let text = "日☺本";
    let run = chain
        .shape(text, &ShapingAttributes::horizontal_ltr("ja"))
        .expect("shape with fallback");
    let line = PositionedLine::from_shaped_run(primary.id.clone(), &run, 5.0, 10.0, 8.0, "ja");
    let ja = line
        .run
        .glyphs
        .iter()
        .find(|g| g.ch == '日')
        .expect("ja glyph");
    let smile = line
        .run
        .glyphs
        .iter()
        .find(|g| g.ch == '☺')
        .expect("fallback glyph");
    assert_ne!(ja.font_digest, smile.font_digest);
    let shapes = positioned_line_to_glyph_shapes(&line, Color::BLACK);
    let export = glyph_shapes_export_coords(&shapes);
    assert_eq!(export.len(), line.run.glyphs.len());
    let smile_export = export
        .iter()
        .find(|(_, _, _, _, d)| *d == smile.font_digest)
        .expect("fallback digest in export");
    assert_eq!(smile_export.2, smile.gid);
}
