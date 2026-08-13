//! Language builtin `break-between` backed by `reciplexa_std::japanese`.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::{break_opportunity_chars, BreakOpportunity};

fn tag(name: &str) -> RuntimeValue {
    RuntimeValue::Variant {
        tag: name.into(),
        payload: None,
    }
}

#[test]
fn break_between_builtin_matches_std_opportunity() {
    let cases = [
        ("「", "あ", BreakOpportunity::Prohibited),
        ("あ", "。", BreakOpportunity::Prohibited),
        ("漢", "字", BreakOpportunity::Allowed),
        ("…", "あ", BreakOpportunity::Inseparable),
        ("A", "B", BreakOpportunity::Inseparable),
    ];
    for (prev, next, expected) in cases {
        let src = format!(r#"(val main (break-between "{prev}" "{next}"))"#);
        let v = eval_source(&src).unwrap();
        assert_eq!(v, tag(expected.as_str()), "pair {prev}{next}");
        assert_eq!(
            break_opportunity_chars(
                prev.chars().next().unwrap(),
                next.chars().next().unwrap()
            ),
            expected
        );
    }
}

#[test]
fn break_between_builtin_uses_first_chars() {
    let v = eval_source(r#"(val main (break-between "漢字" "。！"))"#).unwrap();
    assert_eq!(v, tag("prohibited"));
}

#[test]
fn break_between_builtin_rejects_bad_args() {
    let err = eval_source(r#"(val main (break-between "" "あ"))"#).unwrap_err();
    assert!(err.message.contains("non-empty"));
    let err = eval_source(r#"(val main (break-between "あ" 1))"#).unwrap_err();
    assert!(err.message.contains("string"));
    let err = eval_source(r#"(val main (break-between "あ"))"#).unwrap_err();
    assert!(err.message.contains("expects 2"));
}

#[test]
fn break_between_typechecks_lightly() {
    let ty = typecheck_language_source(r#"(val main (break-between "あ" "い"))"#).unwrap();
    // Dynamic / dyn_any is fine for tag result.
    assert!(matches!(ty, CoreType::Dynamic(_) | CoreType::Any) || ty == CoreType::dyn_any());
}
