//! Example `pkg_ja_hang.rpx` — break-line + hang-width (+ justify-line).

use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::{break_line, hang_width_em_char, justify_line, HANG_WIDTH_EM};

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
fn pkg_ja_hang_example_eval() {
    let src = include_str!("../../../examples/pkg_ja_hang.rpx");
    let v = eval_source(src).expect("eval pkg_ja_hang.rpx");
    assert_eq!(
        field(&v, "tag"),
        &RuntimeValue::String("ja-hang-demo".into())
    );

    let lines = cons_strings(field(&v, "lines"));
    let expected = break_line("今日はいい天気です。東京タワーへ行こう。", 5.0);
    assert_eq!(lines, expected);
    assert!(lines.len() >= 2);

    assert_eq!(field(&v, "hang-stop"), &RuntimeValue::Number(HANG_WIDTH_EM));
    assert_eq!(field(&v, "hang-ideograph"), &RuntimeValue::Number(0.0));
    assert!((hang_width_em_char('。') - HANG_WIDTH_EM).abs() < 1e-9);

    let justified = field(&v, "justified");
    let chars: Vec<char> = "漢字列".chars().collect();
    let expected_j = justify_line(&chars, 5.0);
    // Just check length / first placement via cons walk.
    let mut n = 0usize;
    let mut cur = justified;
    while let RuntimeValue::Variant { tag, payload } = cur {
        if tag == "nil" {
            break;
        }
        assert_eq!(tag, "cons");
        let payload = payload.as_ref().expect("cons");
        let RuntimeValue::Record(fields) = payload.as_ref() else {
            panic!("record");
        };
        n += 1;
        cur = fields
            .iter()
            .find(|(k, _)| k == "tail")
            .map(|(_, v)| v)
            .expect("tail");
    }
    assert_eq!(n, expected_j.len());
}
