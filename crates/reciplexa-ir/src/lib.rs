//! Lowered effect IR (Phase 5 §7.2).

#![forbid(unsafe_code)]

pub mod continuation;
pub mod handler;
pub mod lower;
pub mod verify;

pub use continuation::{Continuation, ContinuationId};
pub use handler::{HandlerFrame, HandlerKind, LoweredOp};
pub use lower::{lower_expr, lower_perform, LoweredProgram};
pub use verify::{verify_one_shot, VerifyError};
