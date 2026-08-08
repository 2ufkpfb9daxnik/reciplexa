//! Task scopes with structured cleanup.

use std::collections::HashSet;

use reciplexa_outcome::cancellation::{CancellationReason, CancellationReport, CleanupStatus};

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
    cleanup_hooks: Vec<String>,
}

impl RootScope {
    pub fn new() -> Self {
        Self {
            next_task: 1,
            children: HashSet::new(),
            cleanup_ran: false,
            cleanup_hooks: Vec::new(),
        }
    }

    pub fn on_cleanup(&mut self, label: impl Into<String>) {
        self.cleanup_hooks.push(label.into());
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

    /// Cancel all outstanding child tasks and mark cleanup complete.
    pub fn cancel_all_children(&mut self) -> CancellationReport {
        let remaining = self.children.len();
        self.children.clear();
        let mut report = CancellationReport::new(self.next_task, CancellationReason::HostShutdown);
        if remaining > 0 {
            report.cleanup_status = CleanupStatus::Completed;
        }
        report
    }

    /// Shut down after cancelling any remaining children (structured cleanup path).
    pub fn shutdown_with_cleanup(&mut self) -> CancellationReport {
        let report = self.cancel_all_children();
        self.cleanup_ran = true;
        report
    }

    pub fn shutdown(&mut self) -> Result<(), ScopeError> {
        if !self.children.is_empty() {
            return Err(ScopeError::ChildTasksRemain);
        }
        self.run_cleanup();
        Ok(())
    }

    pub fn cancel_all(
        &mut self,
        token: &mut crate::cancel::CancellationTokenSource,
    ) -> CancellationReport {
        token.cancel();
        self.run_cleanup();
        CancellationReport::new(1, CancellationReason::UserRequested)
    }

    fn run_cleanup(&mut self) {
        self.cleanup_ran = true;
        self.cleanup_hooks.clear();
    }

    pub fn cleanup_hook_count(&self) -> usize {
        self.cleanup_hooks.len()
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
        let mut report = CancellationReport::new(self.task_id.0, CancellationReason::UserRequested);
        report.cleanup_status = CleanupStatus::Completed;
        report
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
    use crate::cancel::CancellationTokenSource;

    #[test]
    fn root_scope_rejects_shutdown_with_children() {
        let mut root = RootScope::new();
        let (_scope, handle) = root.spawn(SpawnPolicy::FailFast);
        assert!(root.shutdown().is_err());
        root.child_finished(handle.task_id);
        assert!(root.shutdown().is_ok());
        assert!(root.cleanup_ran());
    }

    #[test]
    fn shutdown_with_cleanup_cancels_children() {
        let mut root = RootScope::new();
        let (_scope, _handle) = root.spawn(SpawnPolicy::FailFast);
        let report = root.shutdown_with_cleanup();
        assert_eq!(report.cleanup_status, CleanupStatus::Completed);
        assert!(root.cleanup_ran());
    }

    #[test]
    fn cancel_all_children_without_tasks() {
        let mut root = RootScope::new();
        let report = root.cancel_all_children();
        assert_ne!(report.cleanup_status, CleanupStatus::Completed);
        assert!(root.shutdown().is_ok());
    }

    #[test]
    fn cancel_all_runs_cleanup_hooks() {
        let mut root = RootScope::new();
        root.on_cleanup("disk");
        root.on_cleanup("gpu");
        assert_eq!(root.cleanup_hook_count(), 2);
        let mut token = CancellationTokenSource::new(9);
        let report = root.cancel_all(&mut token);
        assert!(token.is_cancelled());
        assert_eq!(report.reason, CancellationReason::UserRequested);
        assert_eq!(root.cleanup_hook_count(), 0);
        assert!(root.cleanup_ran());
    }

    #[test]
    fn spawn_increments_task_ids() {
        let mut root = RootScope::new();
        let (scope_a, _) = root.spawn(SpawnPolicy::CollectAll);
        let (scope_b, _) = root.spawn(SpawnPolicy::FailFast);
        assert_eq!(scope_a.task_id, TaskId(1));
        assert_eq!(scope_b.task_id, TaskId(2));
    }

    #[test]
    fn task_scope_cancel_and_complete() {
        let mut scope = TaskScope {
            parent: Some(TaskId(0)),
            task_id: TaskId(5),
            policy: SpawnPolicy::CollectAll,
            state: TaskState::Pending,
            token: CancellationToken::NONE,
        };
        let report = scope.cancel();
        assert_eq!(scope.state, TaskState::Cancelled);
        assert_eq!(report.cleanup_status, CleanupStatus::Completed);

        scope.state = TaskState::Pending;
        match scope.complete(42) {
            TaskOutcome::Completed(v) => assert_eq!(v, 42),
            _ => panic!("expected completed"),
        }
        assert_eq!(scope.state, TaskState::Completed);
    }

    #[test]
    fn await_child_passes_through_for_both_policies() {
        let scope_ff = TaskScope {
            parent: None,
            task_id: TaskId(1),
            policy: SpawnPolicy::FailFast,
            state: TaskState::Running,
            token: CancellationToken::NONE,
        };
        let scope_ca = TaskScope {
            parent: None,
            task_id: TaskId(2),
            policy: SpawnPolicy::CollectAll,
            state: TaskState::Running,
            token: CancellationToken::NONE,
        };
        assert_eq!(
            scope_ff.await_child(TaskOutcome::Completed("ok")),
            TaskOutcome::Completed("ok")
        );
        assert_eq!(
            scope_ca.await_child::<&str>(TaskOutcome::Failed("x".into())),
            TaskOutcome::Failed("x".into())
        );
    }

    #[test]
    fn root_scope_default_and_on_cleanup_before_shutdown() {
        let mut root = RootScope::default();
        root.on_cleanup("a");
        assert_eq!(root.cleanup_hook_count(), 1);
        assert!(root.shutdown().is_ok());
    }
}
