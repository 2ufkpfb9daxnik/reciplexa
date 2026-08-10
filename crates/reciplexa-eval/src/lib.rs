//! Reference evaluator — left-to-right, no optimizations.

#![forbid(unsafe_code)]

pub mod eval;
pub mod value;

pub use eval::{eval_expr, eval_source, EffectHost, EvalError, EvalResult, UnitHost};
pub use value::RuntimeValue;
