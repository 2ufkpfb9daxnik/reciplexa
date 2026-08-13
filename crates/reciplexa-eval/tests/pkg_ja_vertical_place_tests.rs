//! Example `pkg_ja_vertical_place.rpx` — vertical break → host place_lines_vertical.
//! Host path: `lines_to_vertical_text_shapes` → scene Text columns.

use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_scene::Color;
use reciplexa_std::japanese::{
    break_line_vertical, lines_to_vertical_text_shapes, place_lines_vertical, KihonHanmen,
};

fn field<'a>(rec: &'a RuntimeValue, name: &str) -> &'a RuntimeValue {
    match rec {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("missing field {name} in {rec}")),
        other => panic!("expected record, got {other}"),
    }
}

fn cons_strings(v: &RuntimeValue) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = v;
    loop {
        match cur {
            RuntimeValue::Variant { tag, .. } if tag == "nil" => break,
            RuntimeValue::Variant { tag, payload } if tag == "cons" => {
                let payload = payload.as_ref().expect("cons payload");
                let RuntimeValue::Record(fields) = payload.as_ref() else {
                    panic!("cons payload record");
                };
                let head = fields
                    .iter()
                    .find(|(k, _)| k == "head")
                    .map(|(_, v)| v)
                    .expect("head");
                let RuntimeValue::String(s) = head else {
                    panic!("head string");
                };
                out.push(s.clone());
                cur = fields
                    .iter()
                    .find(|(k, _)| k == "tail")
                    .map(|(_, v)| v)
                    .expect("tail");
            }
            other => panic!("expected cons list, got {other}"),
        }
    }
    out
}

#[test]
fn pkg_ja_vertical_place_example_eval_and_text_shapes() {
    let src = include_str!("../../../examples/pkg_ja_vertical_place.rpx");
    let v = eval_source(src).expect("eval pkg_ja_vertical_place.rpx");
    assert_eq!(
        field(&v, "tag"),
        &RuntimeValue::String("ja-vertical-place-demo".into())
    );

    let lines = cons_strings(field(&v, "lines"));
    let expected = break_line_vertical("春夏秋冬一二三四", 4.0);
    assert_eq!(lines, expected);
    assert!(lines.len() >= 2, "demo should wrap: {lines:?}");

    let pitch = KihonHanmen::default_vertical().line_pitch_em();
    let placed = place_lines_vertical(&lines, 10.0, pitch);
    let shapes = lines_to_vertical_text_shapes(&lines, 10.0, 40.0, 12.0, pitch, Color::BLACK);
    assert_eq!(shapes.len(), lines.len());
    assert_eq!(shapes.len(), placed.len());
    for (i, shape) in shapes.iter().enumerate() {
        assert!((shape.x_mm - placed[i].1).abs() < 1e-9);
        assert!((shape.y_mm - 40.0).abs() < 1e-9);
        assert_eq!(shape.content, placed[i].0);
        assert!(shape.is_drawable());
    }
}
