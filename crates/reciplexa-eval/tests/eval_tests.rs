//! Integration tests moved from src/eval.rs for region coverage.

use std::collections::HashMap;

use reciplexa_core::expr::{CoreExpr, CoreLiteral, MatchArm};
use reciplexa_eval::eval::*;
use reciplexa_eval::value::*;
use reciplexa_eval::UnitHost;

#[test]
fn seq_evaluates_left_to_right() {
    let expr = CoreExpr::Seq(vec![
        CoreExpr::Lit(CoreLiteral::Number(1.0)),
        CoreExpr::Lit(CoreLiteral::Number(2.0)),
    ]);
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(2.0));
}

#[test]
fn let_binds_in_body() {
    let expr = CoreExpr::Let {
        name: "x".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Number(3.0))),
        body: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
    };
    let _ = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
}

#[test]
fn lambda_application() {
    // (λx. x) 2
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Lambda {
            params: vec!["x".into()],
            body: Box::new(CoreExpr::Var("x".into())),
        }),
        args: vec![CoreExpr::Lit(CoreLiteral::Number(2.0))],
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(2.0));
}

#[test]
fn let_binds_var_in_body() {
    // let x = 1 in x
    let expr = CoreExpr::Let {
        name: "x".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        body: Box::new(CoreExpr::Var("x".into())),
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(1.0));
}

#[test]
fn unbound_var_errors() {
    let err = eval_expr(&CoreExpr::Var("x".into()), &HashMap::new(), &mut UnitHost).unwrap_err();
    assert!(err.message.contains("unbound variable"));
}

#[test]
fn if_then_else_left_to_right() {
    let then_expr = CoreExpr::If {
        cond: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
        then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
    };
    assert_eq!(
        eval_expr(&then_expr, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(1.0)
    );

    let else_expr = CoreExpr::If {
        cond: Box::new(CoreExpr::Lit(CoreLiteral::Bool(false))),
        then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
    };
    assert_eq!(
        eval_expr(&else_expr, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(2.0)
    );
}

#[test]
fn if_non_bool_cond_errors() {
    let expr = CoreExpr::If {
        cond: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
    };
    let err = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap_err();
    assert!(err.message.contains("Bool"));
}

#[test]
fn eval_bool_literal() {
    assert_eq!(
        eval_expr(
            &CoreExpr::Lit(CoreLiteral::Bool(true)),
            &HashMap::new(),
            &mut UnitHost
        )
        .unwrap(),
        RuntimeValue::Bool(true)
    );
}

#[test]
fn pattern_match_variant() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(42.0)))),
        }),
        arms: vec![MatchArm::variant(
            "some".into(),
            Some("n".into()),
            CoreExpr::Lit(CoreLiteral::Number(0.0)),
        )],
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(0.0));
}

#[test]
fn eval_perform_log() {
    let expr = CoreExpr::Perform {
        op: "log".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Unit);
}

#[test]
fn eval_perform_random() {
    let expr = CoreExpr::Perform {
        op: "random".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("".into()))),
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(0.5));
}

#[test]
fn eval_unknown_op_errors() {
    let expr = CoreExpr::Perform {
        op: "draw".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("".into()))),
    };
    assert!(eval_expr(&expr, &HashMap::new(), &mut UnitHost).is_err());
}

#[test]
fn eval_shape_literal_tags() {
    for tag in ["circle", "rect", "text"] {
        let expr = CoreExpr::Lit(CoreLiteral::String(tag.into()));
        let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
        assert_eq!(v, RuntimeValue::ShapeTag(tag.into()));
    }
    let expr = CoreExpr::Lit(CoreLiteral::String("other".into()));
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::String("other".into()));
}

#[test]
fn eval_app_non_closure_errors() {
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        args: vec![CoreExpr::Lit(CoreLiteral::Number(2.0))],
    };
    assert!(eval_expr(&expr, &HashMap::new(), &mut UnitHost).is_err());
}

#[test]
fn eval_record_and_get() {
    let expr = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Record {
            fields: vec![("k".into(), CoreExpr::Lit(CoreLiteral::Number(9.0)))],
        }),
        field: "k".into(),
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Number(9.0));
}

