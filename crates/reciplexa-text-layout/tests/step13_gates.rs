//! Step 13 gates: MATH glyph assembly (no visual scale).

use reciplexa_identity::document::StableNodeId;
use reciplexa_std::math::{EstimateStyle, MathAtom, MathClass};
use reciplexa_text_layout::{layout_math_atom, LoadedFont, FIXTURE_TALL_PAREN_VARIANT_ADVANCE};

fn font() -> LoadedFont {
    LoadedFont::fixture()
}

#[test]
fn step13_slice1_variant_used_when_tall_enough() {
    let f = font();
    let delim = MathAtom::delimiter_with_stretch(
        StableNodeId::new(1),
        "(",
        ")",
        MathAtom::symbol(StableNodeId::new(2), "x", MathClass::Ordinary),
        1.5,
    );
    let laid = layout_math_atom(&f, &delim, EstimateStyle::Display).unwrap();
    let left: Vec<_> = laid.glyphs.iter().filter(|g| g.glyph.ch == '(').collect();
    assert_eq!(left.len(), 1, "variant should suffice at stretch 1.5");
    assert!((left[0].scale - 1.0).abs() < 1e-9, "no visual scale");
    let cmap = f.glyph_id('(').unwrap();
    assert_ne!(left[0].glyph.gid, cmap);
}

#[test]
fn step13_slice1_assembly_when_taller_than_variants() {
    let f = font();
    let delim = MathAtom::delimiter_with_stretch(
        StableNodeId::new(1),
        "(",
        ")",
        MathAtom::symbol(StableNodeId::new(2), "x", MathClass::Ordinary),
        5.0,
    );
    let laid = layout_math_atom(&f, &delim, EstimateStyle::Display).unwrap();
    let left: Vec<_> = laid.glyphs.iter().filter(|g| g.glyph.ch == '(').collect();
    assert!(
        left.len() > 1,
        "assembly must emit multiple parts, got {}",
        left.len()
    );
    assert!(
        left.iter().all(|g| (g.scale - 1.0).abs() < 1e-9),
        "assembly parts must not visual-scale"
    );
    let tall_h = f64::from(FIXTURE_TALL_PAREN_VARIANT_ADVANCE) / 1000.0;
    assert!(
        laid.metrics.total_height() + 1e-9 >= tall_h,
        "assembled height {} should meet or exceed tall variant {tall_h}",
        laid.metrics.total_height()
    );
    let gids: Vec<u16> = left.iter().map(|g| g.glyph.gid).collect();
    assert!(
        gids.windows(2).any(|w| w[0] != w[1]),
        "assembly should mix end and extender GIDs, got {gids:?}"
    );
}
