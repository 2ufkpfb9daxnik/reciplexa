use reciplexa_native::adapter::*;
use reciplexa_native::foreign::ForeignValue;

#[test]
fn portable_contract_metadata() {
    let p = PortableImageDecode;
    let c = p.contract();
    assert_eq!(c.name, "portable-image-decode");
    assert_eq!(c.abi_version, 1);
    assert!(c.pure_replayable);
}

#[test]
fn decode_header_returns_png_magic() {
    let p = PortableImageDecode;
    let out = p.call("decode_header", &[]).unwrap();
    assert_eq!(out, ForeignValue::Bytes(vec![0x89, 0x50, 0x4E, 0x47]));
}

#[test]
fn rejects_unknown_op() {
    let p = PortableImageDecode;
    let err = p.call("resize", &[]).unwrap_err();
    assert!(err.contains("unknown op"));
}
