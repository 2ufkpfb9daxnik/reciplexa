//! Step 11 gates: typed protocol + deterministic font fallback.

use reciplexa_text_layout::{
    build_fallback_only_font_bytes, clusters_cover_input, infer_script, FontFallbackChain,
    LoadedFont, Script, ShapingAttributes,
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