#[test]
fn eval_match_non_variant_errors() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        arms: vec![],
    };
    assert!(eval_expr(&expr, &HashMap::new(), &mut UnitHost).is_err());
}

#[test]
fn eval_empty_seq_is_unit() {
    let v = eval_expr(&CoreExpr::Seq(vec![]), &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(v, RuntimeValue::Unit);
}

#[test]
fn eval_record_get_errors() {
    let missing = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Record {
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Number(1.0)))],
        }),
        field: "b".into(),
    };
    assert!(eval_expr(&missing, &HashMap::new(), &mut UnitHost).is_err());

    let not_rec = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        field: "a".into(),
    };
    assert!(eval_expr(&not_rec, &HashMap::new(), &mut UnitHost).is_err());
}

#[test]
fn eval_match_no_arm_and_bind_payload() {
    let no_arm = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "none".into(),
            payload: None,
        }),
        arms: vec![MatchArm::variant(
            "some".into(),
            None,
            CoreExpr::Lit(CoreLiteral::Number(0.0)),
        )],
    };
    assert!(eval_expr(&no_arm, &HashMap::new(), &mut UnitHost).is_err());

    let with_bind = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(7.0)))),
        }),
        arms: vec![MatchArm::variant(
            "some".into(),
            Some("n".into()),
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
        )],
    };
    assert_eq!(
        eval_expr(&with_bind, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(1.0)
    );
}

#[test]
fn eval_color_literal_and_let_uses_binding() {
    let color = CoreExpr::Lit(CoreLiteral::Color("red".into()));
    assert_eq!(
        eval_expr(&color, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::String("red".into())
    );

    // Let binds value even if body ignores it — exercise insert path.
    let expr = CoreExpr::Let {
        name: "x".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Number(3.0))),
        body: Box::new(CoreExpr::Lit(CoreLiteral::Number(9.0))),
    };
    assert_eq!(
        eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(9.0)
    );
}

#[test]
fn eval_variant_without_payload() {
    let expr = CoreExpr::Variant {
        tag: "none".into(),
        payload: None,
    };
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert!(matches!(
        v,
        RuntimeValue::Variant {
            tag,
            payload: None
        } if tag == "none"
    ));
}

#[test]
fn eval_propagates_nested_errors() {
    let bad = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        field: "missing".into(),
    };
    let env = HashMap::new();
    let mut host = UnitHost;

    assert!(eval_expr(
        &CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(bad.clone()),
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::Seq(vec![CoreExpr::Lit(CoreLiteral::Number(1.0)), bad.clone(),]),
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::Let {
            name: "x".into(),
            value: Box::new(bad.clone()),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::Record {
            fields: vec![("a".into(), bad.clone())],
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(bad.clone())),
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::Match {
            scrutinee: Box::new(bad.clone()),
            arms: vec![],
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::App {
            fun: Box::new(bad.clone()),
            args: vec![CoreExpr::Lit(CoreLiteral::Number(1.0))],
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
            }),
            args: vec![bad.clone()],
        },
        &env,
        &mut host
    )
    .is_err());

    assert!(eval_expr(
        &CoreExpr::RecordGet {
            record: Box::new(bad),
            field: "x".into(),
        },
        &env,
        &mut host
    )
    .is_err());
}

#[test]
fn eval_match_without_bind() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "ok".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0)))),
        }),
        arms: vec![MatchArm::variant(
            "ok".into(),
            None,
            CoreExpr::Lit(CoreLiteral::Number(5.0)),
        )],
    };
    assert_eq!(
        eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(5.0)
    );
}

