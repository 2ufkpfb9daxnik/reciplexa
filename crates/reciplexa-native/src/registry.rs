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
        inst.mark_ready();
        self.instances.insert(id, inst);
        Ok(id)
    }

    pub fn call(
        &self,
        instance_id: u64,
        op: &str,
        args: &[ForeignValue],
    ) -> Result<ForeignValue, NegotiationError> {
        let inst = self
            .instances
            .get(&instance_id)
            .ok_or(NegotiationError::NotReady(instance_id))?;
        if !inst.is_usable() {
            return Err(NegotiationError::NotReady(instance_id));
        }
        let provider = self
            .providers
            .get(&inst.contract.name)
            .ok_or_else(|| NegotiationError::NotRegistered(inst.contract.name.clone()))?;
        provider
            .call(op, args)
            .map_err(NegotiationError::Quarantined)
    }

    pub fn shutdown(&mut self, instance_id: u64) {
        if let Some(inst) = self.instances.get_mut(&instance_id) {
            inst.state = InstanceState::Shutdown;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negotiates_portable_image_decode() {
        let mut reg = AdapterRegistry::with_portable_defaults();
        let id = reg.negotiate("portable-image-decode", 1).unwrap();
        let out = reg.call(id, "decode_header", &[]).unwrap();
        assert!(matches!(out, ForeignValue::Bytes(_)));
    }

    #[test]
    fn rejects_abi_mismatch() {
        let mut reg = AdapterRegistry::with_portable_defaults();
        assert!(matches!(
            reg.negotiate("portable-image-decode", 99),
            Err(NegotiationError::AbiMismatch { .. })
        ));
    }

    #[test]
    fn rejects_not_registered() {
        let mut reg = AdapterRegistry::default();
        assert!(matches!(
            reg.negotiate("missing-provider", 1),
            Err(NegotiationError::NotRegistered(_))
        ));
    }

    #[test]
    fn abi_mismatch_quarantines_subsequent_negotiate() {
        let mut reg = AdapterRegistry::with_portable_defaults();
        let _ = reg.negotiate("portable-image-decode", 99);
        assert!(matches!(
            reg.negotiate("portable-image-decode", 1),
            Err(NegotiationError::Quarantined(_))
        ));
    }

    #[test]
    fn call_unknown_instance_id() {
        let reg = AdapterRegistry::with_portable_defaults();
        assert!(matches!(
            reg.call(9999, "decode_header", &[]),
            Err(NegotiationError::NotReady(9999))
        ));
    }

    #[test]
    fn provider_error_maps_to_quarantined() {
        let mut reg = AdapterRegistry::with_portable_defaults();
        let id = reg.negotiate("portable-image-decode", 1).unwrap();
        assert!(matches!(
            reg.call(id, "unknown-op", &[]),
            Err(NegotiationError::Quarantined(_))
        ));
    }

    #[test]
    fn shutdown_unknown_instance_is_noop() {
        let mut reg = AdapterRegistry::with_portable_defaults();
        reg.shutdown(4242);
        assert!(matches!(
            reg.call(4242, "decode_header", &[]),
            Err(NegotiationError::NotReady(4242))
        ));
    }

    #[test]
    fn shutdown_makes_instance_unusable() {
        let mut reg = AdapterRegistry::with_portable_defaults();
        let id = reg.negotiate("portable-image-decode", 1).unwrap();
        reg.shutdown(id);
        assert!(matches!(
            reg.call(id, "decode_header", &[]),
            Err(NegotiationError::NotReady(_))
        ));
    }
}
