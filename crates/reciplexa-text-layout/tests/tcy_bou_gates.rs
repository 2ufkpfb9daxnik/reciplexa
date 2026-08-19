//! Step 8 item 3: tate-chu-yoko and bou vs stub estimates.

use reciplexa_std::japanese::{char_em_width, TateChuYoko, BOU_MARK_SIZE_EM};
use reciplexa_text_layout::{
    layout_bou_horizontal, layout_bou_vertical, layout_tate_chu_yoko, layout_vertical_run,
    positioned_bou_horizontal_to_shapes, positioned_bou_vertical_to_shapes,
    positioned_tcy_to_shapes, LayoutError, LoadedFont, BOU_PARENT_GAP_EM,
};

fn font() -> LoadedFont {
    LoadedFont::fixture()
}

#[test]
fn tcy_widths_differ_from_stub_for_latin() {
    let f = font();
    let span = TateChuYoko::new("AB");
    let stub = span.estimate_box();
    let laid = layout_tate_chu_yoko(&f, &span, 10.0, 80.0, 10.0).expect("tcy");
    let stub_w: f64 = span.body.chars().map(char_em_width).sum();
    assert!((stub.advance_width - stub_w).abs() < 1e-12);
    let font_em: f64 = span
        .body
        .chars()
        .map(|c| f.hor_advance_em(c).unwrap())
        .sum();
    assert!(
        (font_em - stub_w).abs() > 1e-6,
        "fixture latin must differ from char_em_width"
    );
    assert!((laid.line.width_mm - font_em * laid.line.run.size_mm).abs() < 1e-6);
    assert!((laid.block_mm - 10.0).abs() < 1e-9);
}

#[test]
fn tcy_stays_upright_and_fits_one_em() {
    let f = font();
    let span = TateChuYoko::new("12");
    let size = 12.0;
    let origin_x = 40.0;
    let laid = layout_tate_chu_yoko(&f, &span, origin_x, 100.0, size).expect("tcy");
    assert!(laid.line.width_mm <= size + 1e-9);
    assert!(laid.line.x_mm + 1e-9 >= origin_x);
    assert!(laid.line.x_mm + laid.line.width_mm <= origin_x + size + 1e-6);
    let shapes = positioned_tcy_to_shapes(&laid, reciplexa_scene::Color::BLACK);
    assert_eq!(shapes.len(), 2);
    assert!(shapes
        .iter()
        .all(|s| matches!(s, reciplexa_scene::Shape::GlyphRun(_))));
}

#[test]
fn tcy_refuses_cjk_and_long_runs() {
    let f = font();
    let err = layout_tate_chu_yoko(&f, &TateChuYoko::new("漢"), 0.0, 0.0, 10.0).unwrap_err();
    match err {
        LayoutError::Engine { detail } => assert!(detail.contains("ASCII"), "{detail}"),
        other => panic!("{other:?}"),
    }
    let err = layout_tate_chu_yoko(&f, &TateChuYoko::new("12345"), 0.0, 0.0, 10.0).unwrap_err();
    match err {
        LayoutError::Engine { detail } => assert!(detail.contains("at most"), "{detail}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn tcy_is_narrower_than_tate_rotated_digits() {
    let f = font();
    let size = 10.0;
    let tcy = layout_tate_chu_yoko(&f, &TateChuYoko::new("123"), 0.0, 50.0, size).expect("tcy");
    let rotated = layout_vertical_run(&f, "123", 0.0, 50.0, size).expect("vert");
    assert!(
        tcy.block_mm + 1e-6 < rotated.height_mm,
        "tate-chu-yoko must occupy one em, not stacked rotated digits ({} vs {})",
        tcy.block_mm,
        rotated.height_mm
    );
}

#[test]
fn bou_horizontal_marks_sit_above_body() {
    let f = font();
    let size = 10.0;
    let laid = layout_bou_horizontal(&f, "重要", 20.0, 40.0, size).expect("bou");
    let shapes = positioned_bou_horizontal_to_shapes(&laid, reciplexa_scene::Color::BLACK);
    let marks: Vec<_> = shapes
        .iter()
        .filter_map(|s| match s {
            reciplexa_scene::Shape::Circle(c) => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(marks.len(), 2);
    let radius = size * BOU_MARK_SIZE_EM * 0.5;
    let clear = size * BOU_PARENT_GAP_EM + radius;
    let juu = laid
        .body
        .run
        .glyphs
        .iter()
        .find(|g| g.ch == '重')
        .expect("重");
    for m in &marks {
        assert!(
            m.y_mm + 1e-9 >= juu.y_mm + size + clear,
            "mark must sit above the em-square plus gap, y={} want>={}",
            m.y_mm,
            juu.y_mm + size + clear
        );
        assert!((m.radius_mm - radius).abs() < 1e-9);
    }
    let mark_juu = marks
        .iter()
        .min_by(|a, b| a.x_mm.partial_cmp(&b.x_mm).unwrap())
        .unwrap();
    assert!(
        (mark_juu.x_mm - (juu.x_mm + juu.advance_mm * 0.5)).abs() < 1e-6,
        "horizontal bou is centered on the parent advance"
    );
}

#[test]
fn bou_vertical_marks_sit_beside_column() {
    let f = font();
    let size = 8.0;
    let x = 30.0;
    let laid = layout_bou_vertical(&f, "漢", x, 90.0, size).expect("bou v");
    let shapes = positioned_bou_vertical_to_shapes(&laid, reciplexa_scene::Color::BLACK);
    let mark = shapes
        .iter()
        .find_map(|s| match s {
            reciplexa_scene::Shape::Circle(c) => Some(c),
            _ => None,
        })
        .expect("mark");
    assert!(
        mark.x_mm + 1e-9 >= x + size * (1.0 + BOU_PARENT_GAP_EM),
        "vertical bou sits to the right of the em-square, x={} origin={}",
        mark.x_mm,
        x
    );
    assert!(
        (mark.y_mm - (90.0 + size * 0.5)).abs() < 1e-6,
        "vertical bou is centered on the parent em-square"
    );
}

#[test]
fn empty_bou_is_refused() {
    let f = font();
    assert!(layout_bou_horizontal(&f, "", 0.0, 0.0, 10.0).is_err());
}
