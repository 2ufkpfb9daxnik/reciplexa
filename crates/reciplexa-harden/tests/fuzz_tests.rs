//! Fuzz / adversarial harness hook tests.

use reciplexa_harden::{AdversarialCase, FuzzHarnessHook, FuzzOutcome};

#[test]
fn run_case_counts_accept_reject() {
    let mut h = FuzzHarnessHook::new();
    let ok = AdversarialCase::new("tiny", b"ok".to_vec(), false);
    let bad = AdversarialCase::new("bad", b"!!!!".to_vec(), true);
    let a = h.run_case(&ok, |p| p.len() <= 2);
    let b = h.run_case(&bad, |p| p.iter().all(|c| c.is_ascii_alphanumeric()));
    assert_eq!(a, FuzzOutcome::Accepted);
    assert_eq!(b, FuzzOutcome::Rejected);
    assert_eq!(h.cases_run, 2);
    assert_eq!(h.accepts, 1);
    assert_eq!(h.rejects, 1);
    assert!(FuzzHarnessHook::check_expectation(&ok, a));
    assert!(FuzzHarnessHook::check_expectation(&bad, b));
}

#[test]
fn corpus_seeds() {
    let empty = FuzzHarnessHook::corpus_seed_empty();
    assert!(empty.expect_reject);
    assert!(empty.payload.is_empty());
    let huge = FuzzHarnessHook::corpus_seed_huge(8);
    assert_eq!(huge.payload.len(), 8);
}

#[test]
fn expectation_mismatch() {
    let case = AdversarialCase::new("x", vec![1], true);
    assert!(!FuzzHarnessHook::check_expectation(
        &case,
        FuzzOutcome::Accepted
    ));
    assert!(!FuzzHarnessHook::check_expectation(
        &case,
        FuzzOutcome::Crashed
    ));
    let _ = FuzzOutcome::TimedOut;
}
