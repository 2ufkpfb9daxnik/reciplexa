use reciplexa_native::adapter::AdapterContract;
use reciplexa_native::lifecycle::*;

fn sample_contract() -> AdapterContract {
    AdapterContract {
        name: "test".into(),
        abi_version: 1,
        pure_replayable: true,
    }
}

#[test]
fn is_usable_only_when_ready() {
    let mut inst = NativeInstance::new(1, sample_contract());
    assert!(!inst.is_usable());
    inst.mark_ready();
    assert!(inst.is_usable());
    inst.quarantine();
    assert!(!inst.is_usable());
    inst.state = InstanceState::Shutdown;
    assert!(!inst.is_usable());
}

#[test]
fn quarantine_sets_state() {
    let mut inst = NativeInstance::new(2, sample_contract());
    inst.mark_ready();
    inst.quarantine();
    assert_eq!(inst.state, InstanceState::Quarantined);
}

#[test]
fn new_instance_starts_as_candidate() {
    let inst = NativeInstance::new(3, sample_contract());
    assert_eq!(inst.state, InstanceState::Candidate);
    assert_eq!(inst.instance_id, 3);
}

#[test]
fn negotiating_state_is_not_usable() {
    let mut inst = NativeInstance::new(4, sample_contract());
    inst.state = InstanceState::Negotiating;
    assert!(!inst.is_usable());
    inst.mark_ready();
    assert!(inst.is_usable());
}
