//! Step 8 item 2: vertical-rl stacking + GSUB `vert`.

use reciplexa_std::japanese::needs_tate_rotation;
use reciplexa_text_layout::{
    host_product_font, layout_vertical_run, positioned_vertical_to_shapes, vert_substitute_gid,
    LoadedFont, WritingMode,
};

fn font() -> LoadedFont {
    LoadedFont::fixture()
}

#[test]
fn vertical_run_stacks_down_the_page() {
    let f = font();
    let origin_y = 120.0;
    let size = 10.0;
    let laid = layout_vertical_run(&f, "漢字", 40.0, origin_y, size).expect("layout");
    assert_eq!(laid.run.writing_mode, WritingMode::VerticalRl);
    assert_eq!(laid.run.glyphs.len(), 2);
    assert!((laid.run.glyphs[0].y_mm - origin_y).abs() < 1e-9);
    assert!(
        laid.run.glyphs[1].y_mm < laid.run.glyphs[0].y_mm - size * 0.5,
        "second glyph must sit below the first, got {} then {}",
        laid.run.glyphs[0].y_mm,
        laid.run.glyphs[1].y_mm
    );
    assert!(laid.height_mm > size);
}

#[test]
fn vertical_latin_is_rotated() {
    let f = font();
    let laid = layout_vertical_run(&f, "A漢", 10.0, 80.0, 8.0).expect("layout");
    assert_eq!(laid.rotation_deg[0], -90.0);
    assert_eq!(laid.rotation_deg[1], 0.0);
    assert!(needs_tate_rotation('A'));
    assert!(!needs_tate_rotation('漢'));
    let shapes = positioned_vertical_to_shapes(&laid, reciplexa_scene::Color::BLACK);
    assert!(matches!(shapes[0], reciplexa_scene::Shape::Group { .. }));
    assert!(matches!(shapes[1], reciplexa_scene::Shape::GlyphRun(_)));
}

#[test]
fn fixture_vert_is_identity() {
    let f = font();
    let gid = f.glyph_id('漢').unwrap();
    assert_eq!(vert_substitute_gid(&f, gid), gid);
}

#[test]
fn host_cjk_vert_may_rewrite_ideographic_stop() {
    let f = host_product_font().expect("host font");
    let Ok(cmap) = f.glyph_id('。') else {
        return;
    };
    let vert = vert_substitute_gid(&f, cmap);
    if f.is_fixture() {
        assert_eq!(vert, cmap);
        return;
    }
    if f.face().tables().gsub.is_some() {
        assert_ne!(
            vert, cmap,
            "host CJK GSUB vert should rewrite ideographic stop"
        );
    }
}

#[test]
fn empty_vertical_is_refused() {
    let f = font();
    assert!(layout_vertical_run(&f, "\n", 0.0, 0.0, 10.0).is_err());
}
