//! Phase 11 conformance: native foreign boundary.

use reciplexa_native::adapter::NativeProvider;
use reciplexa_native::{
    differential_decode_header, select_image_decode, AdapterRegistry, ForeignValue,
    NegotiationError, PortableImageDecode,
};

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

#[test]
fn phase11_unselected_binary_not_loaded() {
    let p = select_image_decode(None).unwrap();
    assert_eq!(p.contract().name, "portable-image-decode");
}

#[test]
fn phase11_process_helper_differential_when_available() {
    // Prefer cargo-built helper next to tests; skip gracefully if absent.
    let helper = std::env::var_os("CARGO_BIN_EXE_rpx-native-image")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            let mut p = std::env::current_exe().ok()?.parent()?.to_path_buf();
            // target/debug/deps -> target/debug
            p.pop();
            p.push("rpx-native-image");
            #[cfg(windows)]
            p.set_extension("exe");
            p.exists().then_some(p)
        });
    let Some(helper) = helper else {
        // Still validate portable self-equivalence.
        let portable = PortableImageDecode;
        assert!(differential_decode_header(&portable, &portable, b"").unwrap());
        return;
    };
    let native = select_image_decode(Some(&helper)).unwrap();
    let portable = PortableImageDecode;
    assert!(differential_decode_header(native.as_ref(), &portable, b"").unwrap());
}
