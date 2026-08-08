//! Typed Core IR — Phase 2 lowering target.

#![forbid(unsafe_code)]

pub mod expr;
pub mod lower;
pub mod ty;

pub use expr::{CoreExpr, CoreLiteral, CoreValue};
pub use lower::{lower_surface_form, LowerError, LoweredForm};
pub use ty::{CoreType, EffectRow, TypeVarId};
