//! Parallel compile job graph tests.

use reciplexa_opt::{
    CompileJob, CompileJobGraph, Fingerprint, JobId, JobKind, JobStatus, ParallelSchedule,
};

#[test]
fn ready_when_deps_done() {
    let mut g = CompileJobGraph::new();
    g.insert(CompileJob {
        id: JobId(1),
        kind: JobKind::TypeCheck,
        input: Fingerprint::of(&"a"),
        deps: vec![],
        status: JobStatus::Pending,
    });
    g.insert(CompileJob {
        id: JobId(2),
        kind: JobKind::LowerIr,
        input: Fingerprint::of(&"b"),
        deps: vec![JobId(1)],
        status: JobStatus::Pending,
    });
    assert_eq!(g.ready_jobs(), vec![JobId(1)]);
    assert!(g.mark(JobId(1), JobStatus::Done));
    assert_eq!(g.ready_jobs(), vec![JobId(2)]);
}

#[test]
fn schedule_waves_are_deterministic() {
    let mut g = CompileJobGraph::new();
    for (id, kind, deps) in [
        (1u64, JobKind::TypeCheck, vec![]),
        (2, JobKind::Layout, vec![]),
        (3, JobKind::Render, vec![JobId(1), JobId(2)]),
    ] {
        g.insert(CompileJob {
            id: JobId(id),
            kind,
            input: Fingerprint::of(&id),
            deps,
            status: JobStatus::Pending,
        });
    }
    let sched = ParallelSchedule::from_graph(&g);
    assert_eq!(sched.wave_count(), 2);
    assert_eq!(sched.waves[0], vec![JobId(1), JobId(2)]);
    assert_eq!(sched.waves[1], vec![JobId(3)]);
}

#[test]
fn mark_missing_and_getters() {
    let mut g = CompileJobGraph::new();
    assert!(g.is_empty());
    assert!(!g.mark(JobId(99), JobStatus::Failed));
    g.insert(CompileJob {
        id: JobId(5),
        kind: JobKind::NativePrepare,
        input: Fingerprint::zero(),
        deps: vec![JobId(99)],
        status: JobStatus::Pending,
    });
    assert_eq!(g.len(), 1);
    assert!(g.get(JobId(5)).is_some());
    assert!(g.ready_jobs().is_empty());
    g.get_mut(JobId(5)).unwrap().status = JobStatus::Running;
    assert_eq!(g.get(JobId(5)).unwrap().status, JobStatus::Running);
}
