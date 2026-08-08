use reciplexa_outcome::cancellation::{CancellationReason, CancellationReport};
use reciplexa_outcome::defect::{DefectCode, DefectReport, DefectScope};
use reciplexa_outcome::subject::{
    ApplicationOutcome, ExitCategory, ExitIntent, InfrastructureAbort, SubjectOutcome,
};

type S = SubjectOutcome<i32, &'static str>;
type A = ApplicationOutcome<i32, &'static str>;

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

fn map_s(outcome: S) -> S {
    outcome.map(|n| n + 1)
}

fn map_err_s(outcome: S) -> SubjectOutcome<i32, usize> {
    outcome.map_err(|e| e.len())
}

#[test]
fn cover_subject_flags() {
    let success: S = SubjectOutcome::Success(1);
    assert!(success.is_success());
    assert!(!success.is_failure());

    let failure: S = SubjectOutcome::Failure("e");
    assert!(!failure.is_success());
    assert!(failure.is_failure());

    let cancelled: S = SubjectOutcome::Cancelled;
    assert!(!cancelled.is_success());
    assert!(!cancelled.is_failure());

    let defected: S = SubjectOutcome::Defect(Box::new(defect()));
    assert!(!defected.is_success());
    assert!(!defected.is_failure());
}

#[test]
fn cover_all_subject_map_arms() {
    assert_eq!(map_s(SubjectOutcome::Success(2)), SubjectOutcome::Success(3));
    assert_eq!(map_s(SubjectOutcome::Failure("x")), SubjectOutcome::Failure("x"));
    assert_eq!(map_s(SubjectOutcome::Cancelled), SubjectOutcome::Cancelled);
    assert!(matches!(
        map_s(SubjectOutcome::Defect(Box::new(defect()))),
        SubjectOutcome::Defect(_)
    ));
}

#[test]
fn cover_all_subject_map_err_arms() {
    assert_eq!(
        map_err_s(SubjectOutcome::Success(7)),
        SubjectOutcome::Success(7)
    );
    assert_eq!(
        map_err_s(SubjectOutcome::Failure("ab")),
        SubjectOutcome::Failure(2)
    );
    assert_eq!(map_err_s(SubjectOutcome::Cancelled), SubjectOutcome::Cancelled);
    assert!(matches!(
        map_err_s(SubjectOutcome::Defect(Box::new(defect()))),
        SubjectOutcome::Defect(_)
    ));
}

#[test]
fn cover_application_flags() {
    let completed: A = ApplicationOutcome::Completed(1);
    assert!(completed.is_completed());
    assert!(!completed.is_cancelled());
    assert!(!completed.is_defected());

    let failed: A = ApplicationOutcome::Failed("boom");
    assert!(!failed.is_completed());
    assert!(!failed.is_cancelled());
    assert!(!failed.is_defected());

    let cancelled: A = ApplicationOutcome::Cancelled(cancel());
    assert!(!cancelled.is_completed());
    assert!(cancelled.is_cancelled());
    assert!(!cancelled.is_defected());

    let defected: A = ApplicationOutcome::Defected(Box::new(defect()));
    assert!(!defected.is_completed());
    assert!(!defected.is_cancelled());
    assert!(defected.is_defected());

    let requested: A = ApplicationOutcome::Requested(ExitIntent {
        category: ExitCategory::Success,
        user_message: None,
    });
    assert!(!requested.is_completed());
    assert!(!requested.is_cancelled());
    assert!(!requested.is_defected());

    let aborted: A = ApplicationOutcome::Aborted(InfrastructureAbort {
        reason: "host".into(),
    });
    assert!(!aborted.is_completed());
    assert!(!aborted.is_cancelled());
    assert!(!aborted.is_defected());
}

#[test]
fn exit_category_and_intent_partitions() {
    for cat in [
        ExitCategory::Success,
        ExitCategory::NoChanges,
        ExitCategory::UserCancelledOperation,
        ExitCategory::RestartRequested,
        ExitCategory::Custom,
    ] {
        let intent = ExitIntent {
            category: cat,
            user_message: Some("m".into()),
        };
        let cloned = intent.clone();
        assert_eq!(intent, cloned);
        let _ = format!("{cat:?}");
        let _ = format!("{intent:?}");
        assert!(!ApplicationOutcome::<i32, &'static str>::Requested(intent).is_completed());
    }
    let none_msg = ExitIntent {
        category: ExitCategory::Custom,
        user_message: None,
    };
    assert_ne!(
        none_msg,
        ExitIntent {
            category: ExitCategory::Custom,
            user_message: Some("x".into()),
        }
    );
}

#[test]
fn subject_application_clone_eq_debug() {
    let subjects: [S; 4] = [
        SubjectOutcome::Success(1),
        SubjectOutcome::Failure("e"),
        SubjectOutcome::Cancelled,
        SubjectOutcome::Defect(Box::new(defect())),
    ];
    for v in &subjects {
        assert_eq!(v, &v.clone());
        let _ = format!("{v:?}");
    }
    assert_ne!(subjects[0], subjects[1]);

    let apps: [A; 6] = [
        ApplicationOutcome::Completed(1),
        ApplicationOutcome::Requested(ExitIntent {
            category: ExitCategory::RestartRequested,
            user_message: None,
        }),
        ApplicationOutcome::Failed("e"),
        ApplicationOutcome::Cancelled(cancel()),
        ApplicationOutcome::Defected(Box::new(defect())),
        ApplicationOutcome::Aborted(InfrastructureAbort {
            reason: "disk".into(),
        }),
    ];
    for v in &apps {
        assert_eq!(v, &v.clone());
        let _ = format!("{v:?}");
    }
    assert_ne!(apps[0], apps[2]);

    let a = InfrastructureAbort {
        reason: "x".into(),
    };
    assert_eq!(a, a.clone());
    assert!(format!("{a:?}").contains("x"));
}
