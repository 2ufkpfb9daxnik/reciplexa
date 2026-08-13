//! N6 tip i: reciplexa-test outcome.rs — pair success + panic arms on the
//! *same* monomorphs (prior tips split type params and left panic/success
//! region ends uncovered per Instantiation).

use reciplexa_outcome::defect::{DefectCode, DefectReport, DefectScope};
use reciplexa_outcome::subject::SubjectOutcome;
use reciplexa_test::{assert_subject_failure, assert_subject_success, TestOutcome, TestSubject};

#[test]
fn outcome_n6i_same_mono_success_and_failure_ok_arms() {
    // Shared type params with panic counterparts below.
    assert_subject_success::<String, i32>(SubjectOutcome::Success(9), 9);
    assert_subject_success::<&'static str, u64>(SubjectOutcome::Success(3), 3);
    assert_subject_success::<(), bool>(SubjectOutcome::Success(true), true);

    assert_subject_failure::<String, i32>(SubjectOutcome::Failure("e".into()), "e".into());
    assert_subject_failure::<&'static str, u64>(SubjectOutcome::Failure("nope"), "nope");
    assert_subject_failure::<i32, bool>(SubjectOutcome::Failure(7), 7);

    // is_passed both arms for several concrete A
    for passed in [
        TestOutcome::Passed(1u8).is_passed(),
        TestOutcome::Passed("x".to_string()).is_passed(),
        TestOutcome::Passed(true).is_passed(),
    ] {
        assert!(passed);
    }
    assert!(!TestOutcome::<u8>::Failed("f".into()).is_passed());
    assert!(!TestOutcome::<String>::Failed("f".into()).is_passed());
    assert!(!TestOutcome::<bool>::Failed("f".into()).is_passed());

    let report = DefectReport::new(
        3,
        DefectCode::new("n6i", "D"),
        "inv",
        DefectScope::Task,
        "sub",
        "msg",
    );
    assert!(!TestOutcome::<u8>::Defected(Box::new(report.clone())).is_passed());
    assert!(!TestOutcome::<String>::Defected(Box::new(report)).is_passed());

    let s1 = TestSubject::new("CID-N6I", "SEC-A");
    let s2 = TestSubject::new(String::from("CID-N6I-2"), String::from("SEC-B"));
    assert!(s1.label().contains("CID-N6I"));
    assert!(s2.clone().label().contains("CID-N6I-2"));
    assert!(format!("{s1:?}").contains("CID-N6I"));
}

#[test]
#[should_panic(expected = "expected SubjectOutcome::Success")]
fn outcome_n6i_success_panic_string_i32() {
    assert_subject_success::<String, i32>(SubjectOutcome::Failure("boom".into()), 1);
}

#[test]
#[should_panic(expected = "expected SubjectOutcome::Success")]
fn outcome_n6i_success_panic_str_u64() {
    assert_subject_success::<&'static str, u64>(SubjectOutcome::Failure("boom"), 1);
}

#[test]
#[should_panic(expected = "expected SubjectOutcome::Success")]
fn outcome_n6i_success_panic_unit_bool() {
    assert_subject_success::<(), bool>(SubjectOutcome::Failure(()), false);
}

#[test]
#[should_panic(expected = "expected SubjectOutcome::Failure")]
fn outcome_n6i_failure_panic_string_i32() {
    assert_subject_failure::<String, i32>(SubjectOutcome::Success(1), "e".into());
}

#[test]
#[should_panic(expected = "expected SubjectOutcome::Failure")]
fn outcome_n6i_failure_panic_str_u64() {
    assert_subject_failure::<&'static str, u64>(SubjectOutcome::Success(1), "e");
}

#[test]
#[should_panic(expected = "expected SubjectOutcome::Failure")]
fn outcome_n6i_failure_panic_i32_bool() {
    assert_subject_failure::<i32, bool>(SubjectOutcome::Success(true), 0);
}
