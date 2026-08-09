use reciplexa_native::adapter::{AdapterContract, NativeProvider, PortableImageDecode};
use reciplexa_native::foreign::ForeignValue;
use reciplexa_native::process::*;
use std::path::Path;

#[test]
fn unselected_uses_portable() {
    let p = select_image_decode(None).unwrap();
    assert_eq!(p.contract().name, "portable-image-decode");
}

#[test]
fn missing_binary_returns_error() {
    let err = ProcessNativeImageDecode::negotiate("/nonexistent/rpx-native-image").unwrap_err();
    assert_eq!(err, NativeLaunchError::MissingBinary);
}

#[test]
fn abi_mismatch_when_helper_wrong_magic() {
    let helper = std::env::current_exe().unwrap();
    let err = ProcessNativeImageDecode::negotiate(&helper).unwrap_err();
    assert!(matches!(
        err,
        NativeLaunchError::AbiMismatch { .. } | NativeLaunchError::Spawn(_)
    ));
}

#[test]
fn select_some_when_helper_exists() {
    let helper = std::env::var_os("CARGO_BIN_EXE_rpx-native-image")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            let mut p = std::env::current_exe().ok()?.parent()?.to_path_buf();
            p.pop();
            p.push("rpx-native-image");
            #[cfg(windows)]
            p.set_extension("exe");
            p.exists().then_some(p)
        });
    if let Some(helper) = helper {
        let p = select_image_decode(Some(&helper)).unwrap();
        assert_eq!(p.contract().name, "native-image-decode");
    }
}

#[test]
fn portable_rejects_unknown_op() {
    let p = PortableImageDecode;
    assert!(p.call("bogus", &[]).is_err());
}

#[test]
fn decode_header_with_empty_args() {
    let p = PortableImageDecode;
    let out = p.call("decode_header", &[]).unwrap();
    assert!(matches!(out, ForeignValue::Bytes(_)));
}

#[test]
fn differential_agrees_portable_with_self() {
    let p = PortableImageDecode;
    assert!(differential_decode_header(&p, &p, b"").unwrap());
}

#[test]
fn process_native_unknown_op_errors() {
    let helper = std::env::var_os("CARGO_BIN_EXE_rpx-native-image").map(std::path::PathBuf::from);
    if let Some(helper) = helper {
        if let Ok(native) = ProcessNativeImageDecode::negotiate(&helper) {
            let err = native.call("bogus", &[]).unwrap_err();
            assert!(err.contains("unknown op"));
        }
    }
}

#[test]
fn process_native_decode_header_protocol() {
    let helper = std::env::var_os("CARGO_BIN_EXE_rpx-native-image").map(std::path::PathBuf::from);
    if let Some(helper) = helper {
        if let Ok(native) = ProcessNativeImageDecode::negotiate(&helper) {
            let png_prefix = b"\x89PNG\r\n\x1a\n";
            let out = native
                .call("decode_header", &[ForeignValue::Bytes(png_prefix.to_vec())])
                .unwrap();
            assert!(matches!(out, ForeignValue::Bytes(_)));
        }
    }
}

#[test]
fn spawn_failure_on_directory() {
    let dir = std::env::temp_dir();
    let err = ProcessNativeImageDecode::negotiate(&dir).unwrap_err();
    assert!(matches!(
        err,
        NativeLaunchError::Spawn(_) | NativeLaunchError::AbiMismatch { .. }
    ));
}

#[test]
fn native_launch_error_eq() {
    assert_eq!(
        NativeLaunchError::MissingBinary,
        NativeLaunchError::MissingBinary
    );
}

#[test]
fn abi_probe_failure_exits_nonzero() {
    let dir = std::env::temp_dir();
    let helper = dir.join(format!("rpx_bad_abi_{}.cmd", std::process::id()));
    std::fs::write(
        &helper,
        "@echo off\r\nif \"%1\"==\"--abi\" exit /b 1\r\nexit /b 0\r\n",
    )
    .unwrap();
    let err = ProcessNativeImageDecode::negotiate(&helper).unwrap_err();
    assert!(matches!(
        err,
        NativeLaunchError::Spawn(_) | NativeLaunchError::AbiMismatch { .. }
    ));
    let _ = std::fs::remove_file(&helper);
}

