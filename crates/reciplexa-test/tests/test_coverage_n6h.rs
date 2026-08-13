//! N6 tip: reciplexa-test outcome.rs residual derive / panic / flag arms.

use reciplexa_outcome::defect::{DefectCode, DefectReport, DefectScope};
use reciplexa_outcome::subject::SubjectOutcome;
use reciplexa_test::{assert_subject_failure, assert_subject_success, TestOutcome, TestSubject};

#[test]
fn outcome_n6h_variants_and_labels() {
    let passed = TestOutcome::Passed(42u32);
    let failed = TestOutcome::<u32>::Failed("nope".into());
    let report = DefectReport::new(
        9,
        DefectCode::new("t", "D"),
        "inv",
        DefectScope::Task,
        "sub",
        "msg",
    );
    let defected = TestOutcome::<u32>::Defected(Box::new(report.clone()));

    assert!(passed.is_passed());
    assert!(!failed.is_passed());
    assert!(!defected.is_passed());
    assert_eq!(passed.clone(), TestOutcome::Passed(42));
    assert_ne!(passed, failed);
    assert_ne!(failed, defected);
    assert!(format!("{passed:?}").contains("Passed"));
    assert!(format!("{failed:?}").contains("Failed"));
    assert!(format!("{defected:?}").contains("Defected"));

    let subject = TestSubject::new("CID-N6H", "SEC-1");
    let subject2 = subject.clone();
    assert!(subject.label().contains("CID-N6H"));
    assert!(format!("{subject2:?}").contains("CID-N6H"));

    assert_subject_success::<(), i32>(SubjectOutcome::Success(7), 7);
    assert_subject_failure::<&str, ()>(SubjectOutcome::Failure("e"), "e");
}

#[test]
#[should_panic(expected = "expected SubjectOutcome::Success")]
fn outcome_n6h_success_panic_arm() {
    assert_subject_success::<&str, i32>(SubjectOutcome::Failure("nope"), 1);
}

#[test]
#[should_panic(expected = "expected SubjectOutcome::Failure")]
fn outcome_n6h_failure_panic_arm() {
    assert_subject_failure::<&str, i32>(SubjectOutcome::Success(1), "e");
}
