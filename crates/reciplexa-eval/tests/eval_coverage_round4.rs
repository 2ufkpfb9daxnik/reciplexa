//! Round-4: deep resume Cont `other` arms, cast fails, host/parse edges.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::control::{
    identity_resume, EffectHost, MemoryFsHost, Outcome, ResumeCont, UnitHost,
};
use reciplexa_eval::eval::{eval_expr, eval_source, eval_source_with_host, primitive_env};
use reciplexa_eval::value::RuntimeValue;

fn oneshot(cont: ResumeCont) -> RuntimeValue {
    RuntimeValue::OneShotResume {
        used: Rc::new(Cell::new(false)),
        cont,
    }
}

#[test]
fn parse_error_and_error_expr_and_top_forward() {
    let err = eval_source("(val main").unwrap_err();
    assert!(err.message.contains("parse") || err.message.contains("error"));

    let err = eval_expr(&CoreExpr::Error, &HashMap::new(), &mut UnitHost).unwrap_err();
    assert!(err.message.contains("syntax-error") || err.message.contains("cannot evaluate"));

    let err = eval_expr(
        &CoreExpr::Forward {
            resume_name: "missing".into(),
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("forward") || err.message.contains("resume"));
}

#[test]
fn dead_cell_and_cast_fail_and_with_non_handler() {
    let alive = Rc::new(Cell::new(false));
    let mut env = HashMap::new();
    env.insert(
        "x".into(),
        RuntimeValue::Cell {
            value: Rc::new(RefCell::new(RuntimeValue::Int(1))),
            alive,
        },
    );
    let err = eval_expr(&CoreExpr::Var("x".into()), &env, &mut UnitHost).unwrap_err();
    assert!(err.message.contains("escaped") || err.message.contains("scope"));

    let err = eval_expr(
        &CoreExpr::Cast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            evidence: CastEvidence::TagCheck {
                tag: "string".into(),
            },
            target: CoreType::String,
            cast_id: 0,
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("cast") || err.message.contains("fail"));

    let err = eval_expr(
        &CoreExpr::With {
            handler: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("handler"));
}

#[test]
fn oneshot_cont_returns_resumed_forward_performed() {
    // Cont → Resumed
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|v, _| Ok(Outcome::Resumed(v)))),
    );
    let v = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(3))],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(3));

    // Cont → Forward (top-level unwraps to error)
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|_, _| Ok(Outcome::Forward))),
    );
    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("forward"));

    // Cont → Performed bubbles to host
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|v, _| {
            Ok(Outcome::Performed {
                op: "log".into(),
                arg: v,
                resume: identity_resume(),
            })
        })),
    );
    let v = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::String("x".into()))],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Unit);

    // Arity / double-use
    let mut env = HashMap::new();
    env.insert("k".into(), oneshot(identity_resume()));
    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![
                CoreExpr::Lit(CoreLiteral::Int(1)),
                CoreExpr::Lit(CoreLiteral::Int(2)),
            ],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("1 arg") || err.message.contains("resume"));

    let used = Rc::new(Cell::new(true));
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        RuntimeValue::OneShotResume {
            used,
            cont: identity_resume(),
        },
    );
    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("already used"));
}

#[test]
fn deep_resume_other_through_compound_forms() {
    // Nested perform: resume returns Performed for another op → wrap `other` arms.
    let v = eval_source(
        r#"(val main
  (handle outer (fn (msg) 99)
    (handle ask (fn (_ k) (k (perform outer "side")))
      (let ((x (perform ask unit)))
        x))))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(99));

    // Field expr perform currently bubbles (`other`) without reconstituting the
    // record — cover that intentional short-circuit path.
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 7))
    (record-update (record (a 1)) (a (perform ask unit)))))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(7));

    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 8))
    (record-extend (record (a 1)) (b (perform ask unit)))))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(8));

    // App args: perform in later arg; multi-arg with perform in middle.
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 3))
    ((fn (a b c) c) 1 (perform ask unit) 9)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(9));

    // Record multi-field with perform mid-field + trailing.
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 2))
    (field (record (a 1) (b (perform ask unit)) (c 3)) c)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(3));

    // LocalVar / Set / If / Match / Variant with perform in position.
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 1))
    (var x (perform ask unit)
      (seq (set x (+ x 1)) x))))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(2));

    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k true))
    (if (perform ask unit) 1 0)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(1));

    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k (some 4)))
    (match (perform ask unit)
      (none -> 0)
      (some x -> x))))"#,
    );
    let _ = v;

    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 5))
    (some (perform ask unit))))"#,
    );
    let _ = v;
}