#[test]
fn call_with_non_bytes_arg_uses_empty_input() {
    let helper = std::env::var_os("CARGO_BIN_EXE_rpx-native-image").map(std::path::PathBuf::from);
    if let Some(helper) = helper {
        if let Ok(native) = ProcessNativeImageDecode::negotiate(&helper) {
            let out = native
                .call("decode_header", &[ForeignValue::Float(1.0)])
                .unwrap();
            assert!(matches!(out, ForeignValue::Bytes(_)));
        }
    }
}

#[test]
fn differential_detects_mismatch() {
    struct AlwaysEmpty;
    impl NativeProvider for AlwaysEmpty {
        fn contract(&self) -> AdapterContract {
            PortableImageDecode.contract()
        }
        fn call(&self, _: &str, _: &[ForeignValue]) -> Result<ForeignValue, String> {
            Ok(ForeignValue::Bytes(vec![]))
        }
    }
    let portable = PortableImageDecode;
    let empty = AlwaysEmpty;
    assert!(!differential_decode_header(&empty, &portable, b"sample").unwrap());
}

#[test]
fn call_fails_when_helper_removed() {
    let dir = std::env::temp_dir();
    let helper = dir.join(format!("rpx_rm_helper_{}.cmd", std::process::id()));
    std::fs::write(
        &helper,
        "@echo off\r\nif \"%1\"==\"--abi\" (echo RPX_NATIVE_IMAGE_V1:1& exit /b 0)\r\nexit /b 0\r\n",
    )
    .unwrap();
    let native = ProcessNativeImageDecode::negotiate(&helper).unwrap();
    std::fs::remove_file(&helper).unwrap();
    let err = native.call("decode_header", &[]).unwrap_err();
    assert!(err.contains("native helper failed"));
}

#[test]
fn call_output_err_uses_synthetic_failure() {
    // Extensionless missing paths make CreateProcess return Err (unlike missing
    // .cmd, which yields Ok + non-zero via the shell).
    let native = ProcessNativeImageDecode::from_helper_path(r"C:\nonexistent\rpx_noext");
    let err = native.call("decode_header", &[]).unwrap_err();
    assert!(err.contains("native helper failed"));
    assert_eq!(native.contract().name, "native-image-decode");
}

#[test]
fn select_missing_helper_errors() {
    match select_image_decode(Some(Path::new("/nonexistent/rpx-native-image"))) {
        Err(NativeLaunchError::MissingBinary) => {}
        Err(e) => panic!("expected MissingBinary, got {e:?}"),
        Ok(_) => panic!("expected MissingBinary, got Ok"),
    }
}

#[test]
fn differential_surfaces_provider_error() {
    struct AlwaysErr;
    impl NativeProvider for AlwaysErr {
        fn contract(&self) -> AdapterContract {
            PortableImageDecode.contract()
        }
        fn call(&self, _: &str, _: &[ForeignValue]) -> Result<ForeignValue, String> {
            Err("boom".into())
        }
    }
    let portable = PortableImageDecode;
    let err = differential_decode_header(&AlwaysErr, &portable, b"").unwrap_err();
    assert_eq!(err, "boom");
}

#[test]
fn differential_surfaces_portable_error() {
    struct AlwaysErr;
    impl NativeProvider for AlwaysErr {
        fn contract(&self) -> AdapterContract {
            PortableImageDecode.contract()
        }
        fn call(&self, _: &str, _: &[ForeignValue]) -> Result<ForeignValue, String> {
            Err("portable-boom".into())
        }
    }
    let portable_ok = PortableImageDecode;
    let err = differential_decode_header(&portable_ok, &AlwaysErr, b"").unwrap_err();
    assert_eq!(err, "portable-boom");
}

