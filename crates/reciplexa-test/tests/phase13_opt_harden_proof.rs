//! Phase 13 conformance: optimization, hardening, and proof obligation stubs.

use reciplexa_harden::{
    CapabilityAudit, DecodeBudget, FuzzHarnessHook, PrivacyAudit, PrivacyLabel, ResourceKind,
    ResourceQuota, SandboxDecision, SandboxPolicy,
};
use reciplexa_opt::{
    CacheDomain, CompileJob, CompileJobGraph, Fingerprint, InvalidationKind, InvalidationKey,
    InvalidationSet, JobId, JobKind, JobStatus, MemoCache, MemoKey, MemoLayer, NativeInstanceCache,
    NativeInstanceKey, ParallelSchedule,
};
use reciplexa_proof::{Lemma, LemmaRegistry, ProofObligation, ProofPriority, ProofStatus};
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn invalidation_and_memo_agree() {
    let case = ConformanceCase::new("TEST-OPT-001", "Phase 13", "invalidation clears memo");
    run_conformance(&case, || {
        let mut cache = MemoCache::new();
        let key = MemoKey::new(MemoLayer::Ir, Fingerprint::of(&"unit"));
        cache.insert(key.clone(), Fingerprint::of(&"out"));
        let set = InvalidationSet::from_kind(
            InvalidationKind::Local,
            InvalidationKey::new(CacheDomain::Ir, Fingerprint::of(&"unit")),
        );
        assert_eq!(cache.invalidate(&set), 1);
        assert!(cache.is_empty());
    });
}

#[test]
fn parallel_schedule_deterministic() {
    let case = ConformanceCase::new("TEST-OPT-002", "Phase 13", "parallel waves");
    run_conformance(&case, || {
        let mut g = CompileJobGraph::new();
        g.insert(CompileJob {
            id: JobId(1),
            kind: JobKind::TypeCheck,
            input: Fingerprint::of(&1u8),
            deps: vec![],
            status: JobStatus::Pending,
        });
        g.insert(CompileJob {
            id: JobId(2),
            kind: JobKind::LowerIr,
            input: Fingerprint::of(&2u8),
            deps: vec![JobId(1)],
            status: JobStatus::Pending,
        });
        let a = ParallelSchedule::from_graph(&g);
        let b = ParallelSchedule::from_graph(&g);
        assert_eq!(a, b);
        assert_eq!(a.wave_count(), 2);
    });
}

#[test]
fn native_cache_rejects_quarantine_reuse_path() {
    let case = ConformanceCase::new("TEST-OPT-003", "Phase 13", "native instance cache");
    run_conformance(&case, || {
        let mut cache = NativeInstanceCache::new();
        let key = NativeInstanceKey::new(Fingerprint::of(&"a"), Fingerprint::of(&"b"));
        let h = cache.insert_ready(key.clone());
        assert!(cache.quarantine(&key));
        assert!(cache.lookup(&key).is_none());
        let h2 = cache.insert_ready(key);
        assert_ne!(h, h2);
    });
}

#[test]
fn decode_budget_and_quota_stop_early() {
    let case = ConformanceCase::new("TEST-HARDEN-001", "Phase 13", "budgets");
    run_conformance(&case, || {
        let mut b = DecodeBudget::new(8, 8, 8);
        assert!(b.charge_bytes(8).is_ok());
        assert!(b.charge_bytes(1).is_err());
        let mut q = ResourceQuota::new(16, 2, 100, 100);
        assert!(q.charge(ResourceKind::MemoryBytes, 16).is_ok());
        assert!(q.charge(ResourceKind::MemoryBytes, 1).is_err());
    });
}

#[test]
fn sandbox_and_audit_and_fuzz_hooks() {
    let case = ConformanceCase::new("TEST-HARDEN-002", "Phase 13", "sandbox audit fuzz");
    run_conformance(&case, || {
        let p = SandboxPolicy::default();
        assert_eq!(p.decide(true, false, false), SandboxDecision::Deny);
        let cap = CapabilityAudit::new(vec!["fs".into()], vec![]);
        assert!(!cap.is_clean());
        let priv_a = PrivacyAudit::new(vec![("s".into(), PrivacyLabel::Secret)]);
        assert!(priv_a.has_secrets());
        let mut fuzz = FuzzHarnessHook::new();
        let seed = FuzzHarnessHook::corpus_seed_empty();
        let out = fuzz.run_case(&seed, |p| !p.is_empty());
        assert!(FuzzHarnessHook::check_expectation(&seed, out));
    });
}

#[test]
fn proof_roadmap_registry() {
    let case = ConformanceCase::new("TEST-PROOF-001", "Phase 13", "lemma registry");
    run_conformance(&case, || {
        let obs = ProofObligation::roadmap_defaults();
        assert_eq!(obs.len(), ProofPriority::all().len());
        let mut reg = LemmaRegistry::new();
        reg.insert(
            Lemma::new("progress", ProofPriority::CoreTypeSafety, "progress").with_status(
                ProofStatus::Draft,
            ),
        );
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.discharged_count(), 0);
    });
}
