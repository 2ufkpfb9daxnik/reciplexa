//! Typed Core IR ? Phase 2 lowering target.

#![forbid(unsafe_code)]

pub mod cast;
pub mod check;
pub mod elaborate;
pub mod expr;
pub mod lower;
pub mod ty;
pub mod unify;

pub use cast::{
    cast_success_type, classify_decide, compose_evidence, decide_subtype, is_runtime_checkable,
    is_subtype, judge_dynamic_use, plan_cast_evidence, simplify_evidence, types_disjoint,
    CastEvidence, CastProvenance, DecideResult, DynamicUseJudgment, TypeDiagClass,
};
pub use check::{
    coerce_to_static, infer_expr, infer_with_effects, insert_implicit_casts,
    language_kernel_type_env, typecheck_core_expr, typecheck_language_source, typecheck_value,
    CheckError, TypeEnv,
};
pub use elaborate::{elaborate_source, elaborate_with_data, DataEnv, ElaborateError};
pub use expr::{first_unreachable_arm, CoreExpr, CoreLiteral, CorePattern, CoreValue, MatchArm};
pub use lower::{lower_surface_form, LowerError, LoweredForm};
pub use ty::{CoreType, EffectRow, NumericClass, SingletonValue, TypeVarId, Variance};
pub use unify::{unify, Subst, UnifyError};