#[test]
fn eval_lambda_and_multi_field_record() {
    let closure = eval_expr(
        &CoreExpr::Lambda {
            params: vec!["x".into()],
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap();
    assert!(matches!(closure, RuntimeValue::Closure { .. }));

    let rec = eval_expr(
        &CoreExpr::Record {
            fields: vec![
                ("a".into(), CoreExpr::Lit(CoreLiteral::Number(1.0))),
                ("b".into(), CoreExpr::Lit(CoreLiteral::String("x".into()))),
            ],
        },
        &HashMap::new(),
        &mut UnitHost,
    )
    .unwrap();
    if let RuntimeValue::Record(fields) = rec {
        assert_eq!(fields.len(), 2);
    } else {
        panic!("expected record");
    }
}

#[test]
fn eval_match_bind_without_payload() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "none".into(),
            payload: None,
        }),
        arms: vec![MatchArm::variant(
            "none".into(),
            Some("x".into()),
            CoreExpr::Lit(CoreLiteral::Number(0.0)),
        )],
    };
    assert_eq!(
        eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(0.0)
    );
}

#[test]
fn eval_app_closure_chain() {
    // Nested unary apps still work; prefer n-ary (λ(x y). …) for multi-arg.
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Lambda {
                    params: vec!["y".into()],
                    body: Box::new(CoreExpr::Lit(CoreLiteral::Number(9.0))),
                }),
            }),
            args: vec![CoreExpr::Lit(CoreLiteral::Number(1.0))],
        }),
        args: vec![CoreExpr::Lit(CoreLiteral::Number(2.0))],
    };
    assert_eq!(
        eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(9.0)
    );
}

#[test]
fn eval_nary_lambda_application() {
    // (λ(x y). x) 1 2 — true n-ary params/args, not nested currying
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Lambda {
            params: vec!["x".into(), "y".into()],
            body: Box::new(CoreExpr::Var("x".into())),
        }),
        args: vec![
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
            CoreExpr::Lit(CoreLiteral::Number(2.0)),
        ],
    };
    assert_eq!(
        eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap(),
        RuntimeValue::Number(1.0)
    );
}

#[test]
fn eval_arity_mismatch_errors() {
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Lambda {
            params: vec!["x".into(), "y".into()],
            body: Box::new(CoreExpr::Var("x".into())),
        }),
        args: vec![CoreExpr::Lit(CoreLiteral::Number(1.0))],
    };
    let err = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap_err();
    assert!(err.message.contains("arity"));
}

#[test]
fn eval_source_identity_application() {
    let v = eval_source("(val main ((fn (x) x) 42))").unwrap();
    assert_eq!(v, RuntimeValue::Number(42.0));
}

#[test]
fn eval_source_if_true() {
    let v = eval_source("(val main (if true 1 2))").unwrap();
    assert_eq!(v, RuntimeValue::Number(1.0));
}

#[test]
fn eval_source_sequential_vals() {
    let v = eval_source("(val f (fn (x) x))\n(val main (f 42))").unwrap();
    assert_eq!(v, RuntimeValue::Number(42.0));
}

#[test]
fn eval_source_pure_fn_example() {
    let src = include_str!("../../../examples/pure_fn.rpx");
    let v = eval_source(src).unwrap();
    assert_eq!(v, RuntimeValue::Number(42.0));
}

#[test]
fn eval_source_parse_error() {
    let err = eval_source("(val main").unwrap_err();
    assert!(err.message.contains("parse error"));
}

#[test]
fn eval_source_call1_macro_expands_before_eval() {
    let src = "(macro call1 ($f $x) -> ($f $x))\n(val main (call1 (fn (x) x) 42))";
    let v = eval_source(src).unwrap();
    assert_eq!(v, RuntimeValue::Number(42.0));
}

#[test]
fn eval_source_data_match_some() {
    let src = r#"
(data option (none) (some x))
(val main (match (some 1) (none -> 0) (some x -> x)))
"#;
    let v = eval_source(src).unwrap();
    assert_eq!(v, RuntimeValue::Number(1.0));
}

#[test]
fn eval_source_unit_literal() {
    let v = eval_source("(val main unit)").unwrap();
    assert_eq!(v, RuntimeValue::Unit);
}

