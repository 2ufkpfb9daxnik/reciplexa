//! Task scopes with structured cleanup.

use std::collections::HashSet;

use reciplexa_outcome::cancellation::{CancellationReason, CancellationReport};

use crate::cancel::CancellationToken;
use crate::task::{JoinHandle, SpawnPolicy, TaskId, TaskOutcome, TaskState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeError {
    ChildTasksRemain,
    ScopeCancelled,
}

#[derive(Debug, Clone)]
pub struct RootScope {
    next_task: u64,
    children: HashSet<TaskId>,
    cleanup_ran: bool,
}

impl RootScope {
    pub fn new() -> Self {
        Self {
            next_task: 1,
            children: HashSet::new(),
            cleanup_ran: false,
        }
    }

    pub fn spawn(&mut self, policy: SpawnPolicy) -> (TaskScope, JoinHandle) {
        let id = TaskId(self.next_task);
        self.next_task = self.next_task.saturating_add(1);
        self.children.insert(id);
        (
            TaskScope {
                parent: None,
                task_id: id,
                policy,
                state: TaskState::Pending,
                token: CancellationToken::NONE,
            },
            JoinHandle { task_id: id },
        )
    }

    pub fn child_finished(&mut self, id: TaskId) {
        self.children.remove(&id);
    }

    pub fn shutdown(&mut self) -> Result<(), ScopeError> {
        if !self.children.is_empty() {
            return Err(ScopeError::ChildTasksRemain);
        }
        self.cleanup_ran = true;
        Ok(())
    }

    pub fn cleanup_ran(&self) -> bool {
        self.cleanup_ran
    }
}

impl Default for RootScope {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TaskScope {
    pub parent: Option<TaskId>,
    pub task_id: TaskId,
    pub policy: SpawnPolicy,
    pub state: TaskState,
    pub token: CancellationToken,
}

impl TaskScope {
    pub fn cancel(&mut self) -> CancellationReport {
        self.state = TaskState::Cancelled;
        CancellationReport::new(1, CancellationReason::UserRequested)
    }

    pub fn complete<T>(&mut self, value: T) -> TaskOutcome<T> {
        self.state = TaskState::Completed;
        TaskOutcome::Completed(value)
    }

    pub fn await_child<T>(&self, outcome: TaskOutcome<T>) -> TaskOutcome<T> {
        match self.policy {
            SpawnPolicy::FailFast => outcome,
            SpawnPolicy::CollectAll => outcome,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_scope_rejects_shutdown_with_children() {
        let mut root = RootScope::new();
        let (_scope, handle) = root.spawn(SpawnPolicy::FailFast);
        assert!(root.shutdown().is_err());
        root.child_finished(handle.task_id);
        assert!(root.shutdown().is_ok());
        assert!(root.cleanup_ran());
    }
}
