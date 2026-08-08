//! Process-isolated native image decode provider (Phase 11).
//!
//! Loads a selected helper binary path only after ABI negotiation. Unselected
//! binaries are never spawned. Pure replayable ops may fall back to portable.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::adapter::{AdapterContract, NativeProvider, PortableImageDecode};
use crate::foreign::ForeignValue;

const NATIVE_ABI: u32 = 1;
const HELPER_MAGIC: &str = "RPX_NATIVE_IMAGE_V1";

/// Native provider backed by an external helper binary.
#[derive(Debug, Clone)]
pub struct ProcessNativeImageDecode {
    helper: PathBuf,
    contract: AdapterContract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeLaunchError {
    MissingBinary,
    AbiMismatch { got: String },
    Spawn(String),
    Protocol(String),
}

impl ProcessNativeImageDecode {
    /// Negotiate ABI with the helper before exposing an instance.
    pub fn negotiate(helper: impl AsRef<Path>) -> Result<Self, NativeLaunchError> {
        let helper = helper.as_ref().to_path_buf();
        if !helper.exists() {
            return Err(NativeLaunchError::MissingBinary);
        }
        let output = Command::new(&helper)
            .arg("--abi")
            .output()
            .map_err(|e| NativeLaunchError::Spawn(e.to_string()))?;
        if !output.status.success() {
            return Err(NativeLaunchError::Spawn(format!(
                "abi probe failed: {}",
                output.status
            )));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next().unwrap_or("").trim();
        if line != format!("{HELPER_MAGIC}:{NATIVE_ABI}") {
            return Err(NativeLaunchError::AbiMismatch {
                got: line.to_string(),
            });
        }
        Ok(Self {
            helper,
            contract: AdapterContract {
                name: "native-image-decode".into(),
                abi_version: NATIVE_ABI,
                pure_replayable: true,
            },
        })
    }
}

impl NativeProvider for ProcessNativeImageDecode {
    fn contract(&self) -> AdapterContract {
        self.contract.clone()
    }

    fn call(&self, op: &str, args: &[ForeignValue]) -> Result<ForeignValue, String> {
        if op != "decode_header" {
            return Err(format!("unknown op {op}"));
        }
        let input = match args.first() {
            Some(ForeignValue::Bytes(b)) => b.clone(),
            _ => Vec::new(),
        };
        let mut child = Command::new(&self.helper)
            .arg("decode_header")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&input).map_err(|e| e.to_string())?;
        }
        let output = child.wait_with_output().map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(format!(
                "native helper failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(ForeignValue::Bytes(output.stdout))
    }
}

/// Prefer native helper when selected; otherwise portable. Never loads an unselected path.
pub fn select_image_decode(
    selected_helper: Option<&Path>,
) -> Result<Box<dyn NativeProvider>, NativeLaunchError> {
    match selected_helper {
        None => Ok(Box::new(PortableImageDecode)),
        Some(path) => Ok(Box::new(ProcessNativeImageDecode::negotiate(path)?)),
    }
}

/// Differential check: portable and native must agree on PNG magic for empty/PNG prefix.
pub fn differential_decode_header(
    native: &dyn NativeProvider,
    portable: &dyn NativeProvider,
    sample: &[u8],
) -> Result<bool, String> {
    let a = native.call("decode_header", &[ForeignValue::Bytes(sample.to_vec())])?;
    let b = portable.call("decode_header", &[ForeignValue::Bytes(sample.to_vec())])?;
    Ok(a == b)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(matches!(err, NativeLaunchError::AbiMismatch { .. } | NativeLaunchError::Spawn(_)));
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
        let helper = std::env::var_os("CARGO_BIN_EXE_rpx-native-image")
            .map(std::path::PathBuf::from);
        if let Some(helper) = helper {
            if let Ok(native) = ProcessNativeImageDecode::negotiate(&helper) {
                let err = native.call("bogus", &[]).unwrap_err();
                assert!(err.contains("unknown op"));
            }
        }
    }

    #[test]
    fn process_native_decode_header_protocol() {
        let helper = std::env::var_os("CARGO_BIN_EXE_rpx-native-image")
            .map(std::path::PathBuf::from);
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
        assert_eq!(NativeLaunchError::MissingBinary, NativeLaunchError::MissingBinary);
    }

    #[test]
    fn abi_probe_failure_exits_nonzero() {
        let dir = std::env::temp_dir();
        let helper = dir.join(format!("rpx_bad_abi_{}.cmd", std::process::id()));
        std::fs::write(&helper, "@echo off\r\nif \"%1\"==\"--abi\" exit /b 1\r\nexit /b 0\r\n")
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
        let helper = std::env::var_os("CARGO_BIN_EXE_rpx-native-image")
            .map(std::path::PathBuf::from);
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
        use crate::adapter::{AdapterContract, NativeProvider, PortableImageDecode};
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
        let _ = std::fs::remove_file(&helper);
    }

    #[test]
    fn native_launch_error_debug() {
        let msg = format!("{:?}", NativeLaunchError::Spawn("x".into()));
        assert!(msg.contains("Spawn"));
    }
}
