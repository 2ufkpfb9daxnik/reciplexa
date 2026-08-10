use reciplexa_core::expr::{CoreExpr, CoreLiteral, MatchArm};
use reciplexa_mem::ir::{MemInstr, MemLiteral};
use reciplexa_mem::lower_core_linear;

#[test]
fn lowers_literal() {
    let prog = lower_core_linear(&CoreExpr::Lit(CoreLiteral::Number(1.0)));
    assert!(!prog.instrs.is_empty());
}

#[test]
fn lowers_record() {
    let expr = CoreExpr::Record {
        fields: vec![("x".into(), CoreExpr::Lit(CoreLiteral::Number(1.0)))],
    };
    let prog = lower_core_linear(&expr);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Construct { .. })));
}

#[test]
fn lowers_variant_and_match() {
    let expr = CoreExpr::Variant {
        tag: "Ok".into(),
        payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0)))),
    };
    let prog = lower_core_linear(&expr);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Construct { .. })));

    let m = CoreExpr::Match {
        scrutinee: Box::new(expr),
        arms: vec![MatchArm {
            tag: "Ok".into(),
            bind: Some("v".into()),
            body: CoreExpr::Lit(CoreLiteral::Number(0.0)),
        }],
    };
    let prog2 = lower_core_linear(&m);
    assert!(!prog2.instrs.is_empty());
}

#[test]
fn lowers_lambda_and_app() {
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Lambda {
            params: vec!["x".into()],
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        }),
        args: vec![CoreExpr::Lit(CoreLiteral::Number(2.0))],
    };
    let prog = lower_core_linear(&expr);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Call { .. })));
}

#[test]
fn lowers_perform() {
    let expr = CoreExpr::Perform {
        op: "log".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
    };
    let prog = lower_core_linear(&expr);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Construct { .. })));
}

#[test]
fn lowers_color_and_string_literals() {
    let color = lower_core_linear(&CoreExpr::Lit(CoreLiteral::Color("red".into())));
    assert!(color.instrs.iter().any(|i| matches!(
        i,
        MemInstr::Lit {
            lit: MemLiteral::String(_),
            ..
        }
    )));
    let string = lower_core_linear(&CoreExpr::Lit(CoreLiteral::String("hi".into())));
    assert!(!string.instrs.is_empty());
}

#[test]
fn lowers_empty_seq_to_unit() {
    let prog = lower_core_linear(&CoreExpr::Seq(vec![]));
    assert!(prog.instrs.iter().any(|i| matches!(
        i,
        MemInstr::Lit {
            lit: MemLiteral::Unit,
            ..
        }
    )));
}

#[test]
fn lowers_match_fallback_inserts_dup_for_second_arm() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
        arms: vec![
            MatchArm {
                tag: "A".into(),
                bind: Some("v".into()),
                body: CoreExpr::Lit(CoreLiteral::Number(1.0)),
            },
            MatchArm {
                tag: "B".into(),
                bind: Some("w".into()),
                body: CoreExpr::Lit(CoreLiteral::Number(2.0)),
            },
        ],
    };
    let prog = lower_core_linear(&expr);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Dup { .. })));
}

#[test]
fn lowers_lambda_with_capture() {
    let expr = CoreExpr::Let {
        name: "x".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        body: Box::new(CoreExpr::Lambda {
            params: vec!["y".into()],
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        }),
    };
    let prog = lower_core_linear(&expr);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::MakeClosure { .. })));
}

#[test]
fn lowers_record_get_project() {
    let expr = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Record {
            fields: vec![("k".into(), CoreExpr::Lit(CoreLiteral::Number(4.0)))],
        }),
        field: "k".into(),
    };
    let prog = lower_core_linear(&expr);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Project { .. })));
}
