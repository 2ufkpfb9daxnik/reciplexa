//! Sandbox policy / fence tests.

use reciplexa_harden::{SandboxDecision, SandboxFence, SandboxPolicy, SandboxRequirement};

#[test]
fn default_policy_is_strict_and_requires_fence() {
    let p = SandboxPolicy::default();
    assert!(p.is_strict());
    assert_eq!(p.decide(false, false, false), SandboxDecision::RequireFence);
}

#[test]
fn denies_disallowed_capabilities() {
    let p = SandboxPolicy::default();
    assert_eq!(p.decide(true, false, false), SandboxDecision::Deny);
    assert_eq!(p.decide(false, true, false), SandboxDecision::Deny);
    assert_eq!(p.decide(false, false, true), SandboxDecision::Deny);
}

#[test]
fn none_requirement_allows() {
    let p = SandboxPolicy {
        requirement: SandboxRequirement::None,
        allow_network: true,
        allow_filesystem_write: true,
        allow_native_code: true,
    };
    assert_eq!(p.decide(true, true, true), SandboxDecision::Allow);
}

#[test]
fn fence_enter_exit() {
    let mut f = SandboxFence::new(SandboxPolicy::default());
    assert!(!f.is_entered());
    assert!(f.enter().is_ok());
    assert!(f.is_entered());
    assert_eq!(f.enter().unwrap_err(), "fence already entered");
    assert!(f.exit().is_ok());
    assert_eq!(f.exit().unwrap_err(), "fence not entered");
}

#[test]
fn sandbox_required_also_needs_fence() {
    let p = SandboxPolicy {
        requirement: SandboxRequirement::SandboxRequired,
        allow_network: false,
        allow_filesystem_write: false,
        allow_native_code: false,
    };
    assert_eq!(p.decide(false, false, false), SandboxDecision::RequireFence);
    let mut f = SandboxFence::new(SandboxPolicy {
        requirement: SandboxRequirement::None,
        ..SandboxPolicy::default()
    });
    assert!(f.enter().is_ok());
    assert!(f.exit().is_ok());
}
