//! Reference evaluator — left-to-right, no optimizations.

#![forbid(unsafe_code)]

pub mod control;
pub mod eval;
pub mod graphics_value;
pub mod value;

pub use control::{EffectHost, EvalError, EvalResult, MemoryFsHost, UnitHost};
pub use eval::{eval_expr, eval_source, eval_source_with_host, primitive_env};
pub use graphics_value::{
    color_from_graphics_value, document_from_graphics_value, page_from_graphics_value,
    shape_from_graphics_value, GraphicsValueError,
};
pub use value::{BuiltinOp, RuntimeValue};
