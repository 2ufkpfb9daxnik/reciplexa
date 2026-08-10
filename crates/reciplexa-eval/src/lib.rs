//! Reference evaluator — left-to-right, no optimizations.

#![forbid(unsafe_code)]

pub mod control;
pub mod eval;
pub mod value;

pub use control::{EffectHost, EvalError, EvalResult, UnitHost};
pub use eval::{eval_expr, eval_source};
pub use value::RuntimeValue;
