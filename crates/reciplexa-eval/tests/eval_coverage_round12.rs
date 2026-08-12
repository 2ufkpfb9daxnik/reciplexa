//! Round-12 eval: letrec/if/app cont inner-resume other arms.

use reciplexa_eval::eval_source;
use reciplexa_eval::RuntimeValue;

#[test]
fn if_non_bool_and_letrec_errors() {
    assert!(eval_source("(val main (if 1 2 3))").is_err());
    assert!(eval_source("(val main (letrec ((x 1)) x))").is_err());
}

#[test]
fn seq_mid_perform_resume_completes_tail() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 0))
    (seq (perform ask 0) 99)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(99));
}

#[test]
fn app_fun_perform_then_apply() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k (fn (x) x)))
    ((perform ask 0) 7)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(7));
}
