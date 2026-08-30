//! Step 13 gates: MATH glyph assembly (slice 1), MATH kern (slice 2),
//! remaining layout constants from the font (slice 3), alignment / eqno (slice 4),
//! combined assembly + kern + display math (slice 6).

use reciplexa_identity::document::StableNodeId;
use reciplexa_std::math::{EstimateStyle, MathAccentKind, MathAtom, MathClass};
use reciplexa_text_layout::{
    layout_math_atom, LoadedFont, EQUATION_NUMBER_MARGIN_EM, FIXTURE_AXIS_HEIGHT,
    FIXTURE_DELIMITED_SUB_FORMULA_MIN_HEIGHT, FIXTURE_FRACTION_NUMERATOR_DISPLAY_STYLE_SHIFT_UP,
    FIXTURE_FRACTION_NUMERATOR_SHIFT_UP, FIXTURE_ITALIC_CORRECTION_F,
    FIXTURE_MATH_KERN_BOTTOM_RIGHT, FIXTURE_MATH_KERN_TOP_RIGHT, FIXTURE_OVERBAR_RULE_THICKNESS,
    FIXTURE_RADICAL_DISPLAY_STYLE_VERTICAL_GAP, FIXTURE_RADICAL_KERN_BEFORE_DEGREE,
    FIXTURE_SPACE_AFTER_SCRIPT, FIXTURE_STACK_DISPLAY_STYLE_GAP_MIN,
    FIXTURE_TALL_PAREN_VARIANT_ADVANCE,
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

fn almost_eq(got: f64, want: f64, what: &str) {
    assert!((got - want).abs() < 1e-9, "{what}: got {got} want {want}");
}

#[test]
fn step13_slice3_glyph_ink_from_glyf_bbox() {
    let f = font();
    let upem = f64::from(f.units_per_em());
    let laid = layout_math_atom(
        &f,
        &MathAtom::symbol(StableNodeId::new(1), "x", MathClass::Ordinary),
        EstimateStyle::Display,
    )
    .unwrap();
    almost_eq(laid.metrics.height, 700.0 / upem, "glyf y_max height");
    almost_eq(
        laid.metrics.depth,
        0.0,
        "glyf y_min=0 depth (not a 0.2 em heuristic)",
    );
}

#[test]
fn step13_slice3_radical_uses_display_style_gap() {
    let f = font();
    let upem = f64::from(f.units_per_em());
    let x = layout_math_atom(
        &f,
        &MathAtom::symbol(StableNodeId::new(1), "x", MathClass::Ordinary),
        EstimateStyle::Display,
    )
    .unwrap();
    let laid = layout_math_atom(
        &f,
        &MathAtom::radical(
            StableNodeId::new(2),
            MathAtom::symbol(StableNodeId::new(3), "x", MathClass::Ordinary),
        ),
        EstimateStyle::Display,
    )
    .unwrap();
    let gap = f64::from(FIXTURE_RADICAL_DISPLAY_STYLE_VERTICAL_GAP) / upem;
    almost_eq(
        laid.rules[0].y0_em,
        x.metrics.height + gap,
        "radical rule y",
    );
}

#[test]
fn step13_slice3_overbar_rule_uses_math_thickness() {
    let f = font();
    let upem = f64::from(f.units_per_em());
    let laid = layout_math_atom(
        &f,
        &MathAtom::accent(
            StableNodeId::new(1),
            MathAccentKind::Overline,
            MathAtom::symbol(StableNodeId::new(2), "x", MathClass::Ordinary),
        ),
        EstimateStyle::Display,
    )
    .unwrap();
    almost_eq(
        laid.rules[0].thickness_em,
        f64::from(FIXTURE_OVERBAR_RULE_THICKNESS) / upem,
        "overbar thickness",
    );
}

#[test]
fn step13_slice3_space_after_script_from_font() {
    let f = font();
    let upem = f64::from(f.units_per_em());
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
    let sup = laid.glyphs.iter().find(|g| g.glyph.ch == '2').expect("2");
    let extent = sup.x_em + sup.glyph.advance_em * sup.scale;
    let space = f64::from(FIXTURE_SPACE_AFTER_SCRIPT) / upem;
    almost_eq(laid.metrics.width, extent + space, "script box width");
}

#[test]
fn step13_slice3_display_fraction_shift_differs_from_text() {
    let f = font();
    let upem = f64::from(f.units_per_em());
    let axis = f64::from(FIXTURE_AXIS_HEIGHT) / upem;
    let frac = MathAtom::fraction(
        StableNodeId::new(1),
        MathAtom::symbol(StableNodeId::new(2), "a", MathClass::Ordinary),
        MathAtom::symbol(StableNodeId::new(3), "b", MathClass::Ordinary),
    );
    let display = layout_math_atom(&f, &frac, EstimateStyle::Display).unwrap();
    let text = layout_math_atom(&f, &frac, EstimateStyle::Text).unwrap();
    let a_d = display
        .glyphs
        .iter()
        .find(|g| g.glyph.ch == 'a')
        .expect("a");
    let a_t = text.glyphs.iter().find(|g| g.glyph.ch == 'a').expect("a");
    almost_eq(
        a_d.y_em,
        axis + f64::from(FIXTURE_FRACTION_NUMERATOR_DISPLAY_STYLE_SHIFT_UP) / upem,
        "display numerator y",
    );
    almost_eq(
        a_t.y_em,
        axis + f64::from(FIXTURE_FRACTION_NUMERATOR_SHIFT_UP) / upem,
        "text numerator y",
    );
    assert!(
        (a_d.y_em - a_t.y_em).abs() > 1e-6,
        "display and text numerator shifts must differ"
    );
}

#[test]
fn step13_slice3_stack_uses_display_style_gap() {
    let f = font();
    let upem = f64::from(f.units_per_em());
    let x = layout_math_atom(
        &f,
        &MathAtom::symbol(StableNodeId::new(1), "x", MathClass::Ordinary),
        EstimateStyle::Display,
    )
    .unwrap();
    let laid = layout_math_atom(
        &f,
        &MathAtom::atop(
            StableNodeId::new(2),
            MathAtom::symbol(StableNodeId::new(3), "a", MathClass::Ordinary),
            MathAtom::symbol(StableNodeId::new(4), "b", MathClass::Ordinary),
        ),
        EstimateStyle::Display,
    )
    .unwrap();
    let a = laid.glyphs.iter().find(|g| g.glyph.ch == 'a').expect("a");
    let b = laid.glyphs.iter().find(|g| g.glyph.ch == 'b').expect("b");
    let gap = f64::from(FIXTURE_STACK_DISPLAY_STYLE_GAP_MIN) / upem;
    almost_eq(
        a.y_em - b.y_em,
        x.metrics.height + x.metrics.depth + gap,
        "stack baseline gap",
    );
}

#[test]
fn step13_slice3_delimited_min_height_from_font() {
    let f = font();
    let upem = f64::from(f.units_per_em());
    let min_h = f64::from(FIXTURE_DELIMITED_SUB_FORMULA_MIN_HEIGHT) / upem;
    let laid = layout_math_atom(
        &f,
        &MathAtom::delimiter_with_stretch(
            StableNodeId::new(1),
            "(",
            ")",
            MathAtom::symbol(StableNodeId::new(2), "x", MathClass::Ordinary),
            1.0,
        ),
        EstimateStyle::Display,
    )
    .unwrap();
    let left: Vec<_> = laid.glyphs.iter().filter(|g| g.glyph.ch == '(').collect();
    assert_eq!(
        left.len(),
        1,
        "min height 1.2 em should pick the tall variant"
    );
    let (ink_h, ink_d) = f.glyph_ink_em(left[0].glyph.gid).unwrap();
    assert!(
        ink_h + ink_d + 1e-9 >= min_h,
        "chosen variant ink {} must meet delimitedSubFormulaMinHeight {min_h}",
        ink_h + ink_d
    );
}

#[test]
fn step13_slice3_radical_degree_kern_before_from_font() {
    let f = font();
    let upem = f64::from(f.units_per_em());
    let laid = layout_math_atom(
        &f,
        &MathAtom::radical_indexed(
            StableNodeId::new(1),
            MathAtom::symbol(StableNodeId::new(2), "n", MathClass::Ordinary),
            MathAtom::symbol(StableNodeId::new(3), "x", MathClass::Ordinary),
        ),
        EstimateStyle::Display,
    )
    .unwrap();
    let n = laid.glyphs.iter().find(|g| g.glyph.ch == 'n').expect("n");
    almost_eq(
        n.x_em,
        f64::from(FIXTURE_RADICAL_KERN_BEFORE_DEGREE) / upem,
        "radicalKernBeforeDegree",
    );
}

fn ord(id: u64, glyph: &str) -> MathAtom {
    MathAtom::symbol(StableNodeId::new(id), glyph, MathClass::Ordinary)
}

fn rel(id: u64, glyph: &str) -> MathAtom {
    MathAtom::symbol(StableNodeId::new(id), glyph, MathClass::Relation)
}

#[test]
fn step13_slice4_aligned_columns_share_relation_x() {
    let f = font();
    let laid = layout_math_atom(
        &f,
        &MathAtom::aligned(
            StableNodeId::new(1),
            vec![
                vec![ord(2, "a"), rel(3, "="), ord(4, "b")],
                vec![ord(5, "ccc"), rel(6, "="), ord(7, "d")],
            ],
        ),
        EstimateStyle::Display,
    )
    .unwrap();
    let eqs: Vec<_> = laid.glyphs.iter().filter(|g| g.glyph.ch == '=').collect();
    assert_eq!(eqs.len(), 2, "two relation cells");
    almost_eq(eqs[0].x_em, eqs[1].x_em, "aligned = column");
    let a = laid.glyphs.iter().find(|g| g.glyph.ch == 'a').expect("a");
    assert!(
        eqs[0].x_em > a.x_em + a.glyph.advance_em * a.scale + 1e-9,
        "relation sits in column 1, not flush to a"
    );
}

#[test]
fn step13_slice4_equation_numbers_right_align() {
    let f = font();
    let laid = layout_math_atom(
        &f,
        &MathAtom::aligned_numbered(
            StableNodeId::new(1),
            vec![
                vec![ord(2, "a"), rel(3, "="), ord(4, "b")],
                vec![ord(5, "ccc"), rel(6, "="), ord(7, "d")],
            ],
            vec![Some("(1)".into()), Some("(2)".into())],
        ),
        EstimateStyle::Display,
    )
    .unwrap();
    let one = laid.glyphs.iter().find(|g| g.glyph.ch == '1').expect("1");
    let two = laid.glyphs.iter().find(|g| g.glyph.ch == '2').expect("2");
    let close: Vec<_> = laid.glyphs.iter().filter(|g| g.glyph.ch == ')').collect();
    assert_eq!(close.len(), 2, "two closing parens on equation numbers");
    let r1 = close[0].x_em + close[0].glyph.advance_em * close[0].scale;
    let r2 = close[1].x_em + close[1].glyph.advance_em * close[1].scale;
    almost_eq(r1, r2, "eqno right edges");
    almost_eq(r1, EQUATION_NUMBER_MARGIN_EM, "eqno at margin");
    assert!(
        (one.y_em - two.y_em).abs() > 1e-6,
        "numbers on different rows"
    );
}

#[test]
fn step13_slice6_combined_assembly_kern_and_display() {
    let f = font();
    let upem = f64::from(f.units_per_em());
    let italic = f64::from(FIXTURE_ITALIC_CORRECTION_F) / upem;
    let kern = f64::from(FIXTURE_MATH_KERN_TOP_RIGHT) / upem;
    let cmap = f.glyph_id('(').unwrap();
    let laid = layout_math_atom(
        &f,
        &MathAtom::aligned_numbered(
            StableNodeId::new(1),
            vec![
                vec![
                    MathAtom::delimiter_with_stretch(
                        StableNodeId::new(2),
                        "(",
                        ")",
                        ord(3, "x"),
                        5.0,
                    ),
                    rel(4, "="),
                    ord(5, "a"),
                ],
                vec![
                    MathAtom::superscript(StableNodeId::new(6), ord(7, "f"), ord(8, "2")),
                    rel(9, "="),
                    ord(10, "y"),
                ],
            ],
            vec![Some("(1)".into()), Some("(2)".into())],
        ),
        EstimateStyle::Display,
    )
    .unwrap();

    let assembled: Vec<_> = laid
        .glyphs
        .iter()
        .filter(|g| g.glyph.ch == '(' && g.glyph.gid != cmap)
        .collect();
    assert!(
        assembled.len() > 1,
        "assembly must emit multiple left parts, got {}",
        assembled.len()
    );
    assert!(
        assembled.iter().all(|g| (g.scale - 1.0).abs() < 1e-9),
        "assembly parts must not visual-scale"
    );
    let gids: Vec<u16> = assembled.iter().map(|g| g.glyph.gid).collect();
    assert!(
        gids.windows(2).any(|w| w[0] != w[1]),
        "assembly should mix end and extender GIDs, got {gids:?}"
    );

    let base = laid.glyphs.iter().find(|g| g.glyph.ch == 'f').expect("f");
    let sup = laid.glyphs.iter().find(|g| g.glyph.ch == '2').expect("2");
    let expected = base.x_em + base.glyph.advance_em * base.scale + italic + kern;
    almost_eq(sup.x_em, expected, "combined f^2 kern");

    let eqs: Vec<_> = laid.glyphs.iter().filter(|g| g.glyph.ch == '=').collect();
    assert_eq!(eqs.len(), 2, "two aligned relations");
    almost_eq(eqs[0].x_em, eqs[1].x_em, "aligned = column");
    let close: Vec<_> = laid.glyphs.iter().filter(|g| g.glyph.ch == ')').collect();
    assert!(
        close.len() >= 4,
        "assembly rights plus two eqno closes, got {}",
        close.len()
    );
    let one = laid.glyphs.iter().find(|g| g.glyph.ch == '1').expect("1");
    let two = laid.glyphs.iter().find(|g| g.glyph.ch == '2').expect("2");
    assert!(
        (one.y_em - two.y_em).abs() > 1e-6,
        "equation numbers on different rows"
    );
}
