//! Wave 18 L2 tip coverage: lines_to_vertical_text_shapes + place_lines_vertical.

use reciplexa_scene::Color;
use reciplexa_std::japanese::{
    lines_to_vertical_text_shapes, place_lines_vertical, KihonHanmen,
};

#[test]
fn tip_wave18_vertical_text_shapes() {
    let lines = ["一", "二"];
    let pitch = KihonHanmen::default_vertical().line_pitch_em();
    let placed = place_lines_vertical(&lines, 0.0, pitch);
    let shapes = lines_to_vertical_text_shapes(&lines, 0.0, 8.0, 10.0, pitch, Color::BLACK);
    assert_eq!(shapes.len(), 2);
    assert!((shapes[0].x_mm - placed[0].1).abs() < 1e-9);
    assert!((shapes[1].x_mm - (pitch)).abs() < 1e-9);
    assert!((shapes[0].y_mm - 8.0).abs() < 1e-9);
    assert_eq!(shapes[1].content, "二");
}
