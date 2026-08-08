//! Typed Core IR — Phase 2 lowering target.

#![forbid(unsafe_code)]

pub mod check;
pub mod expr;
pub mod lower;
pub mod ty;
pub mod unify;

pub use check::{infer_expr, typecheck_value, CheckError, TypeEnv};
pub use expr::{CoreExpr, CoreLiteral, CoreValue, MatchArm};
pub use lower::{lower_surface_form, LowerError, LoweredForm};
pub use ty::{CoreType, EffectRow, TypeVarId};
pub use unify::{unify, Subst, UnifyError};
