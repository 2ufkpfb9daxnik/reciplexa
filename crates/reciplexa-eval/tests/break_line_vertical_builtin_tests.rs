//! Language builtin `break-line-vertical` backed by std `break_line_vertical`.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::break_line_vertical;

fn cons_strings(items: &[&str]) -> RuntimeValue {
    let mut acc = RuntimeValue::Variant {
        tag: "nil".into(),
        payload: None,
    };
    for s in items.iter().rev() {
        acc = RuntimeValue::Variant {
            tag: "cons".into(),
            payload: Some(Box::new(RuntimeValue::Record(vec![
                ("head".into(), RuntimeValue::String((*s).into())),
                ("tail".into(), acc),
            ]))),
        };
    }
    acc
}

#[test]
fn break_line_vertical_builtin_matches_std() {
    let text = "一二三四";
    let max_em = 3.0;
    let expected = break_line_vertical(text, max_em);
    let src = format!(r#"(val main (break-line-vertical "{text}" {max_em}))"#);
    let v = eval_source(&src).unwrap();
    let refs: Vec<&str> = expected.iter().map(String::as_str).collect();
    assert_eq!(v, cons_strings(&refs));
    assert!(expected.len() >= 2, "demo should wrap: {expected:?}");
}

#[test]
fn break_line_vertical_builtin_edges() {
    let empty = eval_source(r#"(val main (break-line-vertical "" 4.0))"#).unwrap();
    assert_eq!(
        empty,
        RuntimeValue::Variant {
            tag: "nil".into(),
            payload: None
        }
    );
    let whole = eval_source(r#"(val main (break-line-vertical "東京" 0))"#).unwrap();
    assert_eq!(whole, cons_strings(&["東京"]));
}

#[test]
fn break_line_vertical_builtin_rejects_bad_args() {
    let err = eval_source(r#"(val main (break-line-vertical 1 2.0))"#).unwrap_err();
    assert!(err.message.contains("string"));
    let err = eval_source(r#"(val main (break-line-vertical "あ" "x"))"#).unwrap_err();
    assert!(err.message.contains("numeric"));
    let err = eval_source(r#"(val main (break-line-vertical "あ"))"#).unwrap_err();
    assert!(err.message.contains("expects 2"));
}

#[test]
fn break_line_vertical_typechecks_lightly() {
    let ty = typecheck_language_source(r#"(val main (break-line-vertical "あ" 2.0))"#).unwrap();
    assert!(matches!(ty, CoreType::Dynamic(_) | CoreType::Any) || ty == CoreType::dyn_any());
}
