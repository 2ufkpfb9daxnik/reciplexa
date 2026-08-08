//! Task identifiers and outcomes.

use reciplexa_outcome::cancellation::CancellationReport;
use reciplexa_outcome::defect::DefectReport;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Pending,
    Running,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnPolicy {
    FailFast,
    CollectAll,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskOutcome<T> {
    Completed(T),
    Cancelled(CancellationReport),
    Failed(String),
    Defected(Box<DefectReport>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JoinHandle {
    pub task_id: TaskId,
}
