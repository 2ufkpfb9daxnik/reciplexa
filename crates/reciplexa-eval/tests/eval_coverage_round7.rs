//! Round-7 eval: deep Cont re-perform, compound Cont other, host edges.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::CastEvidence;
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::CoreType;
use reciplexa_eval::control::{MemoryFsHost, Outcome, UnitHost};
use reciplexa_eval::eval::{eval_expr, eval_source, eval_source_with_host, primitive_env};
use reciplexa_eval::value::RuntimeValue;

fn performed_then_forward() -> CoreExpr {
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
fn deep_resume_reperforms_same_op() {
    let v = eval_source(
        r#"(val main
  (handle ping (fn (n k)
      (k (if (= n 0) (perform ping 1) n)))
    (perform ping 0)))"#,
    );
    let _ = v;

    let v = eval_source(
        r#"(val main
  (handle ping (fn (n k)
      (k n))
    (seq (perform ping 0) (perform ping 1))))"#,
    );
    let _ = v;
}

#[test]
fn cont_other_arms_compound_forms() {
    let forms = [
        CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(performed_then_forward()),
            body: Box::new(CoreExpr::Var("c".into())),
        },
        CoreExpr::Set {
            name: "cell".into(),
            value: Box::new(performed_then_forward()),
        },
        CoreExpr::Let {
            name: "x".into(),
            value: Box::new(performed_then_forward()),
            body: Box::new(CoreExpr::Var("x".into())),
        },
        CoreExpr::Match {
            scrutinee: Box::new(performed_then_forward()),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Int(0)),
            }],
        },
        CoreExpr::If {
            cond: Box::new(performed_then_forward()),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        },
        CoreExpr::Record {
            fields: vec![("a".into(), performed_then_forward())],
        },
        CoreExpr::RecordUpdate {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
            }),
            fields: vec![("a".into(), performed_then_forward())],
        },
        CoreExpr::RecordExtend {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Int(0)))],
            }),
            fields: vec![("b".into(), performed_then_forward())],
        },
        CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(performed_then_forward())),
        },
        CoreExpr::Cast {
            expr: Box::new(performed_then_forward()),
            evidence: CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 0,
        },
        CoreExpr::TryCast {
            expr: Box::new(performed_then_forward()),
            target: CoreType::Int,
            cast_id: 1,
        },
        CoreExpr::CheckCast {
            expr: Box::new(performed_then_forward()),
            target: CoreType::Int,
            cast_id: 2,
        },
        CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Var("x".into())),
            }),
            args: vec![performed_then_forward()],
        },
        CoreExpr::App {
            fun: Box::new(performed_then_forward()),
            args: vec![],
        },
        CoreExpr::With {
            handler: Box::new(performed_then_forward()),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        },
        CoreExpr::Seq(vec![
            performed_then_forward(),
            CoreExpr::Lit(CoreLiteral::Int(1)),
        ]),
        CoreExpr::RecordGet {
            record: Box::new(performed_then_forward()),
            field: "a".into(),
        },
    ];

    for form in forms {
        let mut env = HashMap::new();
        env.insert(
            "cell".into(),
            RuntimeValue::Cell {
                value: Rc::new(std::cell::RefCell::new(RuntimeValue::Int(0))),
                alive: Rc::new(Cell::new(true)),
            },
        );
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
fn builtins_host_fs_and_top_forward() {
    let err = eval_expr(&CoreExpr::Error, &HashMap::new(), &mut UnitHost);
    assert!(err.is_err());

    let mut host = MemoryFsHost::default();
    let _ = eval_source_with_host(
        r#"(val main
  (handle write-file (fn (p) p)
    (write-file "t.txt")))"#,
        &mut host,
    );
    let _ = eval_source_with_host(
        r#"(val main
  (handle read-file (fn (p) "data")
    (read-file "t.txt")))"#,
        &mut host,
    );

    let env = primitive_env();
    for src in [
        r#"(val main (int-div 7 2))"#,
        r#"(val main (mod 7 2))"#,
        r#"(val main (unicode 97))"#,
        r#"(val main (encode-utf8 "hi"))"#,
        r#"(val main (decode-utf8 (encode-utf8 "z")))"#,
        r#"(val main (number? 1))"#,
        r#"(val main (string? "x"))"#,
        r#"(val main (bool? false))"#,
        r#"(val main (var c 0 (seq (set c 1) c)))"#,
        r#"(val main (letrec ((f (fn (n) (if (= n 0) 0 (f (- n 1))))) (f 2)))"#,
        r#"(val main (as int 1))"#,
        r#"(val main (try-cast 1 int))"#,
        r#"(val main (check-cast 1 int))"#,
        r#"(val main (record-update (record (a 1)) (a 2)))"#,
        r#"(val main (record-extend (record (a 1)) (b 2)))"#,
        r#"(val main (field (record (a 1)) a))"#,
        r#"(val main (match (some 1) ((some x) -> x) (none -> 0)))"#,
    ] {
        let _ = eval_source(src);
    }
    let _ = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("number?".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
        },
        &env,
        &mut UnitHost,
    );

    let mut env = HashMap::new();
    env.insert(
        "k".into(),
        RuntimeValue::OneShotResume {
            used: Rc::new(Cell::new(false)),
            cont: Rc::new(|_, _| Ok(Outcome::Forward)),
        },
    );
    let err = eval_expr(
        &CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Lit(CoreLiteral::Unit)],
        },
        &env,
        &mut UnitHost,
    );
    assert!(err.is_err());
}
