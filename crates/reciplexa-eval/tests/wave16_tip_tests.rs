//! Wave 16 J2 tip coverage: place_lines + kihon pitch + lines_to_text_shapes.

use reciplexa_scene::Color;
use reciplexa_std::japanese::{
    lines_to_text_shapes, place_lines_horizontal, place_lines_vertical, KihonHanmen,
};

#[test]
fn tip_wave16_place_lines_and_text_shapes() {
    let lines = ["一", "二"];
    let kihon = KihonHanmen::default_horizontal();
    let pitch = kihon.line_pitch_em();
    assert!((pitch - 1.5).abs() < 1e-9);

    let placed = place_lines_horizontal(&lines, 0.0, pitch);
    assert!((placed[1].1 - pitch).abs() < 1e-9);

    let vert = place_lines_vertical(&lines, 5.0, pitch);
    assert!((vert[1].1 - (5.0 + pitch)).abs() < 1e-9);

    let shapes = lines_to_text_shapes(&lines, 1.0, 2.0, 10.0, pitch, Color::BLACK);
    assert_eq!(shapes.len(), 2);
    assert!((shapes[0].y_mm - 2.0).abs() < 1e-9);
    assert!((shapes[1].y_mm - (2.0 + pitch)).abs() < 1e-9);
    assert_eq!(shapes[0].content, "一");
}
