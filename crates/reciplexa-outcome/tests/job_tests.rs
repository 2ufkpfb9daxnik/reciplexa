use reciplexa_outcome::cancellation::{CancellationReason, CancellationReport};
use reciplexa_outcome::defect::{DefectCode, DefectReport, DefectScope};
use reciplexa_outcome::JobResult;

type J = JobResult<i32, &'static str>;

fn defect() -> DefectReport {
    DefectReport::new(
        1,
        DefectCode::new("test", "X"),
        "inv",
        DefectScope::Task,
        "sub",
        "msg",
    )
}

fn cancel() -> CancellationReport {
    CancellationReport::new(1, CancellationReason::Timeout)
}

/// Single monomorphization of `map` so every match arm is covered together.
fn map_j(job: J) -> J {
    job.map(|n| n + 1)
}

/// Single monomorphization of `map_err` so every match arm is covered together.
fn map_err_j(job: J) -> JobResult<i32, usize> {
    job.map_err(|e| e.len())
}

#[test]
fn cover_is_completed_and_is_defected() {
    let completed: J = JobResult::Completed(1);
    assert!(completed.is_completed());
    assert!(!completed.is_defected());

    let failed: J = JobResult::Failed("e");
    assert!(!failed.is_completed());
    assert!(!failed.is_defected());

    let cancelled: J = JobResult::Cancelled(cancel());
    assert!(!cancelled.is_completed());
    assert!(!cancelled.is_defected());

    let defected: J = JobResult::Defected(Box::new(defect()));
    assert!(!defected.is_completed());
    assert!(defected.is_defected());
}

#[test]
fn cover_all_map_arms_one_instantiation() {
    assert_eq!(map_j(JobResult::Completed(2)), JobResult::Completed(3));
    assert_eq!(map_j(JobResult::Failed("x")), JobResult::Failed("x"));
    assert!(matches!(
        map_j(JobResult::Cancelled(cancel())),
        JobResult::Cancelled(_)
    ));
    assert!(matches!(
        map_j(JobResult::Defected(Box::new(defect()))),
        JobResult::Defected(_)
    ));
}

#[test]
fn cover_all_map_err_arms_one_instantiation() {
    assert_eq!(map_err_j(JobResult::Completed(7)), JobResult::Completed(7));
    assert_eq!(map_err_j(JobResult::Failed("ab")), JobResult::Failed(2));
    assert!(matches!(
        map_err_j(JobResult::Cancelled(cancel())),
        JobResult::Cancelled(_)
    ));
    assert!(matches!(
        map_err_j(JobResult::Defected(Box::new(defect()))),
        JobResult::Defected(_)
    ));
}

#[test]
fn job_result_clone_eq_debug() {
    let variants: [J; 4] = [
        JobResult::Completed(1),
        JobResult::Failed("e"),
        JobResult::Cancelled(cancel()),
        JobResult::Defected(Box::new(defect())),
    ];
    for v in &variants {
        let cloned = v.clone();
        assert_eq!(v, &cloned);
        let _ = format!("{v:?}");
    }
    assert_ne!(variants[0], variants[1]);
    assert_ne!(variants[1], variants[2]);
    assert_ne!(variants[2], variants[3]);
}
