//! Region-coverage batch: deep resume continuations, cast/check, builtins, value surfaces.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::elaborate_source;
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::control::{identity_resume, EffectHost, EvalError, MemoryFsHost, UnitHost};
use reciplexa_eval::eval::{eval_expr, eval_source, eval_source_with_host};
use reciplexa_eval::value::{BuiltinOp, RuntimeValue};

#[test]
fn deep_resume_through_if_cond() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k true))
    (if (perform ask 0) 1 2)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(1));
}

#[test]
fn deep_resume_through_if_false_branch() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k false))
    (if (perform ask 0) 1 2)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(2));
}

#[test]
fn deep_resume_through_local_var_init() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 5))
    (var x (perform ask 0) x)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(5));
}

#[test]
fn deep_resume_through_set() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 9))
    (var x 0
      (seq (set x (perform ask 0)) x))))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(9));
}

#[test]
fn deep_resume_through_record_field_and_get() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 4))
    (field (record (a (perform ask 0)) (b 2)) a)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(4));
}

#[test]
fn deep_resume_through_record_get_on_performed_record() {
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k (record (a 7))))
    (field (perform ask 0) a)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(7));
}

#[test]
fn deep_resume_through_record_update_and_extend() {
    let updated = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k (record (a 1) (b 2))))
    (field
      (record-update (perform ask 0) (a 3))
      a)))"#,
    )
    .unwrap();
    assert_eq!(updated, RuntimeValue::Int(3));

    let extended = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k (record (a 1))))
    (field
      (record-extend (perform ask 0) (b 8))
      b)))"#,
    )
    .unwrap();
    assert_eq!(extended, RuntimeValue::Int(8));

    // Field-side perform in update/extend bubbles the resumed value (no re-wrap).
    let bubbled = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 3))
    (record-update (record (a 1)) (a (perform ask 0)))))"#,
    )
    .unwrap();
    assert_eq!(bubbled, RuntimeValue::Int(3));
}

#[test]
fn deep_resume_through_app_fun_and_arg() {
    let via_arg = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k 11))
    ((fn (x) x) (perform ask 0))))"#,
    )
    .unwrap();
    assert_eq!(via_arg, RuntimeValue::Int(11));

    let via_fun = eval_source(
        r#"(val main
  (handle ask (fn (_ k) (k (fn (x) x)))
    ((perform ask 0) 12)))"#,
    )
    .unwrap();
    assert_eq!(via_fun, RuntimeValue::Int(12));
}

#[test]
fn deep_resume_through_variant_and_match() {
    let v = eval_source(
        r#"
(data option (none) (some x))
(val main
  (handle ask (fn (_ k) (k 13))
    (match (some (perform ask 0))
      (none -> 0)
      (some x -> x))))
"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(13));
}

#[test]
fn deep_resume_through_match_scrutinee() {
    let v = eval_source(
        r#"
(data option (none) (some x))
(val main
  (handle ask (fn (_ k) (k (some 14)))
    (match (perform ask 0)
      (none -> 0)
      (some x -> x))))
"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(14));
}

#[test]
fn check_cast_ok_and_err_runtime() {
    let ok = eval_source("(val main (check-cast 1 int))").unwrap();
    assert!(matches!(
        ok,
        RuntimeValue::Variant {
            tag,
            payload: Some(inner)
        } if tag == "ok" && matches!(inner.as_ref(), RuntimeValue::Int(1))
    ));
    let err = eval_source(r#"(val main (check-cast "x" int))"#).unwrap();
    assert!(matches!(
        err,
        RuntimeValue::Variant {
            tag,
            payload: Some(inner)
        } if tag == "err" && matches!(inner.as_ref(), RuntimeValue::String(_))
    ));
}

#[test]
fn cast_tag_check_fail_and_identity() {
    let fail = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
        evidence: CastEvidence::TagCheck { tag: "int".into() },
        target: CoreType::Int,
        cast_id: 0,
    };
    let err = eval_expr(&fail, &HashMap::new(), &mut UnitHost).unwrap_err();
    assert!(err.message.contains("cast failed"));

    let ok = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(3))),
        evidence: CastEvidence::Identity,
        target: CoreType::Int,
        cast_id: 1,
    };
    assert_eq!(
        eval_expr(&ok, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Int(3)
    );
}

