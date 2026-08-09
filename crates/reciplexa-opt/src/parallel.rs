//! Parallel compile job graph stub (Phase 13).

use std::collections::{BTreeMap, BTreeSet};

use crate::fingerprint::Fingerprint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JobId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JobKind {
    TypeCheck,
    LowerIr,
    Layout,
    Render,
    NativePrepare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JobStatus {
    Pending,
    Ready,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileJob {
    pub id: JobId,
    pub kind: JobKind,
    pub input: Fingerprint,
    pub deps: Vec<JobId>,
    pub status: JobStatus,
}

/// Deterministic DAG of compile jobs (no real parallelism yet).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompileJobGraph {
    jobs: BTreeMap<JobId, CompileJob>,
}

impl CompileJobGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, job: CompileJob) {
        self.jobs.insert(job.id, job);
    }

    pub fn get(&self, id: JobId) -> Option<&CompileJob> {
        self.jobs.get(&id)
    }

    pub fn get_mut(&mut self, id: JobId) -> Option<&mut CompileJob> {
        self.jobs.get_mut(&id)
    }

    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    /// Jobs whose deps are all `Done`.
    pub fn ready_jobs(&self) -> Vec<JobId> {
        let mut out = Vec::new();
        for (id, job) in &self.jobs {
            if job.status != JobStatus::Pending && job.status != JobStatus::Ready {
                continue;
            }
            let deps_done = job.deps.iter().all(|d| {
                self.jobs
                    .get(d)
                    .map(|j| j.status == JobStatus::Done)
                    .unwrap_or(false)
            });
            if deps_done {
                out.push(*id);
            }
        }
        out
    }

    pub fn mark(&mut self, id: JobId, status: JobStatus) -> bool {
        if let Some(j) = self.jobs.get_mut(&id) {
            j.status = status;
            true
        } else {
            false
        }
    }
}

/// Suggested wave schedule: successive ready sets (stub for a thread pool).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParallelSchedule {
    pub waves: Vec<Vec<JobId>>,
}

impl ParallelSchedule {
    pub fn from_graph(graph: &CompileJobGraph) -> Self {
        let mut sim = graph.clone();
        let mut waves = Vec::new();
        let mut seen = BTreeSet::new();
        loop {
            let ready: Vec<JobId> = sim
                .ready_jobs()
                .into_iter()
                .filter(|id| !seen.contains(id))
                .collect();
            if ready.is_empty() {
                break;
            }
            for id in &ready {
                seen.insert(*id);
                sim.mark(*id, JobStatus::Done);
            }
            waves.push(ready);
        }
        Self { waves }
    }

    pub fn wave_count(&self) -> usize {
        self.waves.len()
    }
}
