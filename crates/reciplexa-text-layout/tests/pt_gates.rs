//! Product typesetting gates (PT-0..PT-8) against the pinned fixture font.

use reciplexa_identity::document::StableNodeId;
use reciplexa_std::japanese::char_em_width;
use reciplexa_std::japanese::ParagraphSceneLayout;
use reciplexa_std::math::{EstimateStyle, MathAccentKind, MathAtom, MathClass};
use reciplexa_text_layout::{
    break_line_font, clusters_cover_input, fixture_font_bytes, glyph_advance_em,
    host_typeset_engine, justify_line_font, layout_math_atom,
    layout_wrapped_paragraph_product_lines, pair_matrix_fingerprint, positioned_lines_to_shapes,
    positioned_math_to_shapes, productize_shape_text, select_font, shape_run, FontRegistry,
    LayoutError, LoadedFont, MathConstantsEm, PositionedLine, TypesetEngine, FIXTURE_AXIS_HEIGHT,
    FIXTURE_FRACTION_RULE_THICKNESS, FIXTURE_SCRIPT_PERCENT_SCALE_DOWN, JLREQ_PROFILE_V1,
    JLREQ_PROFILE_V1_TABLES, MATH_PROFILE_V1,
};

fn font() -> LoadedFont {
    LoadedFont::fixture()
}

#[test]
fn pt0_fixture_parses_offline() {
    let bytes = fixture_font_bytes();
    assert!(bytes.len() > 100);
    let f = LoadedFont::from_bytes(bytes.to_vec(), "ReciplexaFixture").expect("parse fixture");
    assert_eq!(f.units_per_em(), 1000);
    assert_eq!(f.glyph_id('A').expect("A"), f.glyph_id('A').unwrap());
}

#[test]
fn pt0_advances_differ_from_stub_char_em_width() {
    let f = font();
    let a = glyph_advance_em(&f, 'A').expect("A");
    assert!((char_em_width('A') - 0.5).abs() < 1e-12);
    assert!((a - 0.6).abs() < 1e-9, "fixture A advance {a} em");
    let stop = glyph_advance_em(&f, '。').expect("ideographic stop");
    assert!((char_em_width('。') - 1.0).abs() < 1e-12);
    assert!((stop - 0.4).abs() < 1e-9, "fixture 。 advance {stop} em");
}

#[test]
fn pt0_clusters_cover_input_without_loss_or_overlap() {
    let f = font();
    let text = "Aあ。「x+日";
    let run = shape_run(&f, text).expect("shape");
    assert!(clusters_cover_input(&run));
    assert_eq!(run.glyphs.len(), text.chars().count());
    assert_eq!(run.text, text);
}

#[test]
fn pt0_missing_glyph_is_structured() {
    let f = font();
    let err = shape_run(&f, "☺").expect_err("missing emoji");
    match err {
        LayoutError::MissingGlyph { scalar, .. } => assert_eq!(scalar, '☺'),
        other => panic!("expected MissingGlyph, got {other:?}"),
    }
}

#[test]
fn pt0_missing_font_is_structured() {
    let registry = FontRegistry::new();
    let f = font();
    let err = registry.get(&f.id).expect_err("empty registry");
    assert!(matches!(err, LayoutError::MissingFont { .. }));
}

#[test]
fn pt1_font_widths_stable_and_not_stub() {
    let f = font();
    let w1 = glyph_advance_em(&f, 'x').unwrap();
    let w2 = glyph_advance_em(&f, 'x').unwrap();
    assert_eq!(w1, w2);
    assert!((w1 - 0.52).abs() < 1e-9);
    assert!((char_em_width('x') - 0.5).abs() < 1e-12);
}

#[test]
fn pt2_font_width_break_preserves_cl08_and_hang() {
    let f = font();
    // Inseparable ellipsis must stay together.
    let segs = break_line_font(&f, "……本", 1.5).expect("break");
    assert!(
        segs.iter().any(|s| s.text.contains("……")),
        "cl-08 glue: {segs:?}"
    );
    // Hangable 。 may sit past a tight measure.
    let hang = break_line_font(&f, "あ。", 0.98).expect("hang");
    assert_eq!(hang.len(), 1, "{hang:?}");
    assert_eq!(hang[0].text, "あ。");
}

