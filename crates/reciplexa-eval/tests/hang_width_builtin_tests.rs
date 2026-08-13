//! Language builtin `hang-width` backed by std `hang_width_em_char`.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::{hang_width_em_char, HANG_WIDTH_EM};

#[test]
fn hang_width_builtin_matches_std() {
    let hang = eval_source(r#"(val main (hang-width "。"))"#).unwrap();
    assert_eq!(hang, RuntimeValue::Number(HANG_WIDTH_EM));
    assert!((hang_width_em_char('。') - HANG_WIDTH_EM).abs() < 1e-9);

    let comma = eval_source(r#"(val main (hang-width "、"))"#).unwrap();
    assert_eq!(comma, RuntimeValue::Number(HANG_WIDTH_EM));

    let zero = eval_source(r#"(val main (hang-width "あ"))"#).unwrap();
    assert_eq!(zero, RuntimeValue::Number(0.0));
}

#[test]
fn hang_width_builtin_uses_first_char_only() {
    let v = eval_source(r#"(val main (hang-width "。あ"))"#).unwrap();
    assert_eq!(v, RuntimeValue::Number(HANG_WIDTH_EM));
}

#[test]
fn hang_width_builtin_rejects_empty_and_non_string() {
    let err = eval_source(r#"(val main (hang-width ""))"#).unwrap_err();
    assert!(err.message.contains("non-empty"));
    let err = eval_source(r#"(val main (hang-width 1))"#).unwrap_err();
    assert!(err.message.contains("string"));
    let err = eval_source(r#"(val main (hang-width))"#).unwrap_err();
    assert!(err.message.contains("expects 1"));
}

#[test]
fn hang_width_typechecks_as_string_to_number() {
    let ty = typecheck_language_source(r#"(val main (hang-width "。"))"#).unwrap();
    assert_eq!(ty, CoreType::Number);
}
