//! Language builtin `tate-chu-yoko-width` backed by `TateChuYoko::estimate_box`.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::TateChuYoko;

#[test]
fn tate_chu_yoko_width_builtin_matches_std() {
    let expected = TateChuYoko::new("12").estimate_box().advance_width;
    let v = eval_source(r#"(val main (tate-chu-yoko-width "12"))"#).unwrap();
    match v {
        RuntimeValue::Number(n) | RuntimeValue::F64(n) => {
            assert!((n - expected).abs() < 1e-9);
        }
        other => panic!("expected number, got {other:?}"),
    }
    // ASCII digits are half-em each → 1.0 total.
    assert!((expected - 1.0).abs() < 1e-9);

    let latin = TateChuYoko::new("AB").estimate_box().advance_width;
    let v = eval_source(r#"(val main (tate-chu-yoko-width "AB"))"#).unwrap();
    match v {
        RuntimeValue::Number(n) | RuntimeValue::F64(n) => assert!((n - latin).abs() < 1e-9),
        other => panic!("expected number, got {other:?}"),
    }
}

#[test]
fn tate_chu_yoko_width_builtin_rejects_bad_args() {
    let err = eval_source(r#"(val main (tate-chu-yoko-width 12))"#).unwrap_err();
    assert!(err.message.contains("string"));
    let err = eval_source(r#"(val main (tate-chu-yoko-width))"#).unwrap_err();
    assert!(err.message.contains("expects 1"));
    let err = eval_source(r#"(val main (tate-chu-yoko-width "12" "x"))"#).unwrap_err();
    assert!(err.message.contains("expects 1"));
}

#[test]
fn tate_chu_yoko_width_typechecks_lightly() {
    let ty = typecheck_language_source(r#"(val main (tate-chu-yoko-width "12"))"#).unwrap();
    assert!(matches!(
        ty,
        CoreType::Number | CoreType::Dynamic(_) | CoreType::Any
    ));
}
