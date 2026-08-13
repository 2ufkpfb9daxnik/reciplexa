//! Wave 13 G2 tip coverage: needs_tate_rotation / VerticalGlyphOrientation.

use reciplexa_std::japanese::{
    needs_tate_rotation, vertical_glyph_orientation, VerticalGlyphOrientation,
};

#[test]
fn tip_wave13_tate_rotation_orientation() {
    assert!(needs_tate_rotation('B'));
    assert!(!needs_tate_rotation('東'));
    assert_eq!(
        vertical_glyph_orientation('1'),
        VerticalGlyphOrientation::Rotated
    );
    assert_eq!(
        vertical_glyph_orientation('々'),
        VerticalGlyphOrientation::Upright
    );
    assert_eq!(VerticalGlyphOrientation::Rotated.as_str(), "rotated");
}
