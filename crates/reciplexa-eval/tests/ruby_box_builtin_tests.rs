//! Language builtin `ruby-box` backed by `Ruby::estimate_box`.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::{Ruby, RUBY_HEIGHT_BUMP_EM};

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
fn ruby_box_builtin_matches_std() {
    let expected = Ruby::simple("漢", "かん").estimate_box();
    let v = eval_source(r#"(val main (ruby-box "漢" "かん"))"#).unwrap();
    let RuntimeValue::Record(fields) = v else {
        panic!("expected record, got {v:?}");
    };
    assert_eq!(
        fields
            .iter()
            .find(|(k, _)| k == "tag")
            .map(|(_, v)| v),
        Some(&RuntimeValue::String("ruby-box".into()))
    );
    assert!((field_num(&fields, "base-width") - expected.base_width).abs() < 1e-9);
    assert!((field_num(&fields, "annotation-width") - expected.annotation_width).abs() < 1e-9);
    assert!((field_num(&fields, "advance-width") - expected.advance_width).abs() < 1e-9);
    assert!((field_num(&fields, "height") - expected.height).abs() < 1e-9);
    assert!((expected.height - (1.0 + RUBY_HEIGHT_BUMP_EM)).abs() < 1e-9);
}

#[test]
fn ruby_box_builtin_rejects_bad_args() {
    let err = eval_source(r#"(val main (ruby-box 1 "かん"))"#).unwrap_err();
    assert!(err.message.contains("string"));
    let err = eval_source(r#"(val main (ruby-box "漢" 2))"#).unwrap_err();
    assert!(err.message.contains("string"));
    let err = eval_source(r#"(val main (ruby-box "漢"))"#).unwrap_err();
    assert!(err.message.contains("expects 2"));
}

#[test]
fn ruby_box_typechecks_lightly() {
    let ty = typecheck_language_source(r#"(val main (ruby-box "漢" "かん"))"#).unwrap();
    assert!(matches!(ty, CoreType::Dynamic(_) | CoreType::Any) || ty == CoreType::dyn_any());
}
