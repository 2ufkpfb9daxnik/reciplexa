//! Typed Core IR — Phase 2 lowering target.

#![forbid(unsafe_code)]

pub mod check;
pub mod elaborate;
pub mod expr;
pub mod lower;
pub mod ty;
pub mod unify;

pub use check::{
    infer_expr, infer_with_effects, typecheck_language_source, typecheck_value, CheckError, TypeEnv,
};
pub use elaborate::{elaborate_source, elaborate_with_data, DataEnv, ElaborateError};
pub use expr::{first_unreachable_arm, CoreExpr, CoreLiteral, CorePattern, CoreValue, MatchArm};
pub use lower::{lower_surface_form, LowerError, LoweredForm};
pub use ty::{CoreType, EffectRow, TypeVarId};
pub use unify::{unify, Subst, UnifyError};
