use reciplexa_core::expr::{CoreExpr, CoreLiteral, MatchArm};
use reciplexa_mem::ir::{MemInstr, MemLiteral};
use reciplexa_mem::lower_core_linear;

#[test]
fn lowers_literal() {
    let prog = lower_core_linear(&CoreExpr::Lit(CoreLiteral::Int(1)));
    assert!(!prog.instrs.is_empty());
}

#[test]
fn lowers_record() {
    let expr = CoreExpr::Record {
        fields: vec![("x".into(), CoreExpr::Lit(CoreLiteral::Int(1)))],
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
        payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Int(1)))),
    };
    let prog = lower_core_linear(&expr);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Construct { .. })));

    let m = CoreExpr::Match {
        scrutinee: Box::new(expr),
        arms: vec![MatchArm::variant(
            "Ok".into(),
            Some("v".into()),
            CoreExpr::Lit(CoreLiteral::Int(0)),
        )],
    };
    let prog2 = lower_core_linear(&m);
    assert!(!prog2.instrs.is_empty());
}

#[test]
fn lowers_lambda_and_app() {
    let expr = CoreExpr::App {
        fun: Box::new(CoreExpr::Lambda {
            params: vec!["x".into()],
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        }),
        args: vec![CoreExpr::Lit(CoreLiteral::Int(2))],
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
        scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        arms: vec![
            MatchArm::variant(
                "A".into(),
                Some("v".into()),
                CoreExpr::Lit(CoreLiteral::Int(1)),
            ),
            MatchArm::variant(
                "B".into(),
                Some("w".into()),
                CoreExpr::Lit(CoreLiteral::Int(2)),
            ),
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
        value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        body: Box::new(CoreExpr::Lambda {
            params: vec!["y".into()],
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
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
            fields: vec![("k".into(), CoreExpr::Lit(CoreLiteral::Int(4)))],
        }),
        field: "k".into(),
    };
    let prog = lower_core_linear(&expr);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Project { .. })));
}

#[test]
fn lowers_bytes_literal() {
    let expr = CoreExpr::Lit(CoreLiteral::Bytes(vec![1, 2, 3]));
    let prog = lower_core_linear(&expr);
    assert!(prog.instrs.iter().any(|i| matches!(
        i,
        MemInstr::Lit {
            lit: MemLiteral::Bytes(_),
            ..
        }
    )));
}

#[test]
fn lowers_if_with_select() {
    let expr = CoreExpr::If {
        cond: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
        then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
    };
    let prog = lower_core_linear(&expr);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Select { .. })));
}

#[test]
fn lowers_failure_perform_as_raise() {
    let expr = CoreExpr::Perform {
        op: "failure".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
    };
    let prog = lower_core_linear(&expr);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Raise { .. })));
}

#[test]
fn lowers_record_update_and_extend() {
    let base = CoreExpr::Record {
        fields: vec![
            (
                "title".into(),
                CoreExpr::Lit(CoreLiteral::String("Old".into())),
            ),
            ("n".into(), CoreExpr::Lit(CoreLiteral::Int(1))),
        ],
    };
    let update = CoreExpr::RecordUpdate {
        record: Box::new(base.clone()),
        fields: vec![(
            "title".into(),
            CoreExpr::Lit(CoreLiteral::String("New".into())),
        )],
    };
    let prog = lower_core_linear(&update);
    assert!(prog.instrs.iter().any(|i| matches!(
        i,
        MemInstr::Construct { tag, .. } if tag == "record-update"
    )));

    let extend = CoreExpr::RecordExtend {
        record: Box::new(CoreExpr::Record {
            fields: vec![(
                "title".into(),
                CoreExpr::Lit(CoreLiteral::String("T".into())),
            )],
        }),
        fields: vec![(
            "author".into(),
            CoreExpr::Lit(CoreLiteral::String("A".into())),
        )],
    };
    let prog2 = lower_core_linear(&extend);
    assert!(prog2.instrs.iter().any(|i| matches!(
        i,
        MemInstr::Construct { tag, .. } if tag == "record-extend"
    )));
}