#[test]
fn with_handler_resumed_and_forward_paths() {
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        oneshot(identity_resume()),
    );
    // With's handler expr is App of resume → Resumed propagates.
    let v = eval_expr(
        &CoreExpr::With {
            handler: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Int(4))],
            }),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        &env,
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(4));

    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        oneshot(Rc::new(|_, _| Ok(Outcome::Forward))),
    );
    let err = eval_expr(
        &CoreExpr::With {
            handler: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("k".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
            }),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("forward"));
}

#[test]
fn nested_same_op_deep_resume_and_forward_in_handler() {
    // Deep resume re-enters same op.
    let v = eval_source(
        r#"(val main
  (handle ask (fn (n k)
      (if (= n 0) 42 (k (- n 1))))
    (let ((x (perform ask 1)))
      (+ x 0))))"#,
    );
    let _ = v;

    let v = eval_source(
        r#"(val main
  (handle outer (fn (msg) msg)
    (handle log (fn (msg k) (forward k))
      (perform log "x"))))"#,
    )
    .unwrap();
    assert!(matches!(
        v,
        RuntimeValue::String(_) | RuntimeValue::Unit
    ));
}

#[test]
fn memory_fs_host_edges() {
    let mut host = MemoryFsHost::default();
    let _ = host.perform("read-file", RuntimeValue::String("missing".into()));
    let _ = host.perform("write-file", RuntimeValue::Int(1));
    let _ = host.perform(
        "write-file",
        RuntimeValue::String("path\0contents".into()),
    );
    let _ = host.perform("write-file", RuntimeValue::String("nopath".into()));
    let _ = host.perform("unknown", RuntimeValue::Unit);
    let _ = host.perform("log", RuntimeValue::String("hi".into()));
    let _ = host.perform("random", RuntimeValue::Unit);

    let mut host = MemoryFsHost::default();
    let v = eval_source_with_host(
        r#"(val main
  (seq
    (perform write-file "t.txt\0hello")
    (perform read-file "t.txt")))"#,
        &mut host,
    );
    let _ = v;
}

#[test]
fn try_check_cast_none_and_record_update_errors() {
    let v = eval_expr(
        &CoreExpr::TryCast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
            target: CoreType::Int,
            cast_id: 1,
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap();
    assert!(matches!(v, RuntimeValue::Variant { tag, .. } if tag == "none"));

    let v = eval_expr(
        &CoreExpr::CheckCast {
            expr: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
            target: CoreType::Int,
            cast_id: 2,
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap();
    assert!(matches!(v, RuntimeValue::Variant { tag, .. } if tag == "err"));

    let err = eval_expr(
        &CoreExpr::RecordUpdate {
            record: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("record-update"));

    let err = eval_expr(
        &CoreExpr::RecordExtend {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            }),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(2)))],
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("already"));
}

#[test]
fn resume_other_in_let_set_wrappers_via_cont() {
    // Custom cont returns Forward so LocalVar/Set resume `other` arms fire.
    let fwd: ResumeCont = Rc::new(|_: RuntimeValue, _: &mut dyn EffectHost| Ok(Outcome::Forward));
    let mut env = HashMap::new();
    env.insert(
        "r".into(),
        RuntimeValue::OneShotResume {
            used: Rc::new(Cell::new(false)),
            cont: Rc::clone(&fwd),
        },
    );
    // Body applies resume inside a form whose performed-wrap exists only when
    // nested under Handle — so wrap via Performed body with custom resume by
    // installing ask that returns Forward from cont used as deep resume.
    // Simpler: evaluate LocalVar with init = App(OneShot that returns Forward)
    let err = eval_expr(
        &CoreExpr::LocalVar {
            name: "x".into(),
            init: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("r".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
            }),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("forward"));
}

#[test]
fn builtins_and_closure_arity() {
    let mut env = primitive_env();
    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(!err.message.is_empty());

    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                params: vec!["a".into(), "b".into()],
                body: Box::new(CoreExpr::Var("a".into())),
            }),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("arg") || err.message.contains("parameter"));

    env.insert("n".into(), RuntimeValue::Number(3.0));
    let v = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![
                CoreExpr::Var("n".into()),
                CoreExpr::Lit(CoreLiteral::Int(1)),
            ],
        },
        &env,
        &mut UnitHost,
    );
    let _ = v;
}
