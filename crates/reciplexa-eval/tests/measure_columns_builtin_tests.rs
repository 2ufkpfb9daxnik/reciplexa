//! Language builtin `measure-columns` backed by std `measure_columns`.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::ty::CoreType;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::measure_columns;

fn field<'a>(rec: &'a RuntimeValue, name: &str) -> &'a RuntimeValue {
    match rec {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("missing field {name}")),
        other => panic!("expected record, got {other}"),
    }
}

fn cons_numbers(v: &RuntimeValue) -> Vec<f64> {
    let mut out = Vec::new();
    let mut cur = v;
    loop {
        match cur {
            RuntimeValue::Variant { tag, .. } if tag == "nil" => break,
            RuntimeValue::Variant { tag, payload } if tag == "cons" => {
                let payload = payload.as_ref().expect("cons payload");
                let RuntimeValue::Record(fields) = payload.as_ref() else {
                    panic!("cons payload record");
                };
                let head = fields
                    .iter()
                    .find(|(k, _)| k == "head")
                    .map(|(_, v)| v)
                    .expect("head");
                let RuntimeValue::Number(n) = head else {
                    panic!("head number");
                };
                out.push(*n);
                cur = fields
                    .iter()
                    .find(|(k, _)| k == "tail")
                    .map(|(_, v)| v)
                    .expect("tail");
            }
            other => panic!("expected cons list, got {other}"),
        }
    }
    out
}

#[test]
fn measure_columns_builtin_matches_std() {
    let v = eval_source(r#"(val main (measure-columns 20 2 2))"#).unwrap();
    assert_eq!(
        field(&v, "tag"),
        &RuntimeValue::String("measure-columns".into())
    );
    let (col_w, xs) = measure_columns(20.0, 2, 2.0);
    match field(&v, "col-w") {
        RuntimeValue::Number(w) => assert!((w - col_w).abs() < 1e-9),
        other => panic!("col-w number, got {other}"),
    }
    assert_eq!(cons_numbers(field(&v, "xs")), xs);
}

#[test]
fn measure_columns_builtin_three_cols() {
    let v = eval_source(r#"(val main (measure-columns 32.0 3 1.0))"#).unwrap();
    let (col_w, xs) = measure_columns(32.0, 3, 1.0);
    match field(&v, "col-w") {
        RuntimeValue::Number(w) => assert!((w - col_w).abs() < 1e-9),
        other => panic!("col-w, got {other}"),
    }
    assert_eq!(cons_numbers(field(&v, "xs")), xs);
}

#[test]
fn measure_columns_builtin_rejects_bad_args() {
    let err = eval_source(r#"(val main (measure-columns "x" 2 1))"#).unwrap_err();
    assert!(err.message.contains("total-em"));
    let err = eval_source(r#"(val main (measure-columns 20 "x" 1))"#).unwrap_err();
    assert!(err.message.contains("count"));
    let err = eval_source(r#"(val main (measure-columns 20 2 "x"))"#).unwrap_err();
    assert!(err.message.contains("gutter-em"));
    let err = eval_source(r#"(val main (measure-columns 20 2))"#).unwrap_err();
    assert!(err.message.contains("expects 3"));
}

#[test]
fn measure_columns_typechecks() {
    let ty = typecheck_language_source(r#"(val main (measure-columns 20 2 1))"#).unwrap();
    assert_eq!(ty, CoreType::dyn_any());
}
