//! Example `pkg_ja_classify.rpx` — language builtins classify-char / break-between.

use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::{break_opportunity_chars, classify_char, BreakOpportunity, CharClass};

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

#[test]
fn pkg_ja_classify_example_eval() {
    let src = include_str!("../../../examples/pkg_ja_classify.rpx");
    let v = eval_source(src).expect("eval pkg_ja_classify.rpx");
    assert_eq!(
        field(&v, "tag"),
        &RuntimeValue::String("ja-classify-demo".into())
    );

    assert_eq!(
        field(&v, "open-class"),
        &RuntimeValue::Int(i128::from(CharClass::OpeningBrackets.id()))
    );
    assert_eq!(
        field(&v, "ideograph-class"),
        &RuntimeValue::Int(i128::from(CharClass::Ideographic.id()))
    );
    assert_eq!(
        field(&v, "wave-class"),
        &RuntimeValue::Int(i128::from(CharClass::Hyphens.id()))
    );
    assert_eq!(
        field(&v, "space-class"),
        &RuntimeValue::Int(i128::from(CharClass::Spaces.id()))
    );

    assert_eq!(
        classify_char('「'),
        CharClass::OpeningBrackets,
        "std parity for open"
    );
    assert_eq!(
        break_opportunity_chars('「', 'あ'),
        BreakOpportunity::Prohibited
    );

    assert!(matches!(
        field(&v, "after-open"),
        RuntimeValue::Variant { tag, .. } if tag == "prohibited"
    ));
    assert!(matches!(
        field(&v, "before-stop"),
        RuntimeValue::Variant { tag, .. } if tag == "prohibited"
    ));
    assert!(matches!(
        field(&v, "between-kanji"),
        RuntimeValue::Variant { tag, .. } if tag == "allowed"
    ));
}
