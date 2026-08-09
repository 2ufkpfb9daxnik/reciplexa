//! Coverage gap tests for reciplexa-harden.

use reciplexa_harden::{
    AdversarialCase, FuzzHarnessHook, FuzzOutcome, ResourceKind, ResourceQuota,
};

#[test]
fn used_and_release_all_kinds() {
    let mut q = ResourceQuota::new(100, 10, 1000, 1000);
    for kind in [
        ResourceKind::MemoryBytes,
        ResourceKind::OpenFiles,
        ResourceKind::WallTimeMs,
        ResourceKind::CpuTimeMs,
    ] {
        assert!(q.charge(kind, 1).is_ok());
        assert_eq!(q.used(kind), 1);
        q.release(kind, 1);
        assert_eq!(q.used(kind), 0);
    }
}

#[test]
fn fuzz_expectation_accept_path_and_default() {
    let mut h = FuzzHarnessHook::default();
    let ok = AdversarialCase::new("ok", b"ab".to_vec(), false);
    let out = h.run_case(&ok, |_| true);
    assert_eq!(out, FuzzOutcome::Accepted);
    assert!(FuzzHarnessHook::check_expectation(&ok, out));
    assert!(!FuzzHarnessHook::check_expectation(
        &ok,
        FuzzOutcome::Rejected
    ));
    assert!(!FuzzHarnessHook::check_expectation(
        &ok,
        FuzzOutcome::TimedOut
    ));
}
