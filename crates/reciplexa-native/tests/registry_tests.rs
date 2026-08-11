use reciplexa_native::foreign::ForeignValue;
use reciplexa_native::registry::*;

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
    let mut reg = AdapterRegistry::with_portable_defaults();
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
    assert!(matches!(
        reg.negotiate("portable-image-decode", 1),
        Err(NegotiationError::Quarantined(_))
    ));
    assert!(matches!(
        reg.call(id, "decode_header", &[]),
        Err(NegotiationError::NotReady(_))
    ));
}

#[test]
fn unregister_makes_call_not_registered() {
    let mut reg = AdapterRegistry::with_portable_defaults();
    let id = reg.negotiate("portable-image-decode", 1).unwrap();
    reg.unregister("portable-image-decode");
    assert!(matches!(
        reg.call(id, "decode_header", &[]),
        Err(NegotiationError::NotRegistered(_))
    ));
}

#[test]
fn quarantine_reason_variants_eq() {
    assert_eq!(QuarantineReason::AbiMismatch, QuarantineReason::AbiMismatch);
    assert_eq!(
        QuarantineReason::ContractViolation("x".into()),
        QuarantineReason::ContractViolation("x".into())
    );
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

#[test]
fn quarantine_builds_structured_defect_report() {
    let mut reg = AdapterRegistry::with_portable_defaults();
    let report = reg.defect_report_for_quarantine("portable-image-decode", "unknown op");
    assert_eq!(report.code.namespace, "foreign");
    assert_eq!(report.code.code, "ADAPTER_CONTRACT");
    assert!(report.violated_invariant.contains("portable-image-decode"));
}
