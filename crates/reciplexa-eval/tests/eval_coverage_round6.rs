//! Round-6: chip remaining eval Cont Forward arms + builtin unicode/bytes.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::control::{MemoryFsHost, Outcome, UnitHost};
use reciplexa_eval::eval::{eval_expr, eval_source, eval_source_with_host, primitive_env};
use reciplexa_eval::value::RuntimeValue;

fn performed_then_forward_app() -> CoreExpr {
    CoreExpr::Let {
        name: "tmp".into(),
        value: Box::new(CoreExpr::Perform {
            op: "ask".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        }),
        body: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        }),
    }
}

#[test]
fn forward_other_arms_through_compound_forms() {
    let forms = [
        CoreExpr::Set {
            name: "cell".into(),
            value: Box::new(performed_then_forward_app()),
        },
        CoreExpr::If {
            cond: Box::new(performed_then_forward_app()),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(2))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(3))),
        },
        CoreExpr::RecordGet {
            record: Box::new(performed_then_forward_app()),
            field: "a".into(),
        },
        CoreExpr::RecordUpdate {
            record: Box::new(performed_then_forward_app()),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
        },
        CoreExpr::RecordExtend {
            record: Box::new(performed_then_forward_app()),
            fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
        },
        CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(performed_then_forward_app())),
        },
        CoreExpr::Match {
            scrutinee: Box::new(performed_then_forward_app()),
            arms: vec![],
        },
        CoreExpr::Cast {
            expr: Box::new(performed_then_forward_app()),
            evidence: CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 0,
        },
        CoreExpr::TryCast {
            expr: Box::new(performed_then_forward_app()),
            target: CoreType::Int,
            cast_id: 1,
        },
        CoreExpr::CheckCast {
            expr: Box::new(performed_then_forward_app()),
            target: CoreType::Int,
            cast_id: 2,
        },
        CoreExpr::Seq(vec![
            performed_then_forward_app(),
            CoreExpr::Lit(CoreLiteral::Int(9)),
        ]),
        CoreExpr::Record {
            fields: vec![("a".into(), performed_then_forward_app())],
        },
        CoreExpr::App {
            fun: Box::new(performed_then_forward_app()),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(0))],
        },
        CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Var("x".into())),
            }),
            args: vec![performed_then_forward_app()],
        },
        CoreExpr::Let {
            name: "y".into(),
            value: Box::new(performed_then_forward_app()),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        CoreExpr::With {
            handler: Box::new(performed_then_forward_app()),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
    ];

    for form in forms {
        let mut env = HashMap::new();
        env.insert(
            "k".into(),
            RuntimeValue::OneShotResume {
                used: Rc::new(Cell::new(false)),
                cont: Rc::new(|_, _| Ok(Outcome::Forward)),
            },
        );
        let wrapped = CoreExpr::Handle {
            op: "ask".into(),
            handler_params: vec!["_".into(), "r".into()],
            handler_body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("r".into())),
                args: vec![CoreExpr::Lit(CoreLiteral::Int(0))],
            }),
            body: Box::new(form),
        };
        let _ = eval_expr(&wrapped, &env, &mut UnitHost);
    }
}

#[test]
fn builtins_unicode_encode_decode_and_predicates() {
    let env = primitive_env();
    for (src, _pat) in [
        (r#"(val main (unicode 65))"#, true),
        (r#"(val main (encode-utf8 "hi"))"#, true),
        (r#"(val main (decode-utf8 (encode-utf8 "hi")))"#, true),
        (r#"(val main (number? 1))"#, true),
        (r#"(val main (string? "x"))"#, true),
        (r#"(val main (bool? true))"#, true),
    ] {
        let _ = eval_source(src);
    }

    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("unicode".into())),
            args: vec![
                CoreExpr::Lit(CoreLiteral::Int(1)),
                CoreExpr::Lit(CoreLiteral::Int(2)),
            ],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("1 arg") || err.message.contains("expects"));

    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("unicode".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::String("x".into()))],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("numeric") || err.message.contains("unicode"));

    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("encode-utf8".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("string"));

    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("decode-utf8".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::String("x".into()))],
        },
        &env,
        &mut UnitHost,
    )
    .unwrap_err();
    assert!(err.message.contains("bytes"));

    // Invalid unicode scalar
    let _ = eval_source("(val main (unicode -1))");
    let _ = eval_source("(val main (unicode 0x110000))");
}

#[test]
fn parse_after_expand_and_macro_failure() {
    // Sources that may fail at expand or parse boundary.
    for src in ["(", ")", "(val", "\"unterminated", "(val main ())"] {
        let _ = eval_source(src);
    }
    let mut host = MemoryFsHost::default();
    let _ = eval_source_with_host("(val main (perform read-file 1))", &mut host);
}

#[test]
fn deep_resume_reenter_same_op_and_abort() {
    let v = eval_source(
        r#"(val main
  (handle tick (fn (n k)
      (if (= n 0)
          99
          (k (- n 1))))
    (let ((x (perform tick 2)))
      (+ x 0))))"#,
    );
    let _ = v;

    // Handler aborts via 1-param style leaving resume unused.
    let v = eval_source(
        r#"(val main
  (handle ask (fn (_) 7)
    (seq (perform ask unit) 1)))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::Int(7));
}
