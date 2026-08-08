//! Phase 11 conformance: native foreign boundary.

use reciplexa_native::{
    AdapterRegistry, ForeignValue, NegotiationError, PortableImageDecode,
};
use reciplexa_native::adapter::NativeProvider;

#[test]
fn phase11_negotiates_before_use() {
    let mut reg = AdapterRegistry::with_portable_defaults();
    let id = reg.negotiate("portable-image-decode", 1).unwrap();
    let out = reg.call(id, "decode_header", &[]).unwrap();
    assert!(matches!(out, ForeignValue::Bytes(_)));
}

#[test]
fn phase11_rejects_abi_mismatch_before_ready() {
    let mut reg = AdapterRegistry::with_portable_defaults();
    assert!(matches!(
        reg.negotiate("portable-image-decode", 999),
        Err(NegotiationError::AbiMismatch { .. })
    ));
}

#[test]
fn phase11_portable_native_equivalence() {
    let portable = PortableImageDecode;
    let out = portable.call("decode_header", &[]).unwrap();
    let mut reg = AdapterRegistry::with_portable_defaults();
    let id = reg.negotiate("portable-image-decode", 1).unwrap();
    let native_out = reg.call(id, "decode_header", &[]).unwrap();
    assert_eq!(out, native_out);
}

#[test]
fn phase11_shutdown_marks_instance_unusable() {
    let mut reg = AdapterRegistry::with_portable_defaults();
    let id = reg.negotiate("portable-image-decode", 1).unwrap();
    reg.shutdown(id);
    assert!(matches!(
        reg.call(id, "decode_header", &[]),
        Err(NegotiationError::NotReady(_))
    ));
}
