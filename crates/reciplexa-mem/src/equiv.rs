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
    // Well-formed CoreExpr lowers always verify and execute.
    verify_ownership(&sealed).expect("ownership after seal");
    verify_reuse(&sealed).expect("reuse after seal");
    let mut trace = RcTrace::default();
    Ok(exec_linear(&sealed, &mut trace).expect("exec after seal"))
}

/// Step A baseline: conservative RC without reuse specialization.
pub fn compile_and_run_conservative(expr: &CoreExpr) -> Result<RuntimeValue, EquivError> {
    let raw = lower_core_linear(expr);
    let rc = conservative_rc(&raw);
    let sealed = seal_before_return(rc);
    verify_ownership(&sealed).expect("ownership after conservative seal");
    let mut trace = RcTrace::default();
    Ok(exec_linear(&sealed, &mut trace).expect("exec after conservative seal"))
}

/// Perceus pipeline without reuse specialization (MEM-05 fallback baseline).
pub fn compile_and_run_no_reuse(expr: &CoreExpr) -> Result<RuntimeValue, EquivError> {
    let raw = lower_core_linear(expr);
    let opt = perceus_pass(&raw);
    let sealed = seal_before_return(opt);
    verify_ownership(&sealed).expect("ownership after perceus seal");
    let mut trace = RcTrace::default();
    Ok(exec_linear(&sealed, &mut trace).expect("exec after perceus seal"))
}

/// Compare two observable results and map inequality to [`EquivError::Mismatch`].
pub fn check_observational_equiv(
    reference: &RuntimeValue,
    optimized: &RuntimeValue,
) -> Result<(), EquivError> {
    if observably_equal(reference, optimized) {
        Ok(())
    } else {
        Err(EquivError::Mismatch {
            reference: format!("{reference:?}"),
            optimized: format!("{optimized:?}"),
        })
    }
}

/// Assert reference and optimized evaluators agree on observable results.
pub fn assert_observational_equiv(expr: &CoreExpr) -> Result<(), EquivError> {
    let reference = eval_expr(expr, &HashMap::new(), &mut UnitHost)
        .map_err(|e| EquivError::ReferenceEval(e.message))?;
    // compile_and_run only panics on internal pipeline invariants for CoreExpr.
    let optimized = compile_and_run(expr).expect("mem pipeline");
    check_observational_equiv(&reference, &optimized)
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
                    (None, Some(_)) | (Some(_), None) => false,
                }
        }
        // Closures compared by tag only in Phase 6 slice
        (RuntimeValue::Closure { .. }, RuntimeValue::Closure { .. }) => true,
        (
            RuntimeValue::Unit
            | RuntimeValue::Number(_)
            | RuntimeValue::String(_)
            | RuntimeValue::ShapeTag(_)
            | RuntimeValue::Record(_)
            | RuntimeValue::Variant { .. }
            | RuntimeValue::Closure { .. },
            _,
        ) => false,
    }
}
