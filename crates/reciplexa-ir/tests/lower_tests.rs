use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_ir::{lower_expr, lower_perform, LoweredOp, LoweredProgram};

#[test]
fn perform_lowers_to_op() {
    let expr = CoreExpr::Perform {
        op: "log".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("hi".into()))),
    };
    let p = lower_expr(&expr);
    assert!(matches!(p.ops[0], LoweredOp::Perform { .. }));
}

#[test]
fn lower_perform_direct() {
    let arg = CoreExpr::Lit(CoreLiteral::String("hi".into()));
    let p = lower_perform("log", &arg);
    assert_eq!(p.ops.len(), 1);
    assert!(matches!(p.ops[0], LoweredOp::Perform { .. }));
}

#[test]
fn literal_and_seq_lower_to_return() {
    let lit = lower_expr(&CoreExpr::Lit(CoreLiteral::Int(1)));
    assert_eq!(lit.ops, vec![LoweredOp::Return]);

    let seq = lower_expr(&CoreExpr::Seq(vec![
        CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::String("a".into()))),
        },
        CoreExpr::Lit(CoreLiteral::Int(2)),
    ]));
    assert_eq!(seq.ops.len(), 2);
    assert!(matches!(seq.ops[0], LoweredOp::Perform { .. }));
    assert!(matches!(seq.ops[1], LoweredOp::Return));
}

#[test]
fn empty_seq_lowers_to_empty_program() {
    let seq = lower_expr(&CoreExpr::Seq(vec![]));
    assert!(seq.ops.is_empty());
    let _ = LoweredProgram::default();
}

#[test]
fn unsupported_expr_lowers_to_return() {
    let lambda = lower_expr(&CoreExpr::Lambda {
        params: vec!["x".into()],
        body: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
    });
    assert_eq!(lambda.ops, vec![LoweredOp::Return]);

    let app = lower_expr(&CoreExpr::App {
        fun: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        args: vec![CoreExpr::Lit(CoreLiteral::Int(1))],
    });
    assert_eq!(app.ops, vec![LoweredOp::Return]);
}
