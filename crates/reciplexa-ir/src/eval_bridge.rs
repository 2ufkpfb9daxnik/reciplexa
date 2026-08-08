//! Bridge from reference evaluator to lowered effect IR.

use std::collections::HashMap;

use reciplexa_core::expr::CoreExpr;
use reciplexa_eval::{eval_expr, EffectHost, EvalResult, RuntimeValue};

use crate::handler::LoweredOp;
use crate::lower::LoweredProgram;

/// Effect host that records lowered operations instead of performing I/O.
#[derive(Debug, Clone, Default)]
pub struct IrEffectHost {
    pub program: LoweredProgram,
}

impl IrEffectHost {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EffectHost for IrEffectHost {
    fn perform(&mut self, op: &str, _arg: RuntimeValue) -> EvalResult {
        self.program
            .ops
            .push(LoweredOp::Perform { op: op.to_string() });
        Ok(RuntimeValue::Unit)
    }
}

/// Evaluate a Core expression while accumulating lowered effect ops.
pub fn eval_expr_to_ir(expr: &CoreExpr) -> (LoweredProgram, EvalResult) {
    let mut host = IrEffectHost::new();
    let result = eval_expr(expr, &HashMap::new(), &mut host);
    (host.program, result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_core::expr::{CoreExpr, CoreLiteral};

    #[test]
    fn perform_records_lowered_op() {
        let expr = CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
        };
        let (prog, result) = eval_expr_to_ir(&expr);
        assert!(result.is_ok());
        assert!(matches!(prog.ops.last(), Some(LoweredOp::Perform { op }) if op == "log"));
    }
}