#[test]
fn eval_source_record_field() {
    let v = eval_source(
        r#"
(val report (record (title "Report") (page-count 10)))
(val main (field report title))
"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::String("Report".into()));
}

#[test]
fn eval_source_list_and_tuple() {
    let empty = eval_source("(val main (list))").unwrap();
    assert_eq!(
        empty,
        RuntimeValue::Variant {
            tag: "nil".into(),
            payload: None
        }
    );
    let tup = eval_source(r#"(val main (tuple 1 "a"))"#).unwrap();
    match tup {
        RuntimeValue::Record(fields) => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "0");
            assert_eq!(fields[0].1, RuntimeValue::Number(1.0));
            assert_eq!(fields[1].0, "1");
            assert_eq!(fields[1].1, RuntimeValue::String("a".into()));
        }
        other => panic!("expected Record, got {other:?}"),
    }
}

#[test]
fn eval_source_letrec_simple() {
    // Recurse once then return 7 (no numeric primitives required).
    let src = r#"
(val main
  (letrec ((f (fn (x) (if x (f false) 7))))
    (f true)))
"#;
    let v = eval_source(src).unwrap();
    assert_eq!(v, RuntimeValue::Number(7.0));
}

#[test]
fn eval_source_var_set() {
    let src = r#"
(val main
  (var count 0
    (set count 1)
    count))
"#;
    let v = eval_source(src).unwrap();
    assert_eq!(v, RuntimeValue::Number(1.0));
}

#[test]
fn eval_source_var_escape_fails_after_scope() {
    // Closure captures the cell; calling it after `var` exits must fail.
    let src = r#"
(val main
  (let ((get
          (var count 0
            (fn () count))))
    (get)))
"#;
    let err = eval_source(src).unwrap_err();
    assert!(
        err.message.contains("escaped") || err.message.contains("scope"),
        "{}",
        err.message
    );
}

#[test]
fn eval_source_ker_primitives() {
    let v = eval_source("(val main (+ 1 2))").unwrap();
    assert_eq!(v, RuntimeValue::Number(3.0));
    let v = eval_source("(val main (* 3 4))").unwrap();
    assert_eq!(v, RuntimeValue::Number(12.0));
    let v = eval_source("(val main (/ 8 2))").unwrap();
    assert_eq!(v, RuntimeValue::Number(4.0));
    let v = eval_source("(val main (< 1 2))").unwrap();
    assert_eq!(v, RuntimeValue::Bool(true));
    let v = eval_source("(val main (> 3 1))").unwrap();
    assert_eq!(v, RuntimeValue::Bool(true));
    let v = eval_source("(val main (<= 2 2))").unwrap();
    assert_eq!(v, RuntimeValue::Bool(true));
    let v = eval_source("(val main (>= 2 3))").unwrap();
    assert_eq!(v, RuntimeValue::Bool(false));
    let v = eval_source("(val main (= 2 2))").unwrap();
    assert_eq!(v, RuntimeValue::Bool(true));
    let v = eval_source("(val main (!= 1 2))").unwrap();
    assert_eq!(v, RuntimeValue::Bool(true));
}

#[test]
fn eval_rsc_memory_fs_host() {
    use reciplexa_core::elaborate_source;
    use reciplexa_eval::MemoryFsHost;
    let expr = elaborate_source(r#"(val main (perform read-file "a.txt"))"#).unwrap();
    let mut host = MemoryFsHost::default();
    host.files.insert("a.txt".into(), "hi".into());
    let v = eval_expr(&expr, &HashMap::new(), &mut host).unwrap();
    assert_eq!(v, RuntimeValue::String("hi".into()));

    let nul = '\0';
    let write_src = format!(
        "(val main (seq (perform write-file \"b.txt{nul}x\") (perform read-file \"b.txt\")))"
    );
    let write = elaborate_source(&write_src).unwrap();
    let mut host = MemoryFsHost::default();
    let v = eval_expr(&write, &HashMap::new(), &mut host).unwrap();
    assert_eq!(v, RuntimeValue::String("x".into()));
    assert_eq!(host.files.get("b.txt").map(String::as_str), Some("x"));
}

#[test]
fn eval_rsc_unhandled_residual_fails() {
    let err = eval_source(r#"(val main (perform read-file "missing.txt"))"#).unwrap_err();
    assert!(
        err.message.contains("unhandled residual") || err.message.contains("read-file"),
        "{}",
        err.message
    );
}
