//! Language builtin `classify-char` backed by `reciplexa_std::japanese`.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::{classify_char, CharClass};

#[test]
fn classify_char_builtin_matches_std_ids() {
    let cases = [
        ("「", CharClass::OpeningBrackets),
        ("あ", CharClass::Hiragana),
        ("漢", CharClass::Ideographic),
        ("A", CharClass::WesternCharacters),
        ("。", CharClass::FullStops),
    ];
    for (glyph, class) in cases {
        let src = format!(r#"(val main (classify-char "{glyph}"))"#);
        let v = eval_source(&src).unwrap();
        assert_eq!(
            v,
            RuntimeValue::Int(i128::from(class.id())),
            "glyph {glyph}"
        );
        assert_eq!(classify_char(glyph.chars().next().unwrap()), class);
    }
}

#[test]
fn classify_char_builtin_uses_first_char_only() {
    let v = eval_source(r#"(val main (classify-char "漢字"))"#).unwrap();
    assert_eq!(
        v,
        RuntimeValue::Int(i128::from(CharClass::Ideographic.id()))
    );
}

#[test]
fn classify_char_builtin_rejects_empty_and_non_string() {
    let err = eval_source(r#"(val main (classify-char ""))"#).unwrap_err();
    assert!(err.message.contains("non-empty"));
    let err = eval_source(r#"(val main (classify-char 1))"#).unwrap_err();
    assert!(err.message.contains("string"));
    let err = eval_source(r#"(val main (classify-char))"#).unwrap_err();
    assert!(err.message.contains("expects 1"));
}

#[test]
fn classify_char_typechecks_as_string_to_int() {
    let ty = typecheck_language_source(r#"(val main (classify-char "あ"))"#).unwrap();
    assert_eq!(ty, CoreType::Int);
}
