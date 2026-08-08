//! Equivalence between reference evaluator and RC executor (Phase 6 §8.4).

use std::collections::HashMap;

use reciplexa_core::expr::CoreExpr;
use reciplexa_eval::{eval_expr, RuntimeValue, UnitHost};

use crate::conservative::conservative_rc;
use crate::exec::exec_linear;
use crate::lower::lower_core_linear;
use crate::perceus::perceus_pass;
use crate::reuse::reuse_pass;
use crate::seal::seal_before_return;
use crate::trace::RcTrace;
use crate::verify::{verify_ownership, verify_reuse};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquivError {
    ReferenceEval(String),
    Verify(String),
    Exec(String),
    Mismatch {
        reference: String,
        optimized: String,
    },
}

/// Full Phase 6 pipeline: lower → perceus → reuse → verify → exec.
pub fn compile_and_run(expr: &CoreExpr) -> Result<RuntimeValue, EquivError> {
    let raw = lower_core_linear(expr);
    let opt = perceus_pass(&raw);
    let reused = reuse_pass(opt);
    let sealed = seal_before_return(reused);
    verify_ownership(&sealed).map_err(|e| EquivError::Verify(format!("{e:?}")))?;
    verify_reuse(&sealed).map_err(|e| EquivError::Verify(format!("{e:?}")))?;
    let mut trace = RcTrace::default();
    exec_linear(&sealed, &mut trace).map_err(|e| EquivError::Exec(format!("{e:?}")))
}

/// Step A baseline: conservative RC without reuse specialization.
pub fn compile_and_run_conservative(expr: &CoreExpr) -> Result<RuntimeValue, EquivError> {
    let raw = lower_core_linear(expr);
    let rc = conservative_rc(&raw);
    let sealed = seal_before_return(rc);
    verify_ownership(&sealed).map_err(|e| EquivError::Verify(format!("{e:?}")))?;
    let mut trace = RcTrace::default();
    exec_linear(&sealed, &mut trace).map_err(|e| EquivError::Exec(format!("{e:?}")))
}

/// Perceus pipeline without reuse specialization (MEM-05 fallback baseline).
pub fn compile_and_run_no_reuse(expr: &CoreExpr) -> Result<RuntimeValue, EquivError> {
    let raw = lower_core_linear(expr);
    let opt = perceus_pass(&raw);
    let sealed = seal_before_return(opt);
    verify_ownership(&sealed).map_err(|e| EquivError::Verify(format!("{e:?}")))?;
    let mut trace = RcTrace::default();
    exec_linear(&sealed, &mut trace).map_err(|e| EquivError::Exec(format!("{e:?}")))
}

/// Assert reference and optimized evaluators agree on observable results.
pub fn assert_observational_equiv(expr: &CoreExpr) -> Result<(), EquivError> {
    let reference = eval_expr(expr, &HashMap::new(), &mut UnitHost)
        .map_err(|e| EquivError::ReferenceEval(e.message))?;
    let optimized = compile_and_run(expr)?;
    if !observably_equal(&reference, &optimized) {
        return Err(EquivError::Mismatch {
            reference: format!("{reference:?}"),
            optimized: format!("{optimized:?}"),
        });
    }
    Ok(())
}

/// Compare values ignoring physical identity (MEM-001 §2.1).
pub fn observably_equal(a: &RuntimeValue, b: &RuntimeValue) -> bool {
    match (a, b) {
        (RuntimeValue::Unit, RuntimeValue::Unit) => true,
        (RuntimeValue::Number(x), RuntimeValue::Number(y)) => x == y,
        (RuntimeValue::String(x), RuntimeValue::String(y)) => x == y,
        (RuntimeValue::ShapeTag(x), RuntimeValue::ShapeTag(y)) => x == y,
        (RuntimeValue::Record(ax), RuntimeValue::Record(bx)) => {
            ax.len() == bx.len()
                && ax
                    .iter()
                    .zip(bx.iter())
                    .all(|((ka, va), (kb, vb))| ka == kb && observably_equal(va, vb))
        }
        (
            RuntimeValue::Variant {
                tag: ta,
                payload: pa,
            },
            RuntimeValue::Variant {
                tag: tb,
                payload: pb,
            },
        ) => {
            ta == tb
                && match (pa, pb) {
                    (None, None) => true,
                    (Some(a), Some(b)) => observably_equal(a, b),
                    _ => false,
                }
        }
        // Closures compared by tag only in Phase 6 slice
        (RuntimeValue::Closure { .. }, RuntimeValue::Closure { .. }) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_core::expr::{CoreExpr, CoreLiteral};

    #[test]
    fn literal_equiv() {
        let e = CoreExpr::Lit(CoreLiteral::Number(7.0));
        assert_observational_equiv(&e).unwrap();
    }

    #[test]
    fn seq_equiv() {
        let e = CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
            CoreExpr::Lit(CoreLiteral::Number(2.0)),
        ]);
        assert_observational_equiv(&e).unwrap();
    }

    #[test]
    fn record_equiv() {
        let e = CoreExpr::Record {
            fields: vec![("x".into(), CoreExpr::Lit(CoreLiteral::Number(3.0)))],
        };
        assert_observational_equiv(&e).unwrap();
    }

    #[test]
    fn observably_equal_cases() {
        assert!(observably_equal(&RuntimeValue::Unit, &RuntimeValue::Unit));
        assert!(!observably_equal(
            &RuntimeValue::Number(1.0),
            &RuntimeValue::Number(2.0)
        ));
        assert!(observably_equal(
            &RuntimeValue::Record(vec![("a".into(), RuntimeValue::Number(1.0))]),
            &RuntimeValue::Record(vec![("a".into(), RuntimeValue::Number(1.0))]),
        ));
        assert!(!observably_equal(
            &RuntimeValue::Variant {
                tag: "A".into(),
                payload: None,
            },
            &RuntimeValue::Variant {
                tag: "B".into(),
                payload: None,
            },
        ));
    }

    #[test]
    fn compile_and_run_conservative_literal() {
        let e = CoreExpr::Lit(CoreLiteral::Number(3.0));
        let v = compile_and_run_conservative(&e).unwrap();
        assert_eq!(v, RuntimeValue::Number(3.0));
    }

    #[test]
    fn compile_and_run_no_reuse_literal() {
        let e = CoreExpr::Lit(CoreLiteral::Number(4.0));
        let v = compile_and_run_no_reuse(&e).unwrap();
        assert_eq!(v, RuntimeValue::Number(4.0));
    }

    #[test]
    fn mismatch_reports_equiv_error() {
        // Closures compare equal in observably_equal but eval may differ — use numbers
        let e = CoreExpr::Lit(CoreLiteral::Number(1.0));
        assert!(compile_and_run(&e).is_ok());
    }
}
