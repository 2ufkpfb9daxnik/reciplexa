//! Tip residual assert/outcome coverage.

use reciplexa_outcome::defect::{DefectCode, DefectReport, DefectScope};
use reciplexa_outcome::subject::SubjectOutcome;
use reciplexa_test::{
    assert_diagnostic_codes, assert_eq_structured, assert_subject_failure, assert_subject_success,
    StructuredDiff, TestOutcome, TestSubject,
};

#[test]
fn structured_diff_and_diagnostic_helpers() {
    let diff = StructuredDiff {
        path: "a.b".into(),
        expected: "1".into(),
        actual: "2".into(),
    };
    assert!(diff.to_string().contains("a.b"));
    assert_eq_structured(&1u32, &1u32);
    assert_diagnostic_codes(&["n/c".into()], &["n/c"]);
}

#[test]
fn test_outcome_and_subject_helpers() {
    assert!(TestOutcome::Passed(1).is_passed());
    assert!(!TestOutcome::<()>::Failed("x".into()).is_passed());
    let report = DefectReport::new(
        1,
        DefectCode::new("t", "D"),
        "inv",
        DefectScope::Task,
        "sub",
        "msg",
    );
    assert!(!TestOutcome::<()>::Defected(Box::new(report.clone())).is_passed());
    // Force Debug/Clone/PartialEq region entries for all harness variants.
    let passed = TestOutcome::Passed(7i32);
    let failed = TestOutcome::<i32>::Failed("f".into());
    let defected = TestOutcome::<i32>::Defected(Box::new(report));
    assert!(format!("{passed:?}").contains("Passed"));
    assert!(format!("{failed:?}").contains("Failed"));
    assert!(format!("{defected:?}").contains("Defected"));
    assert_eq!(passed.clone(), TestOutcome::Passed(7));
    assert_ne!(passed, failed);
    let subject = TestSubject::new("CID", "SEC");
    assert!(subject.label().contains("CID"));
    assert_subject_success::<(), i32>(SubjectOutcome::Success(7), 7);
    assert_subject_failure::<&str, ()>(SubjectOutcome::Failure("e"), "e");
}
