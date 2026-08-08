//! Deterministic test scheduler.

use std::collections::VecDeque;

use crate::task::TaskId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tick(pub u64);

#[derive(Debug, Clone)]
pub struct TestScheduler {
    queue: VecDeque<TaskId>,
    tick: u64,
    log: Vec<TaskId>,
}

impl TestScheduler {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            tick: 0,
            log: Vec::new(),
        }
    }

    pub fn enqueue(&mut self, task: TaskId) {
        self.queue.push_back(task);
    }

    pub fn step(&mut self) -> Option<Tick> {
        if let Some(task) = self.queue.pop_front() {
            self.log.push(task);
            self.tick = self.tick.saturating_add(1);
            Some(Tick(self.tick))
        } else {
            None
        }
    }

    pub fn run_to_completion(&mut self) -> Vec<TaskId> {
        while self.step().is_some() {}
        self.log.clone()
    }

    pub fn schedule_log(&self) -> &[TaskId] {
        &self.log
    }
}

impl Default for TestScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_is_deterministic() {
        let mut a = TestScheduler::new();
        let mut b = TestScheduler::new();
        for id in [TaskId(1), TaskId(2), TaskId(3)] {
            a.enqueue(id);
            b.enqueue(id);
        }
        assert_eq!(a.run_to_completion(), b.run_to_completion());
    }

    #[test]
    fn default_scheduler_starts_empty() {
        let mut s = TestScheduler::default();
        assert!(s.step().is_none());
        assert!(s.schedule_log().is_empty());
    }

    #[test]
    fn step_returns_tick_and_logs_order() {
        let mut s = TestScheduler::new();
        s.enqueue(TaskId(7));
        s.enqueue(TaskId(8));
        let t1 = s.step().unwrap();
        assert_eq!(t1, Tick(1));
        let t2 = s.step().unwrap();
        assert_eq!(t2, Tick(2));
        assert_eq!(s.schedule_log(), &[TaskId(7), TaskId(8)]);
        assert!(s.step().is_none());
    }

    #[test]
    fn run_to_completion_on_empty_queue() {
        let mut s = TestScheduler::new();
        assert!(s.run_to_completion().is_empty());
    }
}
