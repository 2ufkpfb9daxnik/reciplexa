//! Native instance lifecycle.

use crate::adapter::AdapterContract;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceState {
    Candidate,
    Negotiating,
    Ready,
    Quarantined,
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct NativeInstance {
    pub contract: AdapterContract,
    pub state: InstanceState,
    pub instance_id: u64,
}

impl NativeInstance {
    pub fn new(instance_id: u64, contract: AdapterContract) -> Self {
        Self {
            contract,
            state: InstanceState::Candidate,
            instance_id,
        }
    }

    pub fn mark_ready(&mut self) {
        self.state = InstanceState::Ready;
    }

    pub fn quarantine(&mut self) {
        self.state = InstanceState::Quarantined;
    }

    pub fn is_usable(&self) -> bool {
        self.state == InstanceState::Ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::AdapterContract;

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
}
