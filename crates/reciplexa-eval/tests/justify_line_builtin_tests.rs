//! Language builtin `justify-line` backed by std `justify_line`.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::justify_line;

fn cons_placements(items: &[(char, f64)]) -> RuntimeValue {
    let mut acc = RuntimeValue::Variant {
        tag: "nil".into(),
        payload: None,
    };
    for &(ch, x) in items.iter().rev() {
        let mut s = String::new();
        s.push(ch);
        acc = RuntimeValue::Variant {
            tag: "cons".into(),
            payload: Some(Box::new(RuntimeValue::Record(vec![
                (
                    "head".into(),
                    RuntimeValue::Record(vec![
                        ("char".into(), RuntimeValue::String(s)),
                        ("x".into(), RuntimeValue::Number(x)),
                    ]),
                ),
                ("tail".into(), acc),
            ]))),
        };
    }
    acc
}

#[test]
fn justify_line_builtin_matches_std() {
    let text = "あいう";
    let target = 5.0;
    let chars: Vec<char> = text.chars().collect();
    let expected = justify_line(&chars, target);
    let src = format!(r#"(val main (justify-line "{text}" {target}))"#);
    let v = eval_source(&src).unwrap();
    assert_eq!(v, cons_placements(&expected));
    assert_eq!(expected.len(), 3);
    assert!((expected[0].1 - 0.0).abs() < 1e-9);
    assert!((expected[2].1 - (5.0 - 1.0)).abs() < 1e-6);
}

#[test]
fn justify_line_builtin_empty() {
    let empty = eval_source(r#"(val main (justify-line "" 4.0))"#).unwrap();
    assert_eq!(
        empty,
        RuntimeValue::Variant {
            tag: "nil".into(),
            payload: None
        }
    );
}

#[test]
fn justify_line_builtin_rejects_bad_args() {
    let err = eval_source(r#"(val main (justify-line 1 2.0))"#).unwrap_err();
    assert!(err.message.contains("string"));
    let err = eval_source(r#"(val main (justify-line "あ" "x"))"#).unwrap_err();
    assert!(err.message.contains("numeric"));
    let err = eval_source(r#"(val main (justify-line "あ"))"#).unwrap_err();
    assert!(err.message.contains("expects 2"));
}

#[test]
fn justify_line_typechecks_lightly() {
    let ty = typecheck_language_source(r#"(val main (justify-line "あ" 2.0))"#).unwrap();
    assert!(matches!(ty, CoreType::Dynamic(_) | CoreType::Any) || ty == CoreType::dyn_any());
}