#[test]
fn negotiate_success_with_stub_helper() {
    let dir = std::env::temp_dir();
    let helper = dir.join(format!("rpx_stub_abi_{}.cmd", std::process::id()));
    std::fs::write(
        &helper,
        "@echo off\r\nif \"%1\"==\"--abi\" (echo RPX_NATIVE_IMAGE_V1:1& exit /b 0)\r\nexit /b 0\r\n",
    )
    .unwrap();
    let native = ProcessNativeImageDecode::negotiate(&helper).unwrap();
    assert_eq!(native.contract().name, "native-image-decode");
    let out = native.call("decode_header", &[]).unwrap();
    assert!(matches!(out, ForeignValue::Bytes(_)));
    let err = native.call("bogus", &[]).unwrap_err();
    assert!(err.contains("unknown op"));
    let out2 = native
        .call("decode_header", &[ForeignValue::Float(1.0)])
        .unwrap();
    assert!(matches!(out2, ForeignValue::Bytes(_)));
    let out3 = native
        .call("decode_header", &[ForeignValue::Bytes(b"\x89PNG".to_vec())])
        .unwrap();
    assert!(matches!(out3, ForeignValue::Bytes(_)));
    let p = select_image_decode(Some(&helper)).unwrap();
    assert_eq!(p.contract().name, "native-image-decode");
    let _ = std::fs::remove_file(&helper);
}

#[test]
fn negotiate_empty_abi_is_protocol_error() {
    let dir = std::env::temp_dir();
    let helper = dir.join(format!("rpx_empty_abi_{}.cmd", std::process::id()));
    std::fs::write(
        &helper,
        "@echo off\r\nif \"%1\"==\"--abi\" (exit /b 0)\r\nexit /b 0\r\n",
    )
    .unwrap();
    let err = ProcessNativeImageDecode::negotiate(&helper).unwrap_err();
    assert!(matches!(err, NativeLaunchError::Protocol(_)));
    let _ = std::fs::remove_file(&helper);
}

#[test]
fn native_launch_error_protocol_eq() {
    assert_eq!(
        NativeLaunchError::Protocol("a".into()),
        NativeLaunchError::Protocol("a".into())
    );
}

#[test]
fn native_launch_error_debug() {
    let msg = format!("{:?}", NativeLaunchError::Spawn("x".into()));
    assert!(msg.contains("Spawn"));
    let abi = format!("{:?}", NativeLaunchError::AbiMismatch { got: "bad".into() });
    assert!(abi.contains("AbiMismatch"));
}

#[test]
fn negotiate_abi_mismatch_exact() {
    let dir = std::env::temp_dir();
    let helper = dir.join(format!("rpx_bad_magic_{}.cmd", std::process::id()));
    std::fs::write(
        &helper,
        "@echo off\r\nif \"%1\"==\"--abi\" (echo WRONG_MAGIC:1& exit /b 0)\r\nexit /b 0\r\n",
    )
    .unwrap();
    let err = ProcessNativeImageDecode::negotiate(&helper).unwrap_err();
    assert!(matches!(err, NativeLaunchError::AbiMismatch { .. }));
    let _ = std::fs::remove_file(&helper);
}

#[test]
fn decode_header_helper_failure_surfaces_stderr() {
    let dir = std::env::temp_dir();
    let helper = dir.join(format!("rpx_fail_decode_{}.cmd", std::process::id()));
    std::fs::write(
        &helper,
        concat!(
            "@echo off\r\n",
            "if \"%1\"==\"--abi\" (echo RPX_NATIVE_IMAGE_V1:1& exit /b 0)\r\n",
            "if \"%1\"==\"decode_header\" (echo fail 1>&2& exit /b 1)\r\n",
            "exit /b 0\r\n"
        ),
    )
    .unwrap();
    let native = ProcessNativeImageDecode::negotiate(&helper).unwrap();
    let err = native.call("decode_header", &[]).unwrap_err();
    assert!(err.contains("native helper failed"));
    let _ = std::fs::remove_file(&helper);
}

#[test]
fn process_native_contract_clone() {
    let dir = std::env::temp_dir();
    let helper = dir.join(format!("rpx_stub_contract_{}.cmd", std::process::id()));
    std::fs::write(
        &helper,
        "@echo off\r\nif \"%1\"==\"--abi\" (echo RPX_NATIVE_IMAGE_V1:1& exit /b 0)\r\nexit /b 0\r\n",
    )
    .unwrap();
    let native = ProcessNativeImageDecode::negotiate(&helper).unwrap();
    assert_eq!(native.contract().abi_version, 1);
    assert!(native.contract().pure_replayable);
    let _ = std::fs::remove_file(&helper);
}