#[test]
fn pt3_profile_v1_trim_and_justify() {
    assert_eq!(JLREQ_PROFILE_V1, "jlreq-profile-v1");
    assert_eq!(JLREQ_PROFILE_V1_TABLES, "jlreq-profile-v1.0");
    let f = font();
    let chars: Vec<char> = "「あ」".chars().collect();
    let placed = justify_line_font(&f, &chars, 3.0).expect("justify");
    assert_eq!(placed.len(), 3);
    // Line-head opening trim shifts first glyph left of 0.
    assert!(placed[0].1 < 0.0);
    let fp = pair_matrix_fingerprint();
    assert_eq!(fp, pair_matrix_fingerprint());
    assert_ne!(fp, 0);
}

#[test]
fn pt3_wrapped_lines_apply_trim_to_positioned_glyphs() {
    let f = font();
    let layout = ParagraphSceneLayout {
        base_x_mm: 20.0,
        start_y_mm: 200.0,
        size_mm: 4.0,
        pitch_mm: -6.0,
        fill: reciplexa_scene::Color::BLACK,
    };
    let lines =
        layout_wrapped_paragraph_product_lines(&f, "「あ」", 10.0, 0.0, &layout).expect("lines");
    assert_eq!(lines.len(), 1);
    let texts =
        reciplexa_text_layout::layout_wrapped_paragraph_product(&f, "「あ」", 10.0, 0.0, &layout)
            .expect("product wrap");
    let x = texts[0].text_x_mm().expect("glyph x");
    assert!(x < layout.base_x_mm, "trim must reach scene glyph x ({x})");
}

#[test]
fn pt4_math_constants_from_fixture() {
    let f = font();
    let c = MathConstantsEm::from_font(&f).expect("MATH constants");
    assert!((c.script_percent_scale_down - 0.70).abs() < 1e-9);
    assert_eq!(FIXTURE_SCRIPT_PERCENT_SCALE_DOWN, 70);
    let upem = f64::from(f.units_per_em());
    assert!((c.axis_height_em - f64::from(FIXTURE_AXIS_HEIGHT) / upem).abs() < 1e-9);
    let ic = f.italic_correction_em('x').unwrap_or(0.0);
    assert!(ic.abs() < 1.0);
}

#[test]
fn pt5_math_profile_scripts_fraction_stretchy() {
    assert_eq!(MATH_PROFILE_V1, "math-profile-v1");
    let f = font();
    let id = || StableNodeId::new(1);
    let x = MathAtom::symbol(id(), "x", MathClass::Ordinary);
    let two = MathAtom::symbol(id(), "2", MathClass::Ordinary);
    let scripts = MathAtom::scripts(id(), x.clone(), Some(two.clone()), None);
    let frac = MathAtom::fraction(
        id(),
        scripts,
        MathAtom::symbol(id(), "a", MathClass::Ordinary),
    );
    let delim = MathAtom::delimiter_with_stretch(id(), "(", ")", frac, 1.5);
    let laid = layout_math_atom(&f, &delim, EstimateStyle::Display).expect("math");
    assert!(laid.metrics.width > 0.0);
    assert!(!laid.glyphs.is_empty());
    assert!(!laid.rules.is_empty(), "fraction rule");
    let left = &laid.glyphs[0];
    assert_eq!(left.glyph.ch, '(');
    let cmap = f.glyph_id('(').unwrap();
    assert_ne!(
        left.glyph.gid, cmap,
        "stretchy '(' must select the construction-only tall variant, not cmap GID {cmap}"
    );
}

