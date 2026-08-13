//! Language builtin `vertical-orientation` backed by `vertical_glyph_orientation`.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::vertical_glyph_orientation;

#[test]
fn vertical_orientation_builtin_matches_std() {
    let rotated = eval_source(r#"(val main (vertical-orientation "A"))"#).unwrap();
    assert_eq!(rotated, RuntimeValue::String("rotated".into()));
    assert_eq!(vertical_glyph_orientation('A').as_str(), "rotated");

    let upright = eval_source(r#"(val main (vertical-orientation "漢"))"#).unwrap();
    assert_eq!(upright, RuntimeValue::String("upright".into()));
    assert_eq!(vertical_glyph_orientation('漢').as_str(), "upright");

    let fullwidth = eval_source(r#"(val main (vertical-orientation "Ａ"))"#).unwrap();
    assert_eq!(fullwidth, RuntimeValue::String("upright".into()));
}

#[test]
fn vertical_orientation_builtin_uses_first_char_only() {
    let v = eval_source(r#"(val main (vertical-orientation "A漢"))"#).unwrap();
    assert_eq!(v, RuntimeValue::String("rotated".into()));
}

#[test]
fn vertical_orientation_builtin_rejects_empty_and_non_string() {
    let err = eval_source(r#"(val main (vertical-orientation ""))"#).unwrap_err();
    assert!(err.message.contains("non-empty"));
    let err = eval_source(r#"(val main (vertical-orientation 1))"#).unwrap_err();
    assert!(err.message.contains("string"));
    let err = eval_source(r#"(val main (vertical-orientation))"#).unwrap_err();
    assert!(err.message.contains("expects 1"));
}

#[test]
fn vertical_orientation_typechecks_as_string_to_string() {
    let ty = typecheck_language_source(r#"(val main (vertical-orientation "漢"))"#).unwrap();
    assert_eq!(ty, CoreType::String);
}
