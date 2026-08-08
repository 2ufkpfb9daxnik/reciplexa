//! Core → lowered IR.

use reciplexa_core::expr::CoreExpr;

use crate::handler::LoweredOp;

#[derive(Debug, Clone, Default)]
pub struct LoweredProgram {
    pub ops: Vec<LoweredOp>,
}

pub fn lower_perform(op: &str, _arg: &CoreExpr) -> LoweredProgram {
    LoweredProgram {
        ops: vec![LoweredOp::Perform { op: op.to_string() }],
    }
}

pub fn lower_expr(expr: &CoreExpr) -> LoweredProgram {
    match expr {
        CoreExpr::Perform { op, .. } => lower_perform(op, expr),
        CoreExpr::Lit(_) => LoweredProgram {
            ops: vec![LoweredOp::Return],
        },
        CoreExpr::Seq(items) => {
            let mut prog = LoweredProgram::default();
            for item in items {
                prog.ops.extend(lower_expr(item).ops);
            }
            prog
        }
        _ => LoweredProgram {
            ops: vec![LoweredOp::Return],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_core::expr::CoreExpr;
    use reciplexa_core::expr::CoreLiteral;

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
    fn literal_and_seq_lower_to_return() {
        let lit = lower_expr(&CoreExpr::Lit(CoreLiteral::Number(1.0)));
        assert_eq!(lit.ops, vec![LoweredOp::Return]);

        let seq = lower_expr(&CoreExpr::Seq(vec![
            CoreExpr::Perform {
                op: "log".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String("a".into()))),
            },
            CoreExpr::Lit(CoreLiteral::Number(2.0)),
        ]));
        assert_eq!(seq.ops.len(), 2);
        assert!(matches!(seq.ops[0], LoweredOp::Perform { .. }));
        assert!(matches!(seq.ops[1], LoweredOp::Return));
    }

    #[test]
    fn unsupported_expr_lowers_to_return() {
        let lambda = lower_expr(&CoreExpr::Lambda {
            param: "x".into(),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
        });
        assert_eq!(lambda.ops, vec![LoweredOp::Return]);
    }
}
