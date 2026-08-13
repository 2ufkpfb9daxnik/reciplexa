//! Reference evaluator — left-to-right, no optimizations.

#![forbid(unsafe_code)]

pub mod control;
pub mod document_value;
pub mod eval;
pub mod graphics_value;
pub mod math_value;
pub mod value;

pub use control::{EffectHost, EvalError, EvalResult, MemoryFsHost, UnitHost};
pub use document_value::{document_from_doc_value, page_from_doc_value};
pub use eval::{eval_expr, eval_source, eval_source_with_host, primitive_env};
pub use graphics_value::{
    color_from_graphics_value, document_from_graphics_value, page_from_graphics_value,
    shape_from_graphics_value, GraphicsValueError,
};
pub use math_value::{
    estimate_math_box_from_value, math_atom_from_value, MathValueError,
};
pub use value::{BuiltinOp, RuntimeValue};
