//! Structured concurrency runtime (Phase 5 §7.3).

#![forbid(unsafe_code)]

pub mod cancel;
pub mod scheduler;
pub mod scope;
pub mod task;

pub use cancel::{CancellationToken, CancellationTokenSource};
pub use scheduler::{TestScheduler, Tick};
pub use scope::{RootScope, ScopeError, TaskScope};
pub use task::{JoinHandle, SpawnPolicy, TaskId, TaskOutcome, TaskState};
