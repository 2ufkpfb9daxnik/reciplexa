//! Round-14 eval: runtime_cast_ok edges, bytes/int display, graphics bridge via eval.

use reciplexa_eval::eval::{eval_source, eval_source_with_host};
use reciplexa_eval::graphics_value::document_from_graphics_value;
use reciplexa_eval::value::RuntimeValue;
use reciplexa_eval::EffectHost;

struct DropLog;
impl EffectHost for DropLog {
    fn perform(
        &mut self,
        _op: &str,
        _arg: RuntimeValue,
    ) -> Result<RuntimeValue, reciplexa_eval::EvalError> {
        Ok(RuntimeValue::Unit)
    }
}

#[test]
fn eval_bytes_int_and_check_cast_runtime() {
    let v = eval_source("(val main (bytes 1 2 255))").unwrap();
    assert!(matches!(v, RuntimeValue::Bytes(_)));
    let v = eval_source("(val main 42)").unwrap();
    match v {
        RuntimeValue::Int(42) => {}
        RuntimeValue::Number(42.0) => {}
        other => panic!("unexpected int literal value: {other:?}"),
    }
    let check = eval_source("(val main (check-cast \"x\" string))").unwrap();
    assert!(matches!(check, RuntimeValue::Variant { tag, .. } if tag == "ok"));
    let try_cast = eval_source("(val main (try-cast 1 string))").unwrap();
    assert!(matches!(try_cast, RuntimeValue::Variant { tag, .. } if tag == "none"));
}

#[test]
fn eval_log_host_no_stdout_side_effect() {
    let mut host = DropLog;
    let v = eval_source_with_host("(val main (perform log \"quiet\"))", &mut host).unwrap();
    assert_eq!(v, RuntimeValue::Unit);
}

#[test]
fn eval_package_like_page_record_via_value_bridge() {
    let page = RuntimeValue::Record(vec![
        ("tag".into(), RuntimeValue::String("page".into())),
        (
            "size".into(),
            RuntimeValue::Record(vec![
                ("width".into(), RuntimeValue::F64(210.0)),
                ("height".into(), RuntimeValue::F64(297.0)),
            ]),
        ),
        (
            "content".into(),
            RuntimeValue::Record(vec![
                ("tag".into(), RuntimeValue::String("fill".into())),
                (
                    "shape".into(),
                    RuntimeValue::Record(vec![
                        ("tag".into(), RuntimeValue::String("circle".into())),
                        ("x".into(), RuntimeValue::F64(50.0)),
                        ("y".into(), RuntimeValue::F64(50.0)),
                        ("r".into(), RuntimeValue::F64(20.0)),
                    ]),
                ),
                (
                    "color".into(),
                    RuntimeValue::Record(vec![
                        ("tag".into(), RuntimeValue::String("rgb".into())),
                        ("r".into(), RuntimeValue::F64(0.0)),
                        ("g".into(), RuntimeValue::F64(0.0)),
                        ("b".into(), RuntimeValue::F64(0.0)),
                    ]),
                ),
            ]),
        ),
    ]);
    let doc = document_from_graphics_value(&page).unwrap();
    assert_eq!(doc.pages.len(), 1);
    assert!(!doc.pages[0].shapes.is_empty());
}