#[test]
fn cast_union_variant_record_function_and_compose() {
    let union_ok = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        evidence: CastEvidence::UnionCheck {
            members: vec![CoreType::Int, CoreType::String],
        },
        target: CoreType::Union(vec![CoreType::Int, CoreType::String]),
        cast_id: 0,
    };
    assert_eq!(
        eval_expr(&union_ok, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Int(1)
    );

    let var_ok = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Int(2)))),
        }),
        evidence: CastEvidence::VariantCheck {
            variants: vec![("some".into(), Some(CoreType::Int))],
        },
        target: CoreType::Variant {
            variants: vec![("some".into(), Some(CoreType::Int))],
        },
        cast_id: 1,
    };
    let v = eval_expr(&var_ok, &HashMap::new(), &mut UnitHost).unwrap();
    assert!(matches!(v, RuntimeValue::Variant { tag, .. } if tag == "some"));

    let rec_ok = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Record {
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
        }),
        evidence: CastEvidence::RecordCheck {
            fields: vec![("a".into(), CoreType::Int)],
        },
        target: CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        },
        cast_id: 2,
    };
    assert!(matches!(
        eval_expr(&rec_ok, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Record(_)
    ));

    let fun_ok = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Lambda {
            params: vec!["x".into()],
            body: Box::new(CoreExpr::Var("x".into())),
        }),
        evidence: CastEvidence::FunctionGuard {
            arity: 1,
            arg_casts: vec![CastEvidence::Identity],
            ret_cast: Box::new(CastEvidence::Identity),
        },
        target: CoreType::dyn_any(),
        cast_id: 3,
    };
    assert!(matches!(
        eval_expr(&fun_ok, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Closure { .. }
    ));

    let compose = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(4))),
        evidence: CastEvidence::Compose(vec![CastEvidence::Identity, CastEvidence::NumericPromote]),
        target: CoreType::F64,
        cast_id: 4,
    };
    assert_eq!(
        eval_expr(&compose, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::F64(4.0)
    );

    let nominal = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Variant {
            tag: "ok".into(),
            payload: None,
        }),
        evidence: CastEvidence::NominalCheck { name: "ok".into() },
        target: CoreType::dyn_any(),
        cast_id: 5,
    };
    assert!(matches!(
        eval_expr(&nominal, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Variant { tag, .. } if tag == "ok"
    ));

    let inter = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        evidence: CastEvidence::IntersectionCheck {
            members: vec![CoreType::Int, CoreType::Number],
        },
        target: CoreType::Int,
        cast_id: 6,
    };
    assert_eq!(
        eval_expr(&inter, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Int(1)
    );
}

