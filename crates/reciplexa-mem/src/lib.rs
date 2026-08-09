//! Perceus ownership, reference counting, and reuse (Phase 6).

#![forbid(unsafe_code)]

pub mod conservative;
pub mod equiv;
pub mod exec;
pub mod ir;
pub mod linear;
pub mod lower;
pub mod perceus;
pub mod reg;
pub mod reuse;
pub mod seal;
pub mod state;
pub mod trace;
pub mod verify;

pub use conservative::conservative_rc;
pub use equiv::{
    assert_observational_equiv, check_observational_equiv, compile_and_run,
    compile_and_run_conservative, compile_and_run_no_reuse, observably_equal, EquivError,
};
pub use exec::{exec_linear, ExecError};
pub use ir::{MemInstr, MemLiteral, OwningProgram};
pub use linear::LinearProgram;
pub use lower::lower_core_linear;
pub use perceus::perceus_pass;
pub use reg::{Reg, RegAlloc};
pub use reuse::reuse_pass;
pub use seal::seal_before_return;
pub use state::{OwnMap, OwnState};
pub use trace::RcTrace;
pub use verify::{verify_ownership, verify_reuse, VerifyError};
