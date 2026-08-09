//! Adversarial / fuzz harness hooks (stubs for libFuzzer-style drivers).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdversarialCase {
    pub name: String,
    pub payload: Vec<u8>,
    pub expect_reject: bool,
}

impl AdversarialCase {
    pub fn new(name: impl Into<String>, payload: Vec<u8>, expect_reject: bool) -> Self {
        Self {
            name: name.into(),
            payload,
            expect_reject,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuzzOutcome {
    Accepted,
    Rejected,
    Crashed,
    TimedOut,
}

/// Hook surface for external fuzzers — no unsafe decoding here.
#[derive(Debug, Clone, Default)]
pub struct FuzzHarnessHook {
    pub cases_run: u64,
    pub rejects: u64,
    pub accepts: u64,
}

impl FuzzHarnessHook {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classify a raw byte blob with a pure predicate (stub entry point).
    pub fn run_case(&mut self, case: &AdversarialCase, classify: fn(&[u8]) -> bool) -> FuzzOutcome {
        self.cases_run += 1;
        let accepted = classify(&case.payload);
        if accepted {
            self.accepts += 1;
            FuzzOutcome::Accepted
        } else {
            self.rejects += 1;
            FuzzOutcome::Rejected
        }
    }

    /// Check that expected rejection matches the outcome.
    pub fn check_expectation(case: &AdversarialCase, outcome: FuzzOutcome) -> bool {
        matches!(
            (case.expect_reject, outcome),
            (true, FuzzOutcome::Rejected) | (false, FuzzOutcome::Accepted)
        )
    }

    pub fn corpus_seed_empty() -> AdversarialCase {
        AdversarialCase::new("empty", Vec::new(), true)
    }

    pub fn corpus_seed_huge(size: usize) -> AdversarialCase {
        AdversarialCase::new("huge", vec![0u8; size], true)
    }
}
