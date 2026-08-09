//! Adapter registry, ABI negotiation, quarantine.

use std::collections::HashMap;
use std::sync::Arc;

use crate::adapter::{NativeProvider, PortableImageDecode};
use crate::foreign::ForeignValue;
use crate::lifecycle::{InstanceState, NativeInstance};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NegotiationError {
    AbiMismatch { expected: u32, found: u32 },
    NotRegistered(String),
    Quarantined(String),
    NotReady(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuarantineReason {
    ContractViolation(String),
    AbiMismatch,
}

#[derive(Default)]
pub struct AdapterRegistry {
    providers: HashMap<String, Arc<dyn NativeProvider>>,
    instances: HashMap<u64, NativeInstance>,
    next_id: u64,
    quarantined: HashMap<String, QuarantineReason>,
}

impl AdapterRegistry {
    pub fn with_portable_defaults() -> Self {
        let mut reg = Self::default();
        reg.register(Arc::new(PortableImageDecode));
        reg
    }

    pub fn register(&mut self, provider: Arc<dyn NativeProvider>) {
        let name = provider.contract().name.clone();
        self.providers.insert(name, provider);
    }

    pub fn negotiate(&mut self, name: &str, required_abi: u32) -> Result<u64, NegotiationError> {
        if self.quarantined.contains_key(name) {
            return Err(NegotiationError::Quarantined(name.into()));
        }
        let provider = self
            .providers
            .get(name)
            .ok_or_else(|| NegotiationError::NotRegistered(name.into()))?;
        let contract = provider.contract();
        if contract.abi_version != required_abi {
            self.quarantined
                .insert(name.into(), QuarantineReason::AbiMismatch);
            return Err(NegotiationError::AbiMismatch {
                expected: required_abi,
                found: contract.abi_version,
            });
        }
        self.next_id += 1;
        let id = self.next_id;
        let mut inst = NativeInstance::new(id, contract);
        inst.state = InstanceState::Negotiating;
        inst.mark_ready();
        self.instances.insert(id, inst);
        Ok(id)
    }

    /// Drop a provider by name (ready instances become NotRegistered on call).
    pub fn unregister(&mut self, name: &str) {
        self.providers.remove(name);
    }

    pub fn call(
        &mut self,
        instance_id: u64,
        op: &str,
        args: &[ForeignValue],
    ) -> Result<ForeignValue, NegotiationError> {
        let name = {
            let inst = self
                .instances
                .get(&instance_id)
                .ok_or(NegotiationError::NotReady(instance_id))?;
            if !inst.is_usable() {
                return Err(NegotiationError::NotReady(instance_id));
            }
            inst.contract.name.clone()
        };
        let provider = self
            .providers
            .get(&name)
            .ok_or_else(|| NegotiationError::NotRegistered(name.clone()))?
            .clone();
        match provider.call(op, args) {
            Ok(v) => Ok(v),
            Err(msg) => {
                self.quarantined
                    .insert(name, QuarantineReason::ContractViolation(msg.clone()));
                self.instances
                    .get_mut(&instance_id)
                    .expect("ready instance must exist")
                    .quarantine();
                Err(NegotiationError::Quarantined(msg))
            }
        }
    }

    pub fn shutdown(&mut self, instance_id: u64) {
        if let Some(inst) = self.instances.get_mut(&instance_id) {
            inst.state = InstanceState::Shutdown;
        }
    }
}
