//! Step 13 gates: MATH glyph assembly (slice 1) and MATH kern (slice 2).

use reciplexa_identity::document::StableNodeId;
use reciplexa_std::math::{EstimateStyle, MathAtom, MathClass};
use reciplexa_text_layout::{
    layout_math_atom, LoadedFont, FIXTURE_ITALIC_CORRECTION_F, FIXTURE_MATH_KERN_BOTTOM_RIGHT,
    FIXTURE_MATH_KERN_TOP_RIGHT, FIXTURE_TALL_PAREN_VARIANT_ADVANCE,
};

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

#[test]
fn step13_slice2_superscript_applies_italic_and_top_kern() {
    let f = font();
    let upem = f64::from(f.units_per_em());
    let italic = f64::from(FIXTURE_ITALIC_CORRECTION_F) / upem;
    let kern = f64::from(FIXTURE_MATH_KERN_TOP_RIGHT) / upem;
    let laid = layout_math_atom(
        &f,
        &MathAtom::superscript(
            StableNodeId::new(1),
            MathAtom::symbol(StableNodeId::new(2), "f", MathClass::Ordinary),
            MathAtom::symbol(StableNodeId::new(3), "2", MathClass::Ordinary),
        ),
        EstimateStyle::Display,
    )
    .unwrap();
    let base = laid.glyphs.iter().find(|g| g.glyph.ch == 'f').expect("f");
    let sup = laid.glyphs.iter().find(|g| g.glyph.ch == '2').expect("2");
    let expected = base.glyph.advance_em * base.scale + italic + kern;
    assert!(
        (sup.x_em - expected).abs() < 1e-9,
        "superscript x {} want {expected} (advance+italic+top kern)",
        sup.x_em
    );
}

#[test]
fn step13_slice2_subscript_uses_bottom_kern_not_italic() {
    let f = font();
    let upem = f64::from(f.units_per_em());
    let italic = f64::from(FIXTURE_ITALIC_CORRECTION_F) / upem;
    let kern = f64::from(FIXTURE_MATH_KERN_BOTTOM_RIGHT) / upem;
    let laid = layout_math_atom(
        &f,
        &MathAtom::subscript(
            StableNodeId::new(1),
            MathAtom::symbol(StableNodeId::new(2), "f", MathClass::Ordinary),
            MathAtom::symbol(StableNodeId::new(3), "i", MathClass::Ordinary),
        ),
        EstimateStyle::Display,
    )
    .unwrap();
    let base = laid.glyphs.iter().find(|g| g.glyph.ch == 'f').expect("f");
    let sub = laid.glyphs.iter().find(|g| g.glyph.ch == 'i').expect("i");
    let expected = base.glyph.advance_em * base.scale + kern;
    assert!(
        (sub.x_em - expected).abs() < 1e-9,
        "subscript x {} want {expected} (advance+bottom kern, no italic)",
        sub.x_em
    );
    assert!(
        (sub.x_em - (base.glyph.advance_em + italic)).abs() > 1e-6,
        "subscript must not sit at italic-shifted advance"
    );
}

#[test]
fn step13_slice2_adjacent_nuclei_add_italic() {
    let f = font();
    let upem = f64::from(f.units_per_em());
    let italic = f64::from(FIXTURE_ITALIC_CORRECTION_F) / upem;
    let laid = layout_math_atom(
        &f,
        &MathAtom::row(
            StableNodeId::new(1),
            vec![
                MathAtom::symbol(StableNodeId::new(2), "f", MathClass::Ordinary),
                MathAtom::symbol(StableNodeId::new(3), "x", MathClass::Ordinary),
            ],
        ),
        EstimateStyle::Display,
    )
    .unwrap();
    let base = laid.glyphs.iter().find(|g| g.glyph.ch == 'f').expect("f");
    let next = laid.glyphs.iter().find(|g| g.glyph.ch == 'x').expect("x");
    let expected = base.glyph.advance_em * base.scale + italic;
    assert!(
        (next.x_em - expected).abs() < 1e-9,
        "adjacent x {} want {expected}",
        next.x_em
    );
}

#[test]
fn step13_slice2_bigop_limits_use_italic_and_kern() {
    let f = font();
    let upem = f64::from(f.units_per_em());
    let italic = f64::from(FIXTURE_ITALIC_CORRECTION_F) / upem;
    let top = f64::from(FIXTURE_MATH_KERN_TOP_RIGHT) / upem;
    let bot = f64::from(FIXTURE_MATH_KERN_BOTTOM_RIGHT) / upem;
    let laid = layout_math_atom(
        &f,
        &MathAtom::big_op(
            StableNodeId::new(1),
            "f",
            Some(MathAtom::symbol(
                StableNodeId::new(2),
                "i",
                MathClass::Ordinary,
            )),
            Some(MathAtom::symbol(
                StableNodeId::new(3),
                "n",
                MathClass::Ordinary,
            )),
            None,
        ),
        EstimateStyle::Display,
    )
    .unwrap();
    let op = laid.glyphs.iter().find(|g| g.glyph.ch == 'f').expect("f");
    let upper = laid.glyphs.iter().find(|g| g.glyph.ch == 'n').expect("n");
    let lower = laid.glyphs.iter().find(|g| g.glyph.ch == 'i').expect("i");
    let op_w = op.glyph.advance_em * op.scale;
    let u_w = upper.glyph.advance_em * upper.scale;
    let l_w = lower.glyph.advance_em * lower.scale;
    let expect_u = (op_w - u_w).max(0.0) * 0.5 + italic * 0.5 + top;
    let expect_l = (op_w - l_w).max(0.0) * 0.5 - italic * 0.5 + bot;
    assert!(
        (upper.x_em - expect_u).abs() < 1e-9,
        "upper x {} want {expect_u}",
        upper.x_em
    );
    assert!(
        (lower.x_em - expect_l).abs() < 1e-9,
        "lower x {} want {expect_l}",
        lower.x_em
    );
}
