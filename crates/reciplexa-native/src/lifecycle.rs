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
