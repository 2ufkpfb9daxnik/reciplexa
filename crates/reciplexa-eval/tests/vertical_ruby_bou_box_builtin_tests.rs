//! Language builtins `vertical-ruby-box` / `bou-box` (thin wrappers).

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::{bou_estimate_box, Ruby};

fn field_num(fields: &[(String, RuntimeValue)], name: &str) -> f64 {
    fields
        .iter()
        .find(|(k, _)| k == name)
        .and_then(|(_, v)| match v {
            RuntimeValue::Number(n) => Some(*n),
            RuntimeValue::F64(n) => Some(*n),
            RuntimeValue::Int(n) => Some(*n as f64),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing numeric field `{name}`"))
}

#[test]
fn vertical_ruby_box_builtin_matches_std() {
    let expected = Ruby::simple("東", "とう").estimate_vertical_box();
    let v = eval_source(r#"(val main (vertical-ruby-box "東" "とう"))"#).unwrap();
    let RuntimeValue::Record(fields) = v else {
        panic!("expected record, got {v:?}");
    };
    assert_eq!(
        fields
            .iter()
            .find(|(k, _)| k == "tag")
            .map(|(_, v)| v),
        Some(&RuntimeValue::String("vertical-ruby-box".into()))
    );
    assert!((field_num(&fields, "base-advance") - expected.base_advance).abs() < 1e-9);
    assert!((field_num(&fields, "annotation-advance") - expected.annotation_advance).abs() < 1e-9);
    assert!((field_num(&fields, "advance") - expected.advance).abs() < 1e-9);
    assert!((field_num(&fields, "inline-em") - expected.inline_em).abs() < 1e-9);
    assert!((field_num(&fields, "annotation-side-x") - expected.annotation_side_x).abs() < 1e-9);
}

#[test]
fn bou_box_builtin_matches_std() {
    let expected = bou_estimate_box("重要");
    let v = eval_source(r#"(val main (bou-box "重要"))"#).unwrap();
    let RuntimeValue::Record(fields) = v else {
        panic!("expected record, got {v:?}");
    };
    assert_eq!(
        fields
            .iter()
            .find(|(k, _)| k == "tag")
            .map(|(_, v)| v),
        Some(&RuntimeValue::String("bou-box".into()))
    );
    assert!((field_num(&fields, "advance") - expected.advance).abs() < 1e-9);
    assert!((field_num(&fields, "side-em") - expected.side_em).abs() < 1e-9);
    assert!((field_num(&fields, "mark-size") - expected.mark_size).abs() < 1e-9);
}

#[test]
fn vertical_ruby_bou_box_reject_bad_args() {
    let err = eval_source(r#"(val main (vertical-ruby-box 1 "とう"))"#).unwrap_err();
    assert!(err.message.contains("string"));
    let err = eval_source(r#"(val main (vertical-ruby-box "東"))"#).unwrap_err();
    assert!(err.message.contains("expects 2"));
    let err = eval_source(r#"(val main (bou-box 1))"#).unwrap_err();
    assert!(err.message.contains("string"));
    let err = eval_source(r#"(val main (bou-box "a" "b"))"#).unwrap_err();
    assert!(err.message.contains("expects 1"));
}

#[test]
fn vertical_ruby_bou_box_typecheck_lightly() {
    let ty = typecheck_language_source(r#"(val main (vertical-ruby-box "東" "とう"))"#).unwrap();
    assert!(matches!(ty, CoreType::Dynamic(_) | CoreType::Any) || ty == CoreType::dyn_any());
    let ty = typecheck_language_source(r#"(val main (bou-box "傍"))"#).unwrap();
    assert!(matches!(ty, CoreType::Dynamic(_) | CoreType::Any) || ty == CoreType::dyn_any());
}