#[test]
fn float_ops_and_predicates_and_errors() {
    assert_eq!(
        eval_source("(val main (+ 1.5 2.5))").unwrap(),
        RuntimeValue::F64(4.0)
    );
    assert_eq!(
        eval_source("(val main (- 5.0 1.5))").unwrap(),
        RuntimeValue::F64(3.5)
    );
    assert_eq!(
        eval_source("(val main (* 2.0 3.0))").unwrap(),
        RuntimeValue::F64(6.0)
    );
    assert_eq!(
        eval_source("(val main (< 1.0 2.0))").unwrap(),
        RuntimeValue::Bool(true)
    );
    assert_eq!(
        eval_source("(val main (> 3.0 1.0))").unwrap(),
        RuntimeValue::Bool(true)
    );
    assert_eq!(
        eval_source("(val main (<= 2.0 2.0))").unwrap(),
        RuntimeValue::Bool(true)
    );
    assert_eq!(
        eval_source("(val main (>= 2.0 3.0))").unwrap(),
        RuntimeValue::Bool(false)
    );

    assert_eq!(
        eval_source("(val main (number? 1))").unwrap(),
        RuntimeValue::Bool(true)
    );
    assert_eq!(
        eval_source(r#"(val main (string? "a"))"#).unwrap(),
        RuntimeValue::Bool(true)
    );
    assert_eq!(
        eval_source("(val main (bool? true))").unwrap(),
        RuntimeValue::Bool(true)
    );
    assert_eq!(
        eval_source(
            r#"
(data option (none) (some x))
(val main (is-none none))
"#
        )
        .unwrap(),
        RuntimeValue::Bool(true)
    );
    assert_eq!(
        eval_source(
            r#"
(data option (none) (some x))
(val main (is-some (some 1)))
"#
        )
        .unwrap(),
        RuntimeValue::Bool(true)
    );

    let err = eval_source("(val main (int-div 1 0))").unwrap_err();
    assert!(err.message.contains("zero"));
    let err = eval_source("(val main (mod 1 0))").unwrap_err();
    assert!(err.message.contains("zero"));
    let err = eval_source(r#"(val main (int-div 1 "x"))"#).unwrap_err();
    assert!(err.message.contains("int"));
}

#[test]
fn letrec_non_lambda_and_set_errors() {
    let err = eval_expr(
        &CoreExpr::LetRec {
            bindings: vec![("f".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("lambda"));

    let err = eval_expr(
        &CoreExpr::Set {
            name: "x".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("unbound"));

    let mut env = HashMap::new();
    env.insert("x".into(), RuntimeValue::Int(1));
    let err = eval_expr(
        &CoreExpr::Set {
            name: "x".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(2))),
        },
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("not a var cell"));
}

#[test]
fn with_non_handler_and_forward_misuse_and_handler_arity() {
    let err = eval_source(r#"(val main (with 1 (perform log "x")))"#).unwrap_err();
    assert!(err.message.contains("handler"));

    let err = eval_source(r#"(val main (forward k))"#).unwrap_err();
    assert!(
        err.message.contains("forward") || err.message.contains("resume"),
        "{}",
        err.message
    );

    let err =
        eval_source(r#"(val main (handle log (fn (a b c) a) (perform log "x")))"#).unwrap_err();
    assert!(err.message.contains("1 or 2") || err.message.contains("parameter"));
}

#[test]
fn record_update_extend_errors_and_match_lit() {
    let err = eval_source(r#"(val main (record-update 1 (a 2)))"#).unwrap_err();
    assert!(err.message.contains("record-update"));

    let err = eval_source(r#"(val main (record-update (record (a 1)) (b 2)))"#).unwrap_err();
    assert!(err.message.contains("not present"));

    let err = eval_source(r#"(val main (record-extend 1 (a 2)))"#).unwrap_err();
    assert!(err.message.contains("record-extend"));

    let err = eval_source(r#"(val main (record-extend (record (a 1)) (a 2)))"#).unwrap_err();
    assert!(err.message.contains("already present"));

    let lit_ok = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            arms: vec![
                MatchArm {
                    pattern: CorePattern::Lit(CoreLiteral::Int(0)),
                    body: CoreExpr::Lit(CoreLiteral::Int(9)),
                },
                MatchArm {
                    pattern: CorePattern::Lit(CoreLiteral::Int(1)),
                    body: CoreExpr::Lit(CoreLiteral::Int(2)),
                },
            ],
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(lit_ok, RuntimeValue::Int(2));

    let wild = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(7)),
            }],
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(wild, RuntimeValue::Int(7));

    let bind = eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::String("z".into()))),
            arms: vec![MatchArm {
                pattern: CorePattern::Bind("x".into()),
                body: CoreExpr::Var("x".into()),
            }],
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(bind, RuntimeValue::String("z".into()));
}

#[test]
fn memory_fs_host_error_paths_and_unit_failure() {
    let mut host = MemoryFsHost::default();
    let err = host.perform("read-file", RuntimeValue::Int(1)).unwrap_err();
    assert!(err.message.contains("string"));
    let err = host
        .perform("read-file", RuntimeValue::String("missing.txt".into()))
        .unwrap_err();
    assert!(err.message.contains("missing"));
    let err = host
        .perform("write-file", RuntimeValue::Int(1))
        .unwrap_err();
    assert!(err.message.contains("string"));
    let err = host
        .perform("write-file", RuntimeValue::String("nopath".into()))
        .unwrap_err();
    assert!(err.message.contains("path"));
    let err = host.perform("unknown", RuntimeValue::Unit).unwrap_err();
    assert!(err.message.contains("unknown"));
    assert!(host.perform("log", RuntimeValue::Unit).is_ok());
    assert_eq!(
        host.perform("random", RuntimeValue::Unit).unwrap(),
        RuntimeValue::F64(0.5)
    );

    let err = UnitHost
        .perform("failure", RuntimeValue::String("boom".into()))
        .unwrap_err();
    assert!(err.message.contains("unhandled failure"));

    let err = EvalError::unhandled_failure("x");
    assert!(err.message.contains("unhandled failure"));
}

#[test]
fn eval_error_core_placeholder_and_number_lit() {
    let err = eval_expr(&CoreExpr::Error, &HashMap::new(), &mut UnitHost).unwrap_err();
    assert!(err.message.contains("syntax-error"));

    let n = eval_expr(
        &CoreExpr::Lit(CoreLiteral::Number(1.25)),
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(n, RuntimeValue::Number(1.25));

    let f = eval_expr(
        &CoreExpr::Lit(CoreLiteral::F64(2.5)),
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap();
    assert_eq!(f, RuntimeValue::F64(2.5));
}

#[test]
fn value_debug_display_eq_ty_remaining_arms() {
    let number = RuntimeValue::Number(1.5);
    assert!(format!("{number:?}").contains("Number"));
    assert_eq!(number.to_string(), "1.5");
    assert_eq!(number.ty(), CoreType::F64);
    assert_eq!(number, RuntimeValue::Number(1.5));
    assert_ne!(number, RuntimeValue::F64(1.5));

    let f64v = RuntimeValue::F64(2.0);
    assert!(format!("{f64v:?}").contains("F64"));
    assert_eq!(f64v.to_string(), "2");

    let s = RuntimeValue::String("hi".into());
    assert!(format!("{s:?}").contains("String"));

    let b = RuntimeValue::Bool(false);
    assert!(format!("{b:?}").contains("Bool"));

    let shape = RuntimeValue::ShapeTag("circle".into());
    assert!(format!("{shape:?}").contains("ShapeTag"));

    let clo = RuntimeValue::Closure {
        params: vec!["x".into()],
        body: CoreExpr::Lit(CoreLiteral::Int(0)),
        env: Rc::new(std::cell::RefCell::new(HashMap::new())),
    };
    assert!(format!("{clo:?}").contains("Closure"));
    assert_eq!(
        clo,
        RuntimeValue::Closure {
            params: vec!["x".into()],
            body: CoreExpr::Lit(CoreLiteral::Int(0)),
            env: Rc::new(std::cell::RefCell::new(HashMap::new())),
        }
    );

    let alive = Rc::new(Cell::new(true));
    let cell = RuntimeValue::Cell {
        value: Rc::new(std::cell::RefCell::new(RuntimeValue::Int(3))),
        alive: Rc::clone(&alive),
    };
    assert!(format!("{cell:?}").contains("Cell"));
    assert_eq!(cell.to_string(), "cell(3)");
    assert_eq!(cell.ty(), CoreType::Int);
    alive.set(false);
    assert_eq!(cell.to_string(), "cell(dead)");
    assert!(format!("{cell:?}").contains("alive"));

    let used = Rc::new(Cell::new(false));
    let resume = RuntimeValue::OneShotResume {
        used: Rc::clone(&used),
        cont: identity_resume(),
    };
    assert!(format!("{resume:?}").contains("OneShotResume"));
    assert_eq!(resume.to_string(), "resume");
    assert!(matches!(resume.ty(), CoreType::Fun { .. }));
    assert_eq!(
        resume,
        RuntimeValue::OneShotResume {
            used: Rc::clone(&used),
            cont: identity_resume(),
        }
    );

    let handler = RuntimeValue::Handler {
        op: "log".into(),
        params: vec!["msg".into()],
        body: CoreExpr::Lit(CoreLiteral::Unit),
        env: Rc::new(std::cell::RefCell::new(HashMap::new())),
    };
    assert!(format!("{handler:?}").contains("Handler"));
    assert_eq!(handler.to_string(), "handler(log)");
    assert_eq!(handler.ty(), CoreType::dyn_any());
    assert_eq!(
        handler,
        RuntimeValue::Handler {
            op: "log".into(),
            params: vec!["msg".into()],
            body: CoreExpr::Lit(CoreLiteral::Unit),
            env: Rc::new(std::cell::RefCell::new(HashMap::new())),
        }
    );

    let builtin = RuntimeValue::Builtin(BuiltinOp::Add);
    assert!(format!("{builtin:?}").contains("Builtin"));
    assert!(builtin.to_string().contains("builtin"));
    assert!(matches!(builtin.ty(), CoreType::Fun { .. }));
    assert_eq!(builtin, RuntimeValue::Builtin(BuiltinOp::Add));

    let bytes = RuntimeValue::Bytes(vec![0xab, 0xcd]);
    assert!(format!("{bytes:?}").contains("Bytes"));
    assert_eq!(bytes.to_string(), "bytes(0xab 0xcd)");
    assert_eq!(bytes.ty(), CoreType::Bytes);
    assert_eq!(bytes, RuntimeValue::Bytes(vec![0xab, 0xcd]));
}

#[test]
fn resume_arity_and_double_use_errors() {
    use reciplexa_eval::control::identity_resume;
    use std::cell::Cell;
    use std::rc::Rc;

    let used = Rc::new(Cell::new(false));
    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        RuntimeValue::OneShotResume {
            used: Rc::clone(&used),
            cont: identity_resume(),
        },
    );
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
    assert!(
        err.message.contains("expects 1") || err.message.contains("resume"),
        "{}",
        err.message
    );

    used.set(true);
    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(
        err.message.contains("already used") || err.message.contains("one-shot"),
        "{}",
        err.message
    );
}

#[test]
fn eval_source_with_host_and_underscore_param() {
    let mut host = MemoryFsHost::default();
    host.files.insert("t.txt".into(), "ok".into());
    let v = eval_source_with_host(r#"(val main (perform read-file "t.txt"))"#, &mut host).unwrap();
    assert_eq!(v, RuntimeValue::String("ok".into()));

    let v = eval_source("(val main ((fn (_) 1) 99))").unwrap();
    assert_eq!(v, RuntimeValue::Int(1));
}

#[test]
fn elaborate_handle_forward_without_resume_binding_errors() {
    // Direct Core: Forward without OneShotResume in env.
    let err = eval_expr(
        &CoreExpr::Forward {
            resume_name: "k".into(),
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("forward") || err.message.contains("resume"));
}

#[test]
fn handler_value_forms_without_perform() {
    let expr = elaborate_source(r#"(val main (handler log (fn (msg) msg)))"#).unwrap();
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert!(matches!(v, RuntimeValue::Handler { op, .. } if op == "log"));
}

#[test]
fn match_literal_patterns_bool_unit_string_bytes() {
    for (lit, val) in [
        (CoreLiteral::Bool(true), RuntimeValue::Bool(true)),
        (CoreLiteral::Unit, RuntimeValue::Unit),
        (
            CoreLiteral::String("a".into()),
            RuntimeValue::String("a".into()),
        ),
        (CoreLiteral::Bytes(vec![1]), RuntimeValue::Bytes(vec![1])),
        (CoreLiteral::F64(1.0), RuntimeValue::F64(1.0)),
        (CoreLiteral::Number(2.0), RuntimeValue::Number(2.0)),
    ] {
        let v = eval_expr(
            &CoreExpr::Match {
                scrutinee: Box::new(CoreExpr::Lit(lit.clone())),
                arms: vec![MatchArm {
                    pattern: CorePattern::Lit(lit),
                    body: CoreExpr::Lit(CoreLiteral::Int(1)),
                }],
            },
            &HashMap::new(),
            &mut UnitHost,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::Int(1), "for {val:?}");
    }
}

#[test]
fn promote_legacy_number_via_numeric_cast() {
    let expr = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::Number(3.0))),
        evidence: CastEvidence::NumericPromote,
        target: CoreType::F64,
        cast_id: 0,
    };
    assert_eq!(
        eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::F64(3.0)
    );
}

#[test]
fn builtin_arity_errors_via_direct_apply() {
    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &reciplexa_eval::primitive_env(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("2 args") || err.message.contains("expects"));

    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("number?".into())),
            args: vec![],
        },
        &reciplexa_eval::primitive_env(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("1 arg") || err.message.contains("expects"));

    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("encode-utf8".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &reciplexa_eval::primitive_env(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("string"));

    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("decode-utf8".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &reciplexa_eval::primitive_env(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("bytes"));

    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("unicode".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::String("x".into()))],
        },
        &reciplexa_eval::primitive_env(),
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("numeric") || err.message.contains("unicode"));
}
