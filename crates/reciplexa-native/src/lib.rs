//! Native foreign boundary (Phase 11).

#![forbid(unsafe_code)]

pub mod adapter;
pub mod foreign;
pub mod lifecycle;
pub mod registry;

pub use adapter::{AdapterContract, NativeProvider, PortableImageDecode};
pub use foreign::ForeignValue;
pub use lifecycle::{InstanceState, NativeInstance};
pub use registry::{AdapterRegistry, NegotiationError, QuarantineReason};
