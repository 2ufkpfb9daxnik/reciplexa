//! Reference evaluator — left-to-right, no optimizations.

#![forbid(unsafe_code)]

pub mod control;
pub mod document_value;
pub mod domain_japanese_constructors;
pub mod domain_length_units;
pub mod domain_math_constructors;
pub mod domain_native;
pub mod domain_pure_constructors;
pub mod eval;
pub mod graphics_value;
pub mod math_value;
pub mod value;

pub use control::{EffectHost, EvalError, EvalResult, MemoryFsHost, UnitHost};
pub use document_value::{document_from_doc_value, layout_doc_page_to_scene, page_from_doc_value};
pub use domain_native::{
    call_domain_native, dn2_slot, qualified_export_key, ColorSrgbOp, DomainNativeOp,
    GraphicsColorOp, GraphicsPageOp, GraphicsShapesOp, JapaneseClassesOp, JapaneseKihonOp,
    JapaneseLinebreakOp, JapaneseMarkupOp, MathAccentsOp, MathAlignOp, MathAtomsOp, MathBigopsOp,
    MathCasesOp, MathDelimitersOp, MathFracOp, MathMatrixOp, MathScriptsOp, MathSqrtOp,
    MathStackOp,
};
pub use eval::{
    eval_expr, eval_expr_with_extra, eval_source, eval_source_with_host, primitive_env,
};
pub use graphics_value::{
    color_from_graphics_value, document_from_graphics_value, page_from_graphics_value,
    shape_from_graphics_value, GraphicsValueError,
};
pub use math_value::{
    estimate_math_box_from_value, estimate_math_box_from_value_with_style,
    estimate_style_from_value, layout_math_to_shapes, math_atom_from_value,
    scripts_attachment_offsets_from_value, MathValueError,
};
pub use value::{BuiltinOp, RuntimeValue};