#[test]
fn pt6_positioned_output_one_text_per_glyph() {
    let f = font();
    let segs = break_line_font(
        &f,
        "あああああああああああああああああああああああああああああああああああああああ",
        10.0,
    )
    .expect("break");
    assert!(segs.len() > 1);
    let lines: Vec<_> = segs
        .iter()
        .enumerate()
        .map(|(i, s)| {
            PositionedLine::from_segment(f.id.clone(), s, 20.0, 200.0 - i as f64 * 6.0, 4.0, "ja")
        })
        .collect();
    let shapes = positioned_lines_to_shapes(&lines, reciplexa_scene::Color::BLACK);
    let n_glyphs: usize = lines.iter().map(|l| l.run.glyphs.len()).sum();
    assert_eq!(shapes.len(), n_glyphs);
    let first = match &shapes[0] {
        reciplexa_scene::Shape::GlyphRun(g) => g,
        other => panic!("expected GlyphRun, got {other:?}"),
    };
    assert_eq!(first.content.chars().count(), 1);
    assert!(first.gids[0] > 0);
}

#[test]
fn pt8_substitution_requires_relayout() {
    let a = font();
    select_font(&a.id, &a).expect("same digest ok");
    let b = LoadedFont::from_bytes(fixture_font_bytes().to_vec(), "OtherLabel").expect("parse");
    select_font(&a.id, &b).expect("same digest");
    let mut registry = FontRegistry::new();
    registry.insert(a.clone());
    assert!(registry.get(&a.id).is_ok());
    let mismatch = reciplexa_text_layout::FontId {
        label: "layout".into(),
        digest: "not-this-face".into(),
    };
    match select_font(&mismatch, &a) {
        Err(LayoutError::SubstitutionRequiresRelayout { .. }) => {}
        other => panic!("expected SubstitutionRequiresRelayout, got {other:?}"),
    }
}

#[test]
fn host_engine_default_is_product() {
    // Do not assert against a process-wide env the test runner may set; the
    // function must accept product as the documented default when unset/other.
    let e = host_typeset_engine();
    if std::env::var("RECIPLEXA_TYPESET_ENGINE")
        .map(|s| s.eq_ignore_ascii_case("stub"))
        .unwrap_or(false)
    {
        assert_eq!(e, TypesetEngine::Stub);
    } else {
        assert_eq!(e, TypesetEngine::Product);
    }
}

#[test]
fn math_scene_adapter_keeps_intra_row_x() {
    let f = font();
    let atom = MathAtom::row(
        StableNodeId::new(1),
        vec![
            MathAtom::symbol(StableNodeId::new(2), "x", MathClass::Ordinary),
            MathAtom::symbol(StableNodeId::new(3), "+", MathClass::Binary),
            MathAtom::symbol(StableNodeId::new(4), "y", MathClass::Ordinary),
        ],
    );
    let laid = layout_math_atom(&f, &atom, EstimateStyle::Display).unwrap();
    let shapes = positioned_math_to_shapes(
        &laid,
        (10.0, 100.0),
        4.0,
        reciplexa_scene::Color::BLACK,
        &f.id,
    );
    let texts: Vec<_> = shapes
        .iter()
        .filter_map(|s| match s {
            reciplexa_scene::Shape::GlyphRun(t) => Some(t),
            _ => None,
        })
        .collect();
    let x = texts.iter().find(|t| t.content == "x").expect("x");
    let plus = texts.iter().find(|t| t.content == "+").expect("+");
    assert!(plus.x_mm > x.x_mm, "advances must survive the adapter");
}

#[test]
fn math_scene_adapter_fraction_is_y_up() {
    let f = font();
    let frac = MathAtom::fraction(
        StableNodeId::new(1),
        MathAtom::symbol(StableNodeId::new(2), "a", MathClass::Ordinary),
        MathAtom::symbol(StableNodeId::new(3), "b", MathClass::Ordinary),
    );
    let laid = layout_math_atom(&f, &frac, EstimateStyle::Display).unwrap();
    let shapes = positioned_math_to_shapes(
        &laid,
        (10.0, 100.0),
        4.0,
        reciplexa_scene::Color::BLACK,
        &f.id,
    );
    let a = shapes
        .iter()
        .find_map(|s| match s {
            reciplexa_scene::Shape::GlyphRun(t) if t.content == "a" => Some(t.y_mm),
            _ => None,
        })
        .expect("num");
    let b = shapes
        .iter()
        .find_map(|s| match s {
            reciplexa_scene::Shape::GlyphRun(t) if t.content == "b" => Some(t.y_mm),
            _ => None,
        })
        .expect("den");
    assert!(
        a > b,
        "numerator must sit above denominator in y-up scene (a={a}, b={b})"
    );
}

