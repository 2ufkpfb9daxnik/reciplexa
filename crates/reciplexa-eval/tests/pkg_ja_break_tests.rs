//! Example `pkg_ja_break.rpx` — language builtin break-line → tagged line list.
//! Host path: `lines_to_text_shapes` / `break_line_to_text_shapes` → scene Text.

use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_scene::Color;
use reciplexa_std::japanese::{break_line, lines_to_text_shapes};

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
fn pkg_ja_break_example_eval_and_text_shapes() {
    let src = include_str!("../../../examples/pkg_ja_break.rpx");
    let v = eval_source(src).expect("eval pkg_ja_break.rpx");
    assert_eq!(
        field(&v, "tag"),
        &RuntimeValue::String("ja-break-demo".into())
    );

    let lines = cons_strings(field(&v, "lines"));
    let expected = break_line("今日はいい天気です。東京タワーへ行こう。", 5.0);
    assert_eq!(lines, expected);
    assert!(lines.len() >= 2, "demo should wrap: {lines:?}");

    // Host light consume: one scene Text per broken line (graphics-page scaffolding).
    let shapes = lines_to_text_shapes(&lines, 20.0, 30.0, 11.0, 14.0, Color::BLACK);
    assert_eq!(shapes.len(), lines.len());
    assert!(shapes.iter().all(|t| t.is_drawable()));
}
