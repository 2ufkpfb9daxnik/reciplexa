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