#[test]
fn pt5_unsupported_accent_is_not_ascii_substitute() {
    let f = font();
    let atom = MathAtom::accent(
        StableNodeId::new(1),
        MathAccentKind::Check,
        MathAtom::symbol(StableNodeId::new(2), "x", MathClass::Ordinary),
    );
    let err = layout_math_atom(&f, &atom, EstimateStyle::Display).expect_err("check");
    match err {
        LayoutError::Engine { detail } => assert!(detail.contains("check"), "{detail}"),
        other => panic!("expected Engine, got {other:?}"),
    }
}

#[test]
fn pt5_fraction_rule_uses_math_thickness() {
    let f = font();
    let frac = MathAtom::fraction(
        StableNodeId::new(1),
        MathAtom::symbol(StableNodeId::new(2), "a", MathClass::Ordinary),
        MathAtom::symbol(StableNodeId::new(3), "b", MathClass::Ordinary),
    );
    let laid = layout_math_atom(&f, &frac, EstimateStyle::Display).unwrap();
    let th = laid.rules[0].thickness_em;
    let expect = f64::from(FIXTURE_FRACTION_RULE_THICKNESS) / 1000.0;
    assert!((th - expect).abs() < 1e-9, "thickness {th} vs {expect}");
    let shapes = positioned_math_to_shapes(
        &laid,
        (10.0, 100.0),
        4.0,
        reciplexa_scene::Color::BLACK,
        &f.id,
    );
    let w = shapes
        .iter()
        .find_map(|s| match s {
            reciplexa_scene::Shape::Line(l) => Some(l.width_mm),
            _ => None,
        })
        .expect("rule");
    assert!((w - th * 4.0).abs() < 1e-9, "scene width {w}");
}

#[test]
fn pt5_stretchy_delim_gid_reaches_scene_glyph_run() {
    let f = font();
    let delim = MathAtom::delimiter_with_stretch(
        StableNodeId::new(1),
        "(",
        ")",
        MathAtom::symbol(StableNodeId::new(2), "x", MathClass::Ordinary),
        1.5,
    );
    let laid = layout_math_atom(&f, &delim, EstimateStyle::Display).unwrap();
    let left = laid
        .glyphs
        .iter()
        .find(|g| g.glyph.ch == '(')
        .expect("left");
    let shapes = positioned_math_to_shapes(
        &laid,
        (10.0, 100.0),
        4.0,
        reciplexa_scene::Color::BLACK,
        &f.id,
    );
    let painted = shapes
        .iter()
        .find_map(|s| match s {
            reciplexa_scene::Shape::GlyphRun(g) if g.content == "(" => Some(g),
            _ => None,
        })
        .expect("scene glyph");
    assert_eq!(painted.gids, vec![left.glyph.gid]);
    let cmap = f.glyph_id('(').unwrap();
    assert_ne!(
        left.glyph.gid, cmap,
        "stretchy '(' must paint a MATH variant GID, not cmap {cmap}"
    );
}

#[test]
fn graphics_text_productize_keeps_one_cluster_run() {
    let f = font();
    let shaped = productize_shape_text(
        reciplexa_scene::Shape::Text(reciplexa_scene::Text {
            x_mm: 30.0,
            y_mm: 260.0,
            size_mm: 8.0,
            width_mm: None,
            height_mm: None,
            content: "Reciplexa".into(),
            fill: reciplexa_scene::Color::BLACK,
        }),
        &f,
    )
    .expect("productize");
    let g = match shaped {
        reciplexa_scene::Shape::GlyphRun(g) => g,
        other => panic!("expected GlyphRun, got {other:?}"),
    };
    assert_eq!(g.content, "Reciplexa");
    assert_eq!(g.gids.len(), "Reciplexa".chars().count());
    assert_eq!(g.gids.len(), g.advances_mm.len());
    assert_eq!(g.font_digest, f.id.digest);
    assert_eq!(g.gids[0], f.glyph_id('R').unwrap());
}
