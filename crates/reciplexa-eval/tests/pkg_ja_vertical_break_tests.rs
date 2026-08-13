//! Example `pkg_ja_vertical_break.rpx` — language builtin break-line-vertical.

use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::break_line_vertical;

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
fn pkg_ja_vertical_break_example_eval() {
    let src = include_str!("../../../examples/pkg_ja_vertical_break.rpx");
    let v = eval_source(src).expect("eval pkg_ja_vertical_break.rpx");
    assert_eq!(
        field(&v, "tag"),
        &RuntimeValue::String("ja-vertical-break-demo".into())
    );

    let lines = cons_strings(field(&v, "lines"));
    let expected = break_line_vertical("一二三四あいうえ", 3.0);
    assert_eq!(lines, expected);
    assert!(lines.len() >= 2, "demo should wrap: {lines:?}");
}
